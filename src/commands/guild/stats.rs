use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::{CommandOptionType, CommandType},
        interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};
use tracing::*;

use crate::{
    commands::{stats::*, AppCmd},
    util::{LocalizedString},
    Handler, HandlerError
};

pub const GUILD_SUB_NAME: LocalizedString = LocalizedString {
    en: "guild"
};
pub const GUILD_SUB_DESC: LocalizedString = LocalizedString {
    en: "Emote usage statistics for the current guild!"
};

pub struct GuildStatsCmd;

#[async_trait]
impl AppCmd for GuildStatsCmd {
    fn to_application_command() -> CreateApplicationCommand
        where
            Self: Sized,
    {
        let mut cmd = CreateApplicationCommand::default();
        cmd.name(NAME.en)
            .kind(CommandType::ChatInput)
            .description(DESC.en)
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(GUILD_SUB_NAME.en)
                    .description(GUILD_SUB_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(EMOTE_OPT_NAME.en)
                            .description(EMOTE_OPT_DESC.en)
                    })
            });
        cmd
    }

    #[instrument(skip(cmd, handler, context))]
    async fn handle(
        cmd: &ApplicationCommandInteraction,
        handler: &Handler,
        context: &Context
    ) -> Result<(), HandlerError>
        where
            Self: Sized,
    {
        cmd.create_interaction_response(context, |res| {
            res.interaction_response_data(|d| d.content("stats"))
        })
            .await?;
        Ok(())
    }

    fn name() -> LocalizedString {
        NAME
    }
}