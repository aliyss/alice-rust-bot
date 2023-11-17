use async_trait::async_trait;
use serenity::builder::CreateEmbed;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::guild::PartialGuild;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::{CommandOptionType, CommandType},
        interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};
use std::str::FromStr;
use tracing::*;

use crate::builders::roles::roles_to_field;
use crate::{
    commands::{option_data::*, AppCmd},
    util::LocalizedString,
    Handler, HandlerError,
};

pub const NAME: LocalizedString = LocalizedString { en: "server" };
pub const DESC: LocalizedString = LocalizedString {
    en: "Commands accessing the server!",
};

pub struct GuildServerCmd;

enum GuildServerPropertyTypes {
    Roles,
    Avatar,
    Name,
    Id,
    Created,
    Owner,
    Description,
    NSFWLevel,
    Channel,
}

impl FromStr for GuildServerPropertyTypes {
    type Err = ();

    fn from_str(input: &str) -> Result<GuildServerPropertyTypes, Self::Err> {
        match input {
            "roles" => Ok(GuildServerPropertyTypes::Roles),
            "avatar" => Ok(GuildServerPropertyTypes::Avatar),
            "id" => Ok(GuildServerPropertyTypes::Id),
            "created" => Ok(GuildServerPropertyTypes::Created),
            "name" => Ok(GuildServerPropertyTypes::Name),
            "owner" => Ok(GuildServerPropertyTypes::Owner),
            "description" => Ok(GuildServerPropertyTypes::Description),
            "nsfwlevel" => Ok(GuildServerPropertyTypes::NSFWLevel),
            "channel" => Ok(GuildServerPropertyTypes::Channel),
            _ => Err(()),
        }
    }
}

fn create_field_from_embed_types<'b>(
    embed_types: &Vec<GuildServerPropertyTypes>,
    server: &PartialGuild,
    mut embed: &'b mut CreateEmbed,
    command_data_option: &CommandDataOption,
) -> &'b CreateEmbed {
    for i in embed_types {
        match i {
            GuildServerPropertyTypes::Roles => {
                embed = roles_to_field(&server.roles.keys().cloned().collect(), None, embed);
            }
            GuildServerPropertyTypes::Avatar => {
                if let Some(avatar_url) = &server.icon_url() {
                    embed.image(avatar_url);
                }
            }
            GuildServerPropertyTypes::Id => {
                embed.field("Id", server.id, true);
            }
            GuildServerPropertyTypes::Created => {
                embed.field("Created", "", true);
            }
            GuildServerPropertyTypes::Name => {
                embed.field("Name", &server.name, true);
            }
            GuildServerPropertyTypes::Owner => {
                embed.field("Owner", format!("<@{}>", &server.owner_id), true);
            }
            GuildServerPropertyTypes::Description => {
                embed.field(
                    "Description",
                    server
                        .description
                        .as_ref()
                        .unwrap_or(&String::from("No description set.")),
                    true,
                );
            }
            GuildServerPropertyTypes::NSFWLevel => {
                embed.field("NSFW Level", format!("<{:#?}>", &server.nsfw_level), true);
            }
            GuildServerPropertyTypes::Channel => {
                if command_data_option.options.first().is_none() {
                    return embed;
                }

                let option = command_data_option.options.first().unwrap();

                match option.name.as_str() {
                    "afk" => {
                        let channel = if server.afk_channel_id.is_some() {
                            format!("<#{}>", &server.afk_channel_id.unwrap())
                        } else {
                            String::from("No channel defined!")
                        };
                        embed.field("AFK Channel", channel, true);
                    }
                    "rules" => {
                        let channel = if server.rules_channel_id.is_some() {
                            format!("<#{}>", &server.rules_channel_id.unwrap())
                        } else {
                            String::from("No channel defined!")
                        };
                        embed.field("Rules Channel", channel, true);
                    }
                    "widget" => {
                        let channel = if server.widget_channel_id.is_some() {
                            format!("<#{}>", &server.widget_channel_id.unwrap())
                        } else {
                            String::from("No channel defined!")
                        };
                        embed.field("Widget Channel", channel, true);
                    }
                    "system" => {
                        let channel = if server.system_channel_id.is_some() {
                            format!("<#{}>", &server.system_channel_id.unwrap())
                        } else {
                            String::from("No channel defined!")
                        };
                        embed.field("System Channel", channel, true);
                    }
                    _ => {}
                }
            }
        };
    }
    embed
}

fn create_embed_single(
    command_data_option: &CommandDataOption,
    server: PartialGuild,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed
        .title(server.name.to_string())
        .description(server.description.as_ref().unwrap_or(&String::from("")));

    let embed_type = &command_data_option.name;

    if embed_type != "avatar" {
        if let Some(avatar_url) = &server.icon_url() {
            embed.thumbnail(avatar_url);
        }
    }
    match embed_type.as_str() {
        "info" => {
            let embed_types = vec![
                GuildServerPropertyTypes::Owner,
                GuildServerPropertyTypes::Id,
                GuildServerPropertyTypes::Roles,
            ];
            embed = create_field_from_embed_types(
                &embed_types,
                &server,
                &mut embed,
                command_data_option,
            )
            .to_owned();
        }
        value => {
            if let Ok(guild_user_embed_type) = GuildServerPropertyTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                embed = create_field_from_embed_types(
                    &embed_types,
                    &server,
                    &mut embed,
                    command_data_option,
                )
                .to_owned();
            }
        }
    }
    embed
}

fn create_response_server(
    embed_type: &CommandDataOption,
    server: PartialGuild,
) -> Vec<CreateEmbed> {
    vec![(create_embed_single(embed_type, server))]
}

#[async_trait]
impl AppCmd for GuildServerCmd {
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
                    .name(INFO.en)
                    .description(INFO_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(AVATAR.en)
                    .description(AVATAR_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(ROLES.en)
                    .description(ROLES_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(ID.en)
                    .description(ID_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(CREATED.en)
                    .description(CREATED_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommandGroup)
                    .name(CHANNEL.en)
                    .description(CHANNEL_DESC.en)
                    .create_sub_option(|opt| {
                        opt.kind(CommandOptionType::SubCommand)
                            .name(AFKCHANNEL.en)
                            .description(AFKCHANNEL_DESC.en)
                    })
                    .create_sub_option(|opt| {
                        opt.kind(CommandOptionType::SubCommand)
                            .name(WIDGETCHANNEL.en)
                            .description(WIDGETCHANNEL_DESC.en)
                    })
                    .create_sub_option(|opt| {
                        opt.kind(CommandOptionType::SubCommand)
                            .name(SYSTEMCHANNEL.en)
                            .description(SYSTEMCHANNEL_DESC.en)
                    })
                    .create_sub_option(|opt| {
                        opt.kind(CommandOptionType::SubCommand)
                            .name(RULESCHANNEL.en)
                            .description(RULESCHANNEL_DESC.en)
                    })
                    .create_sub_option(|opt| {
                        opt.kind(CommandOptionType::SubCommand)
                            .name(CUSTOM.en)
                            .description(CUSTOM_DESC.en)
                            .create_sub_option(|opt| {
                                opt.kind(CommandOptionType::Channel)
                                    .name(CUSTOMCHANNEL.en)
                                    .description(CUSTOMCHANNEL_DESC.en)
                            })
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(NSFWLEVEL.en)
                    .description(NSFWLEVEL_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(DESCRIPTION.en)
                    .description(DESCRIPTION_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(OWNER.en)
                    .description(OWNER_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(USERNAME.en)
                    .description(USERNAME_DESC.en)
            });
        cmd
    }

    #[instrument(skip(cmd, _handler, context))]
    async fn handle(
        cmd: &ApplicationCommandInteraction,
        _handler: &Handler,
        context: &Context,
    ) -> Result<(), HandlerError>
    where
        Self: Sized,
    {
        let mut embeds = vec![];

        if let Some(response_type) = cmd.data.options.first() {
            if let Some(guild_id) = cmd.guild_id {
                let guild = context.http.get_guild(u64::from(guild_id)).await.unwrap();
                embeds = create_response_server(response_type, guild);
            }
        }

        cmd.create_interaction_response(context, |res| {
            res.interaction_response_data(|d| d.add_embeds(embeds))
        })
        .await?;
        Ok(())
    }

    fn name() -> LocalizedString {
        NAME
    }
}
