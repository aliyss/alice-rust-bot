use async_trait::async_trait;
use serenity::builder::CreateEmbed;
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
use yahoo_finance_api as yahoo;

use crate::handler::command_details::parse_command_array;
use crate::{
    commands::{option_data::*, AppCmd},
    util::LocalizedString,
    Handler, HandlerError,
};

use ascii_table::AsciiTable;

pub const NAME: LocalizedString = LocalizedString { en: "fear" };
pub const DESC: LocalizedString = LocalizedString {
    en: "Commands accessing the fear-index!",
};

pub struct StockCmd;

enum StockPropertyTypes {
    History,
}

impl FromStr for StockPropertyTypes {
    type Err = ();

    fn from_str(input: &str) -> Result<StockPropertyTypes, Self::Err> {
        match input {
            "history" => Ok(StockPropertyTypes::History),
            _ => Err(()),
        }
    }
}

async fn create_field_from_embed_types<'b>(
    embed_types: &Vec<StockPropertyTypes>,
    stock: &str,
    embed: &'b mut CreateEmbed,
) -> &'b CreateEmbed {
    for i in embed_types {
        match i {
            StockPropertyTypes::History => {
                let provider = yahoo::YahooConnector::new();
                match provider.get_latest_quotes(stock, "1d").await {
                    Ok(quotes) => {
                        embed.description(format!(
                            "{} ({})",
                            quotes.metadata().unwrap().exchange_name,
                            "1d"
                        ));
                        embed.field(
                            "High / Low",
                            format!(
                                "{} / {}",
                                quotes.last_quote().unwrap().high.round(),
                                quotes.last_quote().unwrap().low.round()
                            ),
                            false,
                        );
                        embed.field(
                            "Open / Close",
                            format!(
                                "{} / {}",
                                quotes.last_quote().unwrap().open.round(),
                                quotes.last_quote().unwrap().close.round()
                            ),
                            false,
                        );
                    }
                    Err(err) => {
                        embed.description(err.to_string());
                    }
                }
            }
        };
    }
    embed
}

fn create_table_from_embed_types(
    embed_types: &Vec<StockPropertyTypes>,
    stocks: &[String],
) -> String {
    let mut ascii_table = AsciiTable::default();
    ascii_table.set_max_width(120);

    let mut data: Vec<Vec<String>> = vec![];

    let mut m = 0;

    stocks.iter().for_each(|stock| {
        data.push(vec![]);
        ascii_table.column(0);
        data[m].push(stock.to_string());
        for i in embed_types {
            match i {
                StockPropertyTypes::History => {
                    data[m].push(stock.to_string());
                }
            };
        }
        m += 1
    });

    let text = ascii_table.format(&data);
    String::from("```\n") + &text + &*String::from("\n```")
}

async fn create_embed_single_stock(embed_type: &str, stock: &String) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(stock.to_string());
    match embed_type {
        "info" => {
            let embed_types = vec![StockPropertyTypes::History];
            embed = create_field_from_embed_types(&embed_types, stock, &mut embed)
                .await
                .to_owned();
        }
        value => {
            if let Ok(guild_user_embed_type) = StockPropertyTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                embed = create_field_from_embed_types(&embed_types, stock, &mut embed)
                    .await
                    .to_owned();
            }
        }
    }
    embed
}

async fn create_content_multiple_stocks(embed_type: &str, stocks: &[String]) -> String {
    match embed_type {
        "info" => {
            let embed_types = vec![StockPropertyTypes::History];
            create_table_from_embed_types(&embed_types, stocks).to_owned()
        }
        value => {
            if let Ok(guild_user_embed_type) = StockPropertyTypes::from_str(value) {
                let embed_types = vec![guild_user_embed_type];
                create_table_from_embed_types(&embed_types, stocks).to_owned()
            } else {
                String::from("")
            }
        }
    }
}

async fn create_response_stocks(
    embed_type: &str,
    stocks: &Vec<String>,
) -> (String, Vec<CreateEmbed>) {
    let mut embeds = vec![];
    let mut content = String::from("");

    match stocks.len() {
        0 => {
            let mut embed = CreateEmbed::default();
            embed
                .title("Error".to_string())
                .description("No stock found!")
                .color(Color::RED);
            embeds.push(embed)
        }
        1 => {
            embeds.push(create_embed_single_stock(embed_type, stocks.first().unwrap()).await);
        }
        _ => {
            content = create_content_multiple_stocks(embed_type, stocks).await;
        }
    }
    (content, embeds)
}

#[async_trait]
impl AppCmd for StockCmd {
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
                            .name(STOCK.en)
                            .description(STOCK_DESC.en)
                    })
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(HISTORY.en)
                    .description(HISTORY_DESC.en)
                    .create_sub_option(|sub| {
                        sub.kind(CommandOptionType::String)
                            .name(STOCK.en)
                            .description(STOCK_DESC.en)
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
        let mut selected_stocks = Vec::new();

        let mut embeds = vec![];
        let mut content = String::from("");

        if let Some(response_type) = cmd.data.options.first() {
            for j in &response_type.options {
                if j.name == "stock" {
                    selected_stocks = parse_command_array(j, context, cmd)
                }
            }
            let response = create_response_stocks(&response_type.name, &selected_stocks).await;
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
