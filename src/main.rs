
use amqp_worker::worker::{Parameter, ParameterType};
use amqp_worker::{job::{Job, JobResult}, start_worker, MessageError, MessageEvent};
use lapin_futures::Channel;
use semver::Version;

mod message;

macro_rules! crate_version {
  () => {
    env!("CARGO_PKG_VERSION")
  };
}

#[derive(Debug)]
struct CommandLineEvent {}

impl MessageEvent for CommandLineEvent {
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
    semver::Version::parse(crate_version!()).expect("unable to locate Package version")
  }

  fn get_parameters(&self) -> Vec<Parameter> {
    vec![
      Parameter {
        identifier: "command_template".to_string(),
        label: "Command Template".to_string(),
        kind: vec![ParameterType::String],
        required: true,
      },
      Parameter {
        identifier: "exec_dir".to_string(),
        label: "Execution directory".to_string(),
        kind: vec![ParameterType::String],
        required: true,
      },
    ]
  }

  fn process(
    &self,
    channel: Option<&Channel>,
    job: &Job,
    job_result: JobResult
    ) -> Result<JobResult, MessageError> {
    message::process(channel, job, job_result)
  }
}

static COMMAND_LINE_EVENT: CommandLineEvent = CommandLineEvent {};

fn main() {
  start_worker(&COMMAND_LINE_EVENT);
}
