pub mod command_details;
pub mod commands;

use thiserror::Error;

pub struct Handler;

impl Handler {
    pub fn new() -> Result<Handler, HandlerError> {
        Ok(Handler)
    }
}

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Unrecognized command ({0})")]
    UnrecognizedCommand(String),
    #[error("Command was empty")]
    EmptyCommand,
    #[error("Internal error, could not build response")]
    TargetNone,
    #[error("Failed to send message")]
    Send(#[from] serenity::Error),
    #[error("Command can only be used in a server")]
    NotGuild,
    #[error("Timed out or had too many inputs")]
    TimeoutOrOverLimit,
    #[error("Couldn't find user")]
    UserNotFound,
    #[error("Unexpected data received from server")]
    UnexpectedData,
    #[error("Maximum number of commands reached")]
    ApplicationCommandCap,
    #[error("Internal error, could not build response")]
    EmoteLogCountNoParams,
    #[error("Internal error, could not build response")]
    CountNone,
    #[error("Received command info for unknown command")]
    CommandRegisterUnknown,
    #[error("Internal error, could not build response")]
    TypeMapNotFound,
    #[error("Could not set up application commands")]
    CommandSetup,
}

impl HandlerError {
    pub fn should_followup(&self) -> bool {
        !matches!(self, HandlerError::TimeoutOrOverLimit)
    }
}
