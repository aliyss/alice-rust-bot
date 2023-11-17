use std::{collections::HashMap, fmt::Debug, hash::Hash, str::FromStr};

use crate::handler::{Handler, HandlerError};
use crate::util::LocalizedString;
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{interaction::application_command::ApplicationCommandInteraction, CommandId},
    prelude::{Context, TypeMapKey},
};

pub mod guild;
mod option_data;

#[async_trait]
trait AppCmd {
    fn to_application_command() -> CreateApplicationCommand
    where
        Self: Sized;
    async fn handle(
        cmd: &ApplicationCommandInteraction,
        handler: &Handler,
        context: &Context,
    ) -> Result<(), HandlerError>
    where
        Self: Sized;
    fn name() -> LocalizedString;
}

#[async_trait]
pub trait CommandsEnum:
    FromStr + TypeMapKey<Value = HashMap<CommandId, Self>> + Debug + Copy + Eq + Hash
{
    async fn handle(
        self,
        cmd: &ApplicationCommandInteraction,
        handler: &Handler,
        context: &Context,
    ) -> Result<(), HandlerError>;
}
