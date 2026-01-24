use std::{collections::HashMap, thread};

use reqwest::{
  Client, ClientBuilder,
  header::{HeaderMap, HeaderValue},
};

use crate::server::DECRYPTED_CONFIG;

pub type ApiMap = HashMap<Box<str>, ApiData>;

pub struct ApiData {
  pub client: Client,
  pub completions: Box<str>,
  pub _props_url: Box<str>,
}

pub fn genapimap() -> ApiMap {
  rustls::crypto::ring::default_provider()
    .install_default()
    .expect("Failed to install rustls crypto provider");

  let mut out = HashMap::new();

  #[allow(clippy::unwrap_used)]
  thread::spawn(move || {
    DECRYPTED_CONFIG
      .blocking_read()
      .llama
      .models
      .iter()
      .for_each(|(k, v)| {
        _ = out.insert(
          k.clone(),
          ApiData {
            completions: format!("{}/v1/chat/completions", v.url).into_boxed_str(),
            _props_url: format!("{}/props", v.url).into_boxed_str(),
            client: {
              let mut builder = ClientBuilder::new().user_agent("AHQ AI");

              if let Some(key) = &v.apikey {
                let mut headers = HeaderMap::new();

                let key = key as &str;

                // SAFETY
                // DECRYPTED_CONFIG is valid for the entire lifecycle of this ApiMap structure
                // The key already is correctly concatenated with `Bearer {token}`
                let key: &'static str = unsafe { &*std::ptr::from_ref(key) };

                let mut auth = HeaderValue::from_static(key);
                auth.set_sensitive(true);

                headers.insert("Authorization", auth);

                builder = builder.default_headers(headers);
              }

              #[allow(clippy::expect_used)]
              builder
                .build()
                .expect("The builder failed to build, server can't start")
            },
          },
        );
      });

    out
  })
  .join()
  .unwrap()
}
