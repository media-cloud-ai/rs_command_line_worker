use std::collections::HashMap;

use amqp_worker::job::*;
use amqp_worker::MessageError;

const COMMAND_TEMPLATE_PARAM_ID: &'static str = "command_template";
const EXEC_DIR_PARAM_ID: &'static str = "exec_dir";
const LIBRARIES_PARAM_ID: &'static str = "libraries";

const FIXED_PARAM_IDS: [&'static str; 3] = [COMMAND_TEMPLATE_PARAM_ID, EXEC_DIR_PARAM_ID, LIBRARIES_PARAM_ID];


pub fn process(message: &str) -> Result<JobResult, MessageError> {
  let job = Job::new(message)?;
  debug!("reveived message: {:?}", job);

  match job.check_requirements() {
    Ok(_) => {}
    Err(message) => { return Err(message); }
  }

  let lib_path = job.get_array_of_strings_parameter(LIBRARIES_PARAM_ID).unwrap_or(vec![]);
  let exec_dir = job.get_string_parameter(EXEC_DIR_PARAM_ID);
  let command_template = job.get_string_parameter(COMMAND_TEMPLATE_PARAM_ID)
    .ok_or(MessageError::RuntimeError(format!("Invalid job message: missing expected '{}' parameter.", COMMAND_TEMPLATE_PARAM_ID)))?;

  let param_map: HashMap<String, Option<String>> = job.get_parameters_as_map();
  let command = compile_command_template(command_template, param_map);

  launch(command.as_str(), lib_path, exec_dir);

  Ok(JobResult::from(job))
}

fn compile_command_template(command_template: String, param_map: HashMap<String, Option<String>>) -> String {
  let mut compiled_command_template = command_template.clone();
  param_map.iter()
    .filter(|(key, _value)| !FIXED_PARAM_IDS.contains(&key.as_str()))
    .filter(|(_key, value)| value.is_some())
    .for_each(|(key, value)|
      compiled_command_template = compiled_command_template.replace(format!("{{{}}}", key).as_str(), value.clone().unwrap().as_str()));
  compiled_command_template
}

fn launch(command: &str, lib_path: Vec<String>, exec_dir: Option<String>) {
  unimplemented!()
}

#[test]
pub fn test_compile_command_template() {
  let mut command_template = "ls {option} {path}".to_string();
  let mut parameters = HashMap::new();
  parameters.insert("option".to_string(), Some("-l".to_string()));
  parameters.insert("path".to_string(), Some(".".to_string()));

  let command = compile_command_template(command_template, parameters);
  assert_eq!("ls -l .", command.as_str());
}

#[test]
pub fn test_compile_command_template_with_fixed_params() {
  let mut command_template = "ls {option} {path}".to_string();
  let mut parameters = HashMap::new();
  parameters.insert("option".to_string(), Some("-l".to_string()));
  parameters.insert("path".to_string(), Some(".".to_string()));
  parameters.insert(COMMAND_TEMPLATE_PARAM_ID.to_string(), Some(command_template.clone()));
  parameters.insert(EXEC_DIR_PARAM_ID.to_string(), Some("/path/to/somewhere".to_string()));
  parameters.insert(LIBRARIES_PARAM_ID.to_string(), Some("/path/to/lib".to_string()));

  let command = compile_command_template(command_template, parameters);
  assert_eq!("ls -l .", command.as_str());
}
