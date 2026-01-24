#![feature(duration_constructors)]
use std::{env::args, panic};

mod log;
mod server;
mod ui;

pub mod auth;
pub(crate) mod structs;

use chalk_rs::Chalk;

#[cfg(feature = "stdalloc")]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
  panic::set_hook(Box::new(|x| {
    let mut chalk = Chalk::new();

    if let Some(x) = x.payload_as_str() {
      println!();

      chalk.red().println(&"----------------");
      chalk
        .red()
        .underline()
        .println(&"An Critical Error has occured");
      chalk.reset_style();
      chalk
        .yellow()
        .println(&"The server was unable to achnowledge");
      chalk
        .yellow()
        .println(&"and handle the error promptly without");
      chalk.yellow().println(&"resorting to server shutdown");

      println!();

      println!("{x}");

      println!();

      chalk.red().println(&"----------------");
    } else {
      println!("ERR: Unknown");
    }

    #[cfg(not(debug_assertions))]
    std::process::exit(1);
  }));

  let mut args = args();
  _ = args.next();

  let mut config_ui = false;

  args.for_each(|x| {
    if &x == "config" {
      config_ui = true;
    } else {
      // Error out & Crash app
      panic!("Unknown arg: {x:?}");
    }
  });

  if config_ui {
    ui::ui();
  } else {
    log::setup();
    server::main().expect("Server failed to start. This is a before-any-checks-done error which means that the server couldn't even connect to the sockets.");
  }
}
