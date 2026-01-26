#![warn(clippy::pedantic)] // Set pedantic to warn initially
#![warn(clippy::nursery)]
// Set nursery to warn initially

// --- DENY MAJOR CLIPPY GROUPS (Turns all warnings from these groups into errors) ---
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
// --- TARGETED DENIALS (Critical for Production Stability) ---
// These specific lints are crucial for high-reliability code,
// often residing in the 'restriction' group which is too verbose to enable fully.
#![deny(clippy::unwrap_used)] // Prevents runtime panics from unwrap()
#![deny(clippy::expect_used)] // Prevents runtime panics from expect()
#![deny(clippy::todo)] // Prevents shipping incomplete code
#![deny(clippy::unimplemented)] // Prevents shipping incomplete code
#![deny(clippy::integer_division)] // Catches potential issues with integer division casting
#![allow(clippy::future_not_send)]

use crate::{
  auth::{
    AuthSessionManager,
    argon::{self, server::verify_server_pass},
  },
  server::apiset::{ApiMap, genapimap},
  structs::{Authentication, Config},
};
use actix_web::{App, HttpServer, web};
use chalk_rs::Chalk;
use log::{error, info, warn};
use secrecy::{ExposeSecret, SecretString};
use serde_json::from_str;
use std::{
  env, fs as stdfs,
  sync::{LazyLock, OnceLock},
  thread::{self, available_parallelism},
};
use tokio::sync::RwLock;
use zeroize::Zeroize;

pub mod admin;
pub mod auth;
pub mod chat;
pub mod http;

pub mod llama;

pub mod ffi;

pub mod apiset;

// This is the encrypted config
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
  #[allow(clippy::expect_used)]
  let data = stdfs::read_to_string("config.json").expect("Unable to load config");

  #[allow(clippy::expect_used)]
  from_str(&data).expect("Invalid configuration file, unable to parse")
});

pub static DECRYPTED_CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(decrypt_config);

pub static API_MAP: LazyLock<ApiMap> = LazyLock::new(genapimap);

pub static AUTH: OnceLock<AuthSessionManager> = OnceLock::new();

pub static REAL_ADMIN_PASSWORD: OnceLock<RwLock<SecretString>> = OnceLock::new();

fn decrypt_config() -> RwLock<Config> {
  // Decrypt config
  if let Some(pass) = REAL_ADMIN_PASSWORD.get() {
    let mut conf = CONFIG.clone();

    #[allow(clippy::expect_used)]
    let conf = thread::spawn(move || {
      argon::decrypt_config(pass.blocking_read().expose_secret(), &mut conf);

      conf
    })
    .join()
    .expect("Unable to decrypt config");

    info!("Successfully decrypted configuration");

    return RwLock::new(conf);
  }

  warn!(
    "No Server Administrator Password found to perform decryption, double check if this is a bug. If it is, create an issue immediately at https://github.com/ahqsoftorg/ahqai/issues"
  );

  RwLock::new(CONFIG.clone())
}

pub fn launch() -> Chalk {
  let mut chalk = Chalk::new();

  info!("AHQ-AI Server v{}", env!("CARGO_PKG_VERSION"));

  chalk.reset_style();

  chalk
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
  let mut chalk = launch();

  let admin_api = request_admin_passwd();

  info!("Decrypting configuration...");

  // Explicitly trigger the LazyLock initialization now that the password is set
  // This guarantees decryption before the server starts.
  _ = DECRYPTED_CONFIG.read().await;

  info!("Decryption successful...");

  info!("Creating API Map eagerly...");

  // Explicitly trigger the LazyLock initialization now that the password is set
  // This guarantees decryption before the server starts.
  let cap = API_MAP.len();

  info!("Api Map Created! Created {cap} keys");

  let auth = !matches!(CONFIG.authentication, Authentication::OpenToAll);
  let mut registration_api = false;

  if auth {
    info!("Starting up authentication manager using the decrypted configuration.");

    if let Authentication::Account {
      registration_allowed,
      ..
    } = &CONFIG.authentication
    {
      registration_api = *registration_allowed;
    }

    _ = AUTH.set(AuthSessionManager::create().await);
  }

  let mut server = HttpServer::new(move || {
    let mut app = App::new()
      .service(http::index)
      .route("/chat", web::get().to(chat::chat))
      .service(http::challenge);

    let auth = !matches!(CONFIG.authentication, Authentication::OpenToAll);

    if auth {
      app = app.service(auth::auth).service(http::me);
    }

    if admin_api {
      app = app
        .service(admin::verify)
        .service(admin::list)
        .service(admin::create)
        .service(admin::create_token)
        .service(admin::delete);
    }

    if registration_api {
      app = app.service(auth::register);
    }

    app
  })
  .workers(available_parallelism()?.get());

  for (host, port) in &CONFIG.binds {
    info!("Binding to {host}:{port}");
    server = server.bind((host as &str, *port))?;
  }

  info!("Server is starting");

  let out = server.run().await;

  if let Err(e) = &out {
    error!("Server exited in an error state.");
    error!("{e}");
  }

  info!("Zeroizing the decrypted configuration and server administrator key");

  DECRYPTED_CONFIG.write().await.zeroize();
  if let Some(x) = REAL_ADMIN_PASSWORD.get() {
    x.write().await.zeroize();
  }

  info!("Zeroized successfully");

  warn!("Ctrl+C detected (most probably). Starting shutdown procedure. This might take a while.");
  info!(
    "{}",
    chalk
      .red()
      .bold()
      .string(&"Please DO NOT use Ctrl+C to terminate. It will lead to data corruption!")
  );

  out
}

// Rquests admin password if needed and outputs if
// you can enable admin urls
#[allow(clippy::useless_let_if_seq, clippy::expect_used)]
fn request_admin_passwd() -> bool {
  if let Some(x) = &CONFIG.admin_pass_hash {
    let hash = x as &str;

    let passwd;

    if let Ok(x) = env::var("AHQAI_ADMIN_PASSWORD") {
      passwd = x;
    } else {
      warn!("----------------");
      warn!("THE GIVEN SERVER IS PROTECTED BY SERVER ADMIN PASSWORD");
      warn!("BUT THE `AHQAI_ADMIN_PASSWORD` VARIABLE WAS NOT FOUND");
      warn!("IN THE CURRENT SERVER ENVIRONMENT. REQUESTING MANUAL ENTRY");
      warn!("----------------");
      warn!("");

      passwd = rpassword::prompt_password("Enter your administrator password : ")
        .expect("Unable to read your password");
    }

    assert!(
      verify_server_pass(&passwd, hash).unwrap_or(false),
      "Invalid Password was provided"
    );

    info!("");
    info!("----------------");
    info!("SERVER ADMIN PASSWORD AUTH SUCCESSFUL");
    info!("SERVER WILL START UP NOW");
    info!("----------------");
    info!("");

    REAL_ADMIN_PASSWORD
      .set(RwLock::new(SecretString::from(passwd)))
      .expect("Impossible Error");

    return true;
  }

  false
}
