use std::str::FromStr;
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::{CommandOptionType, CommandType},
        interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};
use serenity::builder::CreateEmbed;
use tracing::*;
use serenity::model::guild::Member;
use serenity::utils::Color;

use crate::{
    commands::{AppCmd, option_data::*},
    Handler,
    HandlerError, util::LocalizedString,
};
use crate::builders::roles::{roles_to_field};
use crate::handler::command_details::parse_command_members;

pub const NAME: LocalizedString = LocalizedString {
    en: "user"
};
pub const DESC: LocalizedString = LocalizedString {
    en: "Commands accessing the user!"
};

pub struct GuildUserCmd;

enum GuildUserEmbedTypes {
    Roles,
    Avatar,
    Nick,
    Id,
    Created,
    Joined
}

impl FromStr for GuildUserEmbedTypes {

    type Err = ();

    fn from_str(input: &str) -> Result<GuildUserEmbedTypes, Self::Err> {
        match input {
            "roles"  => Ok(GuildUserEmbedTypes::Roles),
            "avatar"  => Ok(GuildUserEmbedTypes::Avatar),
            "nick"  => Ok(GuildUserEmbedTypes::Nick),
            "id" => Ok(GuildUserEmbedTypes::Id),
            "created" => Ok(GuildUserEmbedTypes::Created),
            "joined" => Ok(GuildUserEmbedTypes::Joined),
            _      => Err(()),
        }
    }
}


fn create_field_from_embed_types<'b>(embed_types: &Vec<GuildUserEmbedTypes>, member: &Member, mut embed: &'b mut CreateEmbed) -> &'b CreateEmbed {
    for i in embed_types {
        match i {
            GuildUserEmbedTypes::Roles => {
                embed = roles_to_field(&member.roles, None, embed);
            },
            GuildUserEmbedTypes::Avatar => {
                if let Some(avatar_url) = member.user.avatar_url() {
                    embed.image(avatar_url);
                } else {
                    embed.image(member.user.default_avatar_url());
                }
            }
            GuildUserEmbedTypes::Nick => {
                embed.field("Nickname", member.nick.clone().unwrap_or(String::from("No Nickname")), true);
            }
            GuildUserEmbedTypes::Id => {
                embed.field("Id", &member.user.id, true);
            }
            GuildUserEmbedTypes::Created => {
                embed.field("Created", &member.user.created_at(), true);
            }
            GuildUserEmbedTypes::Joined => {
                embed.field("Joined", &member.joined_at.unwrap().to_string(), true);
            }
        };
    }
    embed
}

fn create_embed_single_member(embed_type: &String, member: &Member) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(format!("{}#{}", member.user.name, member.user.discriminator))
        .description(format!("Created: {}", member.user.created_at()));
    if embed_type != "avatar" {
        if let Some(avatar_url) = member.user.avatar_url() {
            embed.thumbnail(avatar_url);
        } else {
            embed.thumbnail(member.user.default_avatar_url());
        }
    }
    match embed_type.as_str() {
        "info" => {
            let embed_types = vec![GuildUserEmbedTypes::Nick, GuildUserEmbedTypes::Id, GuildUserEmbedTypes::Nick, GuildUserEmbedTypes::Roles];
            embed = create_field_from_embed_types(&embed_types, member, &mut embed).to_owned();
        }
        value => {
            if let Ok(guild_user_embed_type) = GuildUserEmbedTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                embed = create_field_from_embed_types(&embed_types, member, &mut embed).to_owned();
            }
        }
    }
    embed
}

fn create_embed_members(embed_type: &String, members: &Vec<Member>) -> Vec<CreateEmbed> {
    let mut embeds = vec![];

    match members.len() {
        0 => {
            let mut embed = CreateEmbed::default();
            embed.title(format!("Error"))
                .description("No user found!")
                .color(Color::RED);
            embeds.push(embed)
        }
        1 => {
            embeds.push(create_embed_single_member(embed_type, members.first().unwrap()));
        }
        _ => {
            for i in members {
                let mut embed = CreateEmbed::default();
                embed.title("Profiles")
                    .description("");
                embeds.push(embed);
            }
        }
    }
    return embeds;
}

#[async_trait]
impl AppCmd for GuildUserCmd {
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
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(AVATAR.en)
                    .description(AVATAR_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(ROLES.en)
                    .description(ROLES_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(NICK.en)
                    .description(NICK_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(ID.en)
                    .description(ID_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(CREATED.en)
                    .description(CREATED_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(JOINED.en)
                    .description(JOINED_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            });
        cmd
    }

    #[instrument(skip(cmd, handler, context))]
    async fn handle(
        cmd: &ApplicationCommandInteraction,
        handler: &Handler,
        context: &Context,
    ) -> Result<(), HandlerError>
        where
            Self: Sized,
    {
        let user_id_options = cmd.data.resolved.users.keys();
        let mut selected_users = Vec::new();
        if user_id_options.len() == 0 {
            selected_users.push(cmd.member.clone().unwrap())
        }

        let mut embeds = vec![];

        if let Some(response_type) = cmd.data.options.first() {
            for j in &response_type.options {
                if j.name == "members" {
                    selected_users = parse_command_members(j, &context, &cmd).await
                }
            }
            embeds = create_embed_members(&response_type.name, &selected_users)
        }

        cmd.create_interaction_response(context, |res| {
            res.interaction_response_data(|d| {
                d.add_embeds(embeds)
            })
        })
            .await?;
        Ok(())
    }

    fn name() -> LocalizedString {
        NAME
    }
}