use async_trait::async_trait;
use serenity::builder::CreateEmbed;
use serenity::model::guild::Member;
use serenity::utils::Color;
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

use crate::builders::roles::{roles_to_field, roles_to_text};
use crate::handler::command_details::parse_command_members;
use crate::{
    commands::{option_data::*, AppCmd},
    util::LocalizedString,
    Handler, HandlerError,
};

use ascii_table::AsciiTable;

pub const NAME: LocalizedString = LocalizedString { en: "user" };
pub const DESC: LocalizedString = LocalizedString {
    en: "Commands accessing the user!",
};

pub struct GuildUserCmd;

enum GuildUserPropertyTypes {
    Roles,
    Avatar,
    Nick,
    Id,
    Created,
    Joined,
    Discriminator,
    Name,
}

impl FromStr for GuildUserPropertyTypes {
    type Err = ();

    fn from_str(input: &str) -> Result<GuildUserPropertyTypes, Self::Err> {
        match input {
            "roles" => Ok(GuildUserPropertyTypes::Roles),
            "avatar" => Ok(GuildUserPropertyTypes::Avatar),
            "nick" => Ok(GuildUserPropertyTypes::Nick),
            "id" => Ok(GuildUserPropertyTypes::Id),
            "created" => Ok(GuildUserPropertyTypes::Created),
            "joined" => Ok(GuildUserPropertyTypes::Joined),
            "discriminator" => Ok(GuildUserPropertyTypes::Discriminator),
            "name" => Ok(GuildUserPropertyTypes::Name),
            _ => Err(()),
        }
    }
}

fn create_field_from_embed_types<'b>(
    embed_types: &Vec<GuildUserPropertyTypes>,
    member: &Member,
    mut embed: &'b mut CreateEmbed,
) -> &'b CreateEmbed {
    for i in embed_types {
        match i {
            GuildUserPropertyTypes::Roles => {
                embed = roles_to_field(&member.roles, None, embed);
            }
            GuildUserPropertyTypes::Avatar => {
                if let Some(avatar_url) = member.user.avatar_url() {
                    embed.image(avatar_url);
                } else {
                    embed.image(member.user.default_avatar_url());
                }
            }
            GuildUserPropertyTypes::Nick => {
                embed.field(
                    "Nickname",
                    member.nick.clone().unwrap_or(String::from("No Nickname")),
                    true,
                );
            }
            GuildUserPropertyTypes::Id => {
                embed.field("Id", member.user.id, true);
            }
            GuildUserPropertyTypes::Created => {
                embed.field("Created", member.user.created_at(), true);
            }
            GuildUserPropertyTypes::Joined => {
                embed.field("Joined", &member.joined_at.unwrap().to_string(), true);
            }
            GuildUserPropertyTypes::Discriminator => {
                embed.field(
                    "Discriminator",
                    format!("#{:04}", member.user.discriminator),
                    true,
                );
            }
            GuildUserPropertyTypes::Name => {
                embed.field("Name", &member.user.name, true);
            }
        };
    }
    embed
}

fn create_table_from_embed_types(
    embed_types: &Vec<GuildUserPropertyTypes>,
    members: &[Member],
) -> String {
    let mut ascii_table = AsciiTable::default();
    ascii_table.set_max_width(120);

    let mut data: Vec<Vec<String>> = vec![];

    let mut m = 0;

    members.iter().for_each(|member| {
        data.push(vec![]);
        ascii_table.column(0);
        data[m].push(format!(
            "{}#{:04}",
            member.user.name, member.user.discriminator
        ));
        for i in embed_types {
            match i {
                GuildUserPropertyTypes::Roles => {
                    data[m].push(roles_to_text(&member.roles));
                }
                GuildUserPropertyTypes::Avatar => {
                    if let Some(avatar_url) = member.user.avatar_url() {
                        data[m].push(avatar_url.to_string());
                    } else {
                        data[m].push(member.user.default_avatar_url());
                    }
                }
                GuildUserPropertyTypes::Nick => {
                    data[m].push(member.nick.clone().unwrap_or(String::from("No Nickname")));
                }
                GuildUserPropertyTypes::Id => {
                    data[m].push(member.user.id.to_string());
                }
                GuildUserPropertyTypes::Created => {
                    data[m].push(member.user.created_at().to_string());
                }
                GuildUserPropertyTypes::Joined => {
                    data[m].push(member.joined_at.unwrap().to_string());
                }
                GuildUserPropertyTypes::Discriminator => {
                    data[m].push(format!("#{:04}", member.user.discriminator));
                }
                GuildUserPropertyTypes::Name => {
                    data[m].push(member.user.name.to_string());
                }
            };
        }
        m += 1
    });

    let text = ascii_table.format(&data);
    String::from("```\n") + &text + &*String::from("\n```")
}

fn create_embed_single_member(embed_type: &String, member: &Member) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed
        .title(format!(
            "{}#{:04}",
            member.user.name, member.user.discriminator
        ))
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
            let embed_types = vec![
                GuildUserPropertyTypes::Nick,
                GuildUserPropertyTypes::Id,
                GuildUserPropertyTypes::Nick,
                GuildUserPropertyTypes::Roles,
            ];
            embed = create_field_from_embed_types(&embed_types, member, &mut embed).to_owned();
        }
        value => {
            if let Ok(guild_user_embed_type) = GuildUserPropertyTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                embed = create_field_from_embed_types(&embed_types, member, &mut embed).to_owned();
            }
        }
    }
    embed
}

fn create_content_multiple_members(embed_type: &str, members: &[Member]) -> String {
    match embed_type {
        "info" => {
            let embed_types = vec![
                GuildUserPropertyTypes::Nick,
                GuildUserPropertyTypes::Id,
                GuildUserPropertyTypes::Roles,
            ];
            create_table_from_embed_types(&embed_types, members).to_owned()
        }
        value => {
            if let Ok(guild_user_embed_type) = GuildUserPropertyTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                create_table_from_embed_types(&embed_types, members).to_owned()
            } else {
                String::from("")
            }
        }
    }
}

fn create_response_members(
    embed_type: &String,
    members: &Vec<Member>,
) -> (String, Vec<CreateEmbed>) {
    let mut embeds = vec![];
    let mut content = String::from("");

    match members.len() {
        0 => {
            let mut embed = CreateEmbed::default();
            embed
                .title("Error".to_string())
                .description("No user found!")
                .color(Color::RED);
            embeds.push(embed)
        }
        1 => {
            embeds.push(create_embed_single_member(
                embed_type,
                members.first().unwrap(),
            ));
        }
        _ => {
            content = create_content_multiple_members(embed_type, members);
        }
    }
    (content, embeds)
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
                    .name(USERNAME.en)
                    .description(USERNAME_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(MEMBER.en)
                            .description(MEMBER_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(DISCRIMINATOR.en)
                    .description(DISCRIMINATOR_DESC.en)
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

    #[instrument(skip(cmd, _handler, context))]
    async fn handle(
        cmd: &ApplicationCommandInteraction,
        _handler: &Handler,
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
        let mut content = String::from("");

        if let Some(response_type) = cmd.data.options.first() {
            for j in &response_type.options {
                if j.name == "members" {
                    selected_users = parse_command_members(j, context, cmd).await
                }
            }
            let response = create_response_members(&response_type.name, &selected_users);
            embeds = response.1;
            content = response.0;
        }

        cmd.create_interaction_response(context, |res| {
            res.interaction_response_data(|d| {
                if !embeds.is_empty() {
                    d.add_embeds(embeds)
                } else {
                    d.content(content)
                }
            })
        })
        .await?;
        Ok(())
    }

    fn name() -> LocalizedString {
        NAME
    }
}
