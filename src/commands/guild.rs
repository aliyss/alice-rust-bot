pub mod server;
pub mod user;

use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{interaction::application_command::ApplicationCommandInteraction, CommandId},
    prelude::{Context, TypeMapKey},
};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};
use thiserror::Error;

use crate::{util::LocalizedString, Handler, HandlerError};
use self::{server::GuildServerCmd, user::GuildUserCmd};

use super::{AppCmd, CommandsEnum};

#[derive(Debug, Clone, Copy, AsRefStr, Display, EnumIter, PartialEq, Eq, Hash)]
pub enum GuildCommands {
    Server,
    User
}

impl GuildCommands {
    pub fn to_application_command(self) -> CreateApplicationCommand {
        match self {
            GuildCommands::User => GuildUserCmd::to_application_command(),
            GuildCommands::Server => GuildServerCmd::to_application_command(),
        }
    }

    pub fn application_commands() -> impl Iterator<Item = CreateApplicationCommand> {
        Self::iter().map(Self::to_application_command)
    }

    pub fn name(self) -> LocalizedString {
        match self {
            GuildCommands::User => GuildUserCmd::name(),
            GuildCommands::Server => GuildServerCmd::name(),
        }
    }
}

#[async_trait]
impl CommandsEnum for GuildCommands {
    async fn handle(
        self,
        cmd: &ApplicationCommandInteraction,
        handler: &Handler,
        context: &Context
    ) -> Result<(), HandlerError> {
        match self {
            GuildCommands::Server => GuildServerCmd::handle(cmd, handler, context),
            GuildCommands::User => GuildUserCmd::handle(cmd, handler, context)
        }
            .await
    }
}

impl TypeMapKey for GuildCommands {
    type Value = HashMap<CommandId, Self>;
}

#[derive(Debug, Clone, Error)]
#[error("Not a valid command: {0}")]
pub struct InvalidGuildCommand(String);

impl FromStr for GuildCommands {
    type Err = InvalidGuildCommand;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GuildCommands::iter()
            .find(|cmd| cmd.name().any_eq(s))
            .ok_or_else(|| InvalidGuildCommand(s.to_string()))
    }
}