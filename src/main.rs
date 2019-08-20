
extern crate amqp_worker;
#[macro_use]
extern crate log;

use amqp_worker::*;

mod message;

#[derive(Debug)]
struct CommandLineEvent {
}

impl MessageEvent for CommandLineEvent {
  fn process(&self, message: &str) -> Result<job::JobResult, MessageError> {
    message::process(message)
  }
}

static COMMAND_LINE_EVENT: CommandLineEvent = CommandLineEvent{};

fn main() {
  start_worker(&COMMAND_LINE_EVENT);
}
