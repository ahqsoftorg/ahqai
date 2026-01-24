use log::warn;

use crate::{
  server::{
    API_MAP,
    apiset::ApiData,
    llama::structs::{HTTPAIResponse, HTTPCompletion, Message},
  },
  structs::error::Returns,
};

pub mod structs;

static EMPTY: [Message; 0] = [];

pub struct LlamaChatHandler {
  pub msg: Vec<Message>,
  urls: &'static ApiData,
}

impl LlamaChatHandler {
  pub fn new(model: &str) -> Self {
    Self {
      #[allow(clippy::expect_used)]
      urls: API_MAP
        .get(model)
        .expect("Impossible, this means that checks were not correctly made"),
      msg: vec![],
    }
  }

  pub async fn complete(&mut self) -> Returns<&[Message]> {
    self.complete_inner().await
  }

  /// Does the AI Completion and returns the new messages
  pub async fn complete_inner(&mut self) -> Returns<&[Message]> {
    let response = self
      .urls
      .client
      .post(&self.urls.completions as &str)
      .json(&HTTPCompletion {
        model: "ahqai",
        messages: &self.msg,
        stream: false,
      })
      .send()
      .await?
      .error_for_status()?
      .json::<HTTPAIResponse>()
      .await?;

    let indexone = self.msg.len();
    let mut count = 0;

    for choice in response.choices {
      match choice.finish_reason.as_ref() {
        "stop" => {
          count += 1usize;
          // Directly append response
          self.msg.push(choice.message);
        }
        "tool_calls" => {}
        e => {
          warn!("While parsing LLAMA Chat, found \"{e}\" as exit reason");
        }
      }
    }

    if count == 0 {
      return Ok(&EMPTY);
    }

    Ok(
      self
        .msg
        .get(indexone..(indexone + count))
        .unwrap_or(&EMPTY as &[Message]),
    )
  }
}
