use crate::CommandLineWorkerParameters;
use mcai_worker_sdk::{
  job::{JobResult, JobStatus},
  McaiChannel, MessageError,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

const COMMAND_TEMPLATE_IDENTIFIER: &str = "command_template";
const EXECUTION_DIRECTORY_PARAMETER: &str = "exec_dir";

const INTERNAL_PARAM_IDENTIFIERS: [&str; 2] =
  [COMMAND_TEMPLATE_IDENTIFIER, EXECUTION_DIRECTORY_PARAMETER];

pub fn process(
  _channel: Option<McaiChannel>,
  parameters: CommandLineWorkerParameters,
  job_result: JobResult,
) -> Result<JobResult, MessageError> {
  let command = compile_command_template(parameters.command_template, parameters.parameters);

  let mut result = launch(command, parameters.exec_dir).map_err(|msg| {
    MessageError::ProcessingError(
      job_result
        .clone()
        .with_status(JobStatus::Error)
        .with_message(&msg),
    )
  })?;

  // limit return message size to 1MB
  result.truncate(1024 * 1024);

  Ok(
    job_result
      .with_status(JobStatus::Completed)
      .with_message(&result),
  )
}

fn compile_command_template(
  command_template: String,
  param_map: HashMap<String, String>,
) -> String {
  let mut compiled_command_template = command_template;
  param_map
    .iter()
    .filter(|(key, _value)| !INTERNAL_PARAM_IDENTIFIERS.contains(&key.as_str()))
    .for_each(|(key, value)| {
      compiled_command_template = compiled_command_template.replace(&format!("{{{}}}", key), value)
    });
  compiled_command_template
}

fn launch(command: String, exec_dir: Option<String>) -> Result<String, String> {
  let mut splitted_command: Vec<&str> = command.split(' ').collect();
  if splitted_command.is_empty() {
    return Err("missing executable in the command line template".to_string());
  }
  let program = splitted_command.remove(0);

  let mut process = Command::new(program);

  if let Some(current_dir) = exec_dir {
    process.current_dir(Path::new(&current_dir));
  }

  let output = process
    .args(splitted_command.as_slice())
    .output()
    .map_err(|error| {
      format!(
        "An error occurred process command: {}.\n{:?}",
        command, error
      )
    })?;

  if output.status.success() {
    Ok(String::from_utf8(output.stdout).unwrap_or_default())
  } else {
    let mut message = output.stderr;
    message.extend(&output.stdout);
    Err(String::from_utf8(message).unwrap_or_default())
  }
}

#[test]
pub fn test_compile_command_template() {
  let command_template = "ls {option} {path}".to_string();
  let mut parameters = HashMap::new();
  parameters.insert("option".to_string(), "-l".to_string());
  parameters.insert("path".to_string(), ".".to_string());

  let command = compile_command_template(command_template, parameters);
  assert_eq!("ls -l .", command.as_str());
}

#[test]
pub fn test_compile_command_template_with_doubles() {
  let command_template = "ls {option} {path} {option}".to_string();
  let mut parameters = HashMap::new();
  parameters.insert("option".to_string(), "-l".to_string());
  parameters.insert("path".to_string(), ".".to_string());

  let command = compile_command_template(command_template, parameters);
  assert_eq!("ls -l . -l", command.as_str());
}

#[test]
pub fn test_compile_command_template_with_fixed_params() {
  let command_template = "ls {option} {path}".to_string();
  let mut parameters = HashMap::new();
  parameters.insert("option".to_string(), "-l".to_string());
  parameters.insert("path".to_string(), ".".to_string());
  parameters.insert(
    COMMAND_TEMPLATE_IDENTIFIER.to_string(),
    command_template.clone(),
  );
  parameters.insert(
    EXECUTION_DIRECTORY_PARAMETER.to_string(),
    "/path/to/somewhere".to_string(),
  );

  let command = compile_command_template(command_template, parameters);
  assert_eq!("ls -l .", command.as_str());
}

#[test]
pub fn test_launch() {
  let command = "ls .".to_string();
  let exec_dir = None;
  let result = launch(command, exec_dir);
  assert!(result.is_ok());

  let program_output = result.unwrap();
  assert!(program_output.contains("Cargo.toml"));
  assert!(program_output.contains("Cargo.lock"));
}

#[test]
pub fn test_launch_with_exec_dir() {
  let command = "ls .".to_string();
  let exec_dir = Some("./src".to_string());
  let result = launch(command, exec_dir);
  assert!(result.is_ok());

  let program_output = result.unwrap();
  assert!(program_output.contains("main.rs"));
  assert!(program_output.contains("message.rs"));
}

#[test]
pub fn test_launch_error() {
  let command = "ls sdjqenfdcnekbnbsdvjhqr".to_string();
  let exec_dir = None;
  let result = launch(command, exec_dir);
  assert!(result.is_err());

  let error_message = result.unwrap_err();
  assert!(error_message.contains("ls:"));
  assert!(error_message.contains("sdjqenfdcnekbnbsdvjhqr"));
}

#[test]
pub fn test_process() {
  use mcai_worker_sdk::job::Job;
  use mcai_worker_sdk::ParametersContainer;

  let message = r#"{
    "job_id": 123,
    "parameters": [
      {
        "id": "command_template",
        "type": "string",
        "value": "ls {option} {path}"
      },
      {
        "id": "option",
        "type": "string",
        "value": "-lh"
      },
      {
        "id": "path",
        "type": "string",
        "value": "."
      },
      {
        "id": "exec_dir",
        "type": "string",
        "value": "./src"
      }
    ]
  }"#;

  let job = Job::new(message).unwrap();
  let job_result = JobResult::new(job.job_id);
  let parameters: CommandLineWorkerParameters = job.get_parameters().unwrap();
  let result = process(None, parameters, job_result);

  assert!(result.is_ok());
  let job_result = result.unwrap();
  assert_eq!(123, job_result.get_job_id());
  assert_eq!(&JobStatus::Completed, job_result.get_status());
  let message_param = job_result.get_parameter::<String>("message");
  assert!(message_param.is_ok());
  assert!(message_param.unwrap().contains("main.rs"));
}

#[test]
pub fn test_process_with_requirements() {
  use mcai_worker_sdk::job::Job;
  use mcai_worker_sdk::ParametersContainer;

  let message = r#"{
    "job_id": 123,
    "parameters": [
      {
        "id": "command_template",
        "type": "string",
        "value": "ls {option} {path}"
      },
      {
        "id": "option",
        "type": "string",
        "value": "-lh"
      },
      {
        "id": "path",
        "type": "string",
        "value": "."
      },
      {
        "id": "exec_dir",
        "type": "string",
        "value": "./src"
      },
      {
        "id": "requirements",
        "type": "requirements",
        "value": {
          "paths": [
            "./src"
          ]
        }
      }
    ]
  }"#;

  let job = Job::new(message).unwrap();
  let job_result = JobResult::new(job.job_id);
  let parameters: CommandLineWorkerParameters = job.get_parameters().unwrap();
  let result = process(None, parameters, job_result);

  assert!(result.is_ok());
  let job_result = result.unwrap();
  assert_eq!(123, job_result.get_job_id());
  assert_eq!(&JobStatus::Completed, job_result.get_status());
  let message_param = job_result.get_parameter::<String>("message");
  assert!(message_param.is_ok());
  assert!(message_param.unwrap().contains("main.rs"));
}

#[test]
pub fn test_process_with_error() {
  use mcai_worker_sdk::job::Job;

  let message = r#"{
    "job_id": 123,
    "parameters": [
      {
        "id": "command_template",
        "type": "string",
        "value": "ls {option} {path}"
      },
      {
        "id": "option",
        "type": "string",
        "value": "-lh"
      },
      {
        "id": "path",
        "type": "string",
        "value": "qmslkjggsdlvnqrdgwdnvqrgn"
      },
      {
        "id": "exec_dir",
        "type": "string",
        "value": "./src"
      }
    ]
  }"#;

  let job = Job::new(message).unwrap();
  let job_result = JobResult::new(job.job_id);
  let parameters: CommandLineWorkerParameters = job.get_parameters().unwrap();
  let result = process(None, parameters, job_result);

  assert!(result.is_err());
  let _error = result.unwrap_err();
}
