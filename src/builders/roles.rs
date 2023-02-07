use serenity::builder::CreateEmbed;
use serenity::model::channel::EmbedField;
use serenity::model::id::RoleId;

pub fn roles_to_field<'b>(roles: &Vec<RoleId>, inline: Option<bool>, embed: &'b mut CreateEmbed) -> &'b mut CreateEmbed {
    let roles: Vec<String> = roles.iter().map(|role| format!("<@&{}>", role.to_string())).collect();
    let field = EmbedField::new(format!("**Roles (``{}``)**", roles.len()), format!("{}", roles.join(" ``|`` ")), inline.unwrap_or(false));
    embed.field(field.name, field.value, field.inline)
}

pub fn roles_to_text(roles: &Vec<RoleId>) -> String {
    let roles: Vec<String> = roles.iter().map(|role| format!("<@&{}>", role.to_string())).collect();
    return roles.join(" | ")
}