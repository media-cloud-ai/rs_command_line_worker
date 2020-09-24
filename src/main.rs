#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

use mcai_worker_sdk::{
  job::JobResult, start_worker, McaiChannel, MessageError, MessageEvent, Version,
};
use schemars::JsonSchema;

mod message;

pub mod built_info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Default)]
struct CommandLineEvent {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommandLineWorkerParameters {
  command_template: String,
  exec_dir: Option<String>,
  #[serde(flatten)]
  parameters: HashMap<String, String>,
  requirements: Option<HashMap<String, Vec<String>>>,
}

impl MessageEvent<CommandLineWorkerParameters> for CommandLineEvent {
  fn get_name(&self) -> String {
    "Command Line".to_string()
  }

  fn get_short_description(&self) -> String {
    "Execute command lines on workers".to_string()
  }

  fn get_description(&self) -> String {
    r#"Run any command line available in the command line of the worker.
Provide a template parameter, other parameters will be replaced before running."#
      .to_string()
  }

  fn get_version(&self) -> Version {
    Version::parse(built_info::PKG_VERSION).expect("unable to locate Package version")
  }

  fn process(
    &self,
    channel: Option<McaiChannel>,
    parameters: CommandLineWorkerParameters,
    job_result: JobResult,
  ) -> Result<JobResult, MessageError> {
    message::process(channel, parameters, job_result)
  }
}

fn main() {
  let message_event = CommandLineEvent::default();
  start_worker(message_event);
}
