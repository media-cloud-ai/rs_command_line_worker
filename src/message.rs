use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use amqp_worker::{
  job::{Job, JobResult, JobStatus},
  MessageError,
  ParametersContainer,
};

const COMMAND_TEMPLATE_PARAM_ID: &str = "command_template";
const EXEC_DIR_PARAM_ID: &str = "exec_dir";

const INTERNAL_PARAM_IDS: [&str; 2] = [COMMAND_TEMPLATE_PARAM_ID, EXEC_DIR_PARAM_ID];

pub fn process(message: &str) -> Result<JobResult, MessageError> {
  let job = Job::new(message)?;
  debug!("Received message: {:?}", job);
  job.check_requirements()?;

  let exec_dir = job.get_string_parameter(EXEC_DIR_PARAM_ID);
  let command_template = job
    .get_string_parameter(COMMAND_TEMPLATE_PARAM_ID)
    .ok_or_else(|| {
      MessageError::ProcessingError(
        JobResult::from(&job)
          .with_status(JobStatus::Error)
          .with_message(format!(
            "Invalid job message: missing expected '{}' parameter.",
            COMMAND_TEMPLATE_PARAM_ID
          )),
      )
    })?;

  let param_map = job.get_parameters_as_map();
  let command = compile_command_template(command_template, param_map);

  let result = launch(command, exec_dir).map_err(|msg| {
    MessageError::ProcessingError(
      JobResult::from(&job)
        .with_status(JobStatus::Error)
        .with_message(msg),
    )
  })?;

  Ok(
    JobResult::from(job)
      .with_status(JobStatus::Completed)
      .with_message(result),
  )
}

fn compile_command_template(
  command_template: String,
  param_map: HashMap<String, String>,
) -> String {
  let mut compiled_command_template = command_template;
  param_map
    .iter()
    .filter(|(key, _value)| !INTERNAL_PARAM_IDS.contains(&key.as_str()))
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

  Ok(String::from_utf8(output.stdout).unwrap_or_default())
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
    COMMAND_TEMPLATE_PARAM_ID.to_string(),
    command_template.clone(),
  );
  parameters.insert(
    EXEC_DIR_PARAM_ID.to_string(),
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
pub fn test_process() {
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

  let result = process(message);
  assert!(result.is_ok());
  let job_result = result.unwrap();
  assert_eq!(123, job_result.job_id);
  assert_eq!(JobStatus::Completed, job_result.status);
  let message_param = job_result.get_string_parameter("message");
  assert!(message_param.is_some());
  assert!(message_param.unwrap().contains("main.rs"));
}
