use serenity::client::Context;
use serenity::http::CacheHttp;
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::guild::Member;

pub async fn parse_command_members(option: &CommandDataOption, context: &Context, cmd: &ApplicationCommandInteraction) -> Vec<Member> {
    let mut selected_users = Vec::new();

    let mut string_value: String = String::from("");
    if let Some(option_value) = option.value.clone() {
        string_value = option_value.as_str().unwrap_or("").to_lowercase()
    }

    let mut guild_members = vec![];
    if let Some(guild_id) = cmd.guild_id {
        guild_members = match context.http.get_guild_members(u64::from(guild_id), None, None).await {
            Ok(members) => members,
            Err(..) => Vec::new()
        };
    }

    for i in guild_members {
        let nickname = i.nick.clone().unwrap_or(String::from(""));
        if i.user.id.to_string() == string_value {
            selected_users.push(i)
        } else if i.user.name.to_lowercase().contains(&string_value) {
            selected_users.push(i)
        } else if nickname.to_lowercase().contains(&string_value) {
            selected_users.push(i)
        }
    }
    selected_users
}