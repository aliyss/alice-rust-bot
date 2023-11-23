use async_trait::async_trait;
use rand::Rng;
use serenity::builder::CreateEmbed;
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

use crate::{
    commands::{option_data::*, AppCmd},
    util::LocalizedString,
    Handler, HandlerError,
};

pub const NAME: LocalizedString = LocalizedString { en: "ml" };
pub const DESC: LocalizedString = LocalizedString {
    en: "Commands for Machine Learning!",
};

pub struct MLCmd;

enum MLPropertyTypes {
    And,
    Or,
    Nand,
    Nor,
    Xor,
}

impl FromStr for MLPropertyTypes {
    type Err = ();
    fn from_str(input: &str) -> Result<MLPropertyTypes, Self::Err> {
        match input {
            "and" => Ok(MLPropertyTypes::And),
            "or" => Ok(MLPropertyTypes::Or),
            "nand" => Ok(MLPropertyTypes::Nand),
            "nor" => Ok(MLPropertyTypes::Nor),
            "xor" => Ok(MLPropertyTypes::Xor),
            _ => Err(()),
        }
    }
}

pub struct Neuron {
    weight1: f64,
    weight2: f64,
    bias: f64,
}

fn get_training_data(train_type: &MLPropertyTypes) -> [[f64; 3]; 4] {
    match train_type {
        MLPropertyTypes::And => [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        ],
        MLPropertyTypes::Or => [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
        ],
        MLPropertyTypes::Nand => [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        MLPropertyTypes::Nor => [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        MLPropertyTypes::Xor => [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    }
}

fn sigmoid(v: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-v))
}

fn forward(neurons: &[Neuron], x1: f64, x2: f64) -> f64 {
    let a = sigmoid(neurons[0].weight1 * x1 + neurons[0].weight2 * x2 + neurons[0].bias);
    let b = sigmoid(neurons[1].weight1 * x1 + neurons[1].weight2 * x2 + neurons[1].bias);
    sigmoid(neurons[2].weight1 * a + neurons[2].weight2 * b + neurons[2].bias)
}

fn cost(neurons: &[Neuron], train_type: &MLPropertyTypes) -> f64 {
    let mut result: f64 = 0.0;
    let train_set = get_training_data(train_type);
    train_set.iter().for_each(|train_set_i| {
        let x1 = train_set_i[0];
        let x2 = train_set_i[1];
        let y = forward(neurons, x1, x2);
        let d = y - (train_set_i[2]);
        result += d * d;
    });
    result / train_set.len() as f64
}

fn finite_diff(neurons: &[Neuron], eps: f64, train_type: &MLPropertyTypes) -> Vec<Neuron> {
    let mut neurons_g: Vec<Neuron> = vec![
        Neuron {
            weight1: 0.0,
            weight2: 0.0,
            bias: 0.0,
        },
        Neuron {
            weight1: 0.0,
            weight2: 0.0,
            bias: 0.0,
        },
        Neuron {
            weight1: 0.0,
            weight2: 0.0,
            bias: 0.0,
        },
    ];

    let c = cost(neurons, train_type);
    let mut new_neurons: Vec<Neuron> = vec![];

    neurons.iter().for_each(|old_neuron| {
        let neuron: Neuron = Neuron {
            weight1: old_neuron.weight1,
            weight2: old_neuron.weight2,
            bias: old_neuron.bias,
        };
        new_neurons.push(neuron)
    });

    let mut saved = 0.0;

    neurons.iter().enumerate().for_each(|(i, old_neuron)| {
        let neuron: Neuron = Neuron {
            weight1: old_neuron.weight1,
            weight2: old_neuron.weight2,
            bias: old_neuron.bias,
        };

        saved = neuron.weight1;
        new_neurons[i].weight1 += eps;
        neurons_g[i].weight1 = (cost(&new_neurons, train_type) - c) / eps;
        new_neurons[i].weight1 = saved;

        saved = neuron.weight2;
        new_neurons[i].weight2 += eps;
        neurons_g[i].weight2 = (cost(&new_neurons, train_type) - c) / eps;
        new_neurons[i].weight2 = saved;

        saved = neuron.bias;
        new_neurons[i].bias += eps;
        neurons_g[i].bias = (cost(&new_neurons, train_type) - c) / eps;
        new_neurons[i].bias = saved;
    });

    neurons_g
}

fn train_and_test(train_type: MLPropertyTypes, embed: &mut CreateEmbed) -> &CreateEmbed {
    let mut rng = rand::thread_rng();

    let mut neurons_m: Vec<Neuron> = vec![
        Neuron {
            weight1: rng.gen::<f64>(),
            weight2: rng.gen::<f64>(),
            bias: rng.gen::<f64>(),
        },
        Neuron {
            weight1: rng.gen::<f64>(),
            weight2: rng.gen::<f64>(),
            bias: rng.gen::<f64>(),
        },
        Neuron {
            weight1: rng.gen::<f64>(),
            weight2: rng.gen::<f64>(),
            bias: rng.gen::<f64>(),
        },
    ];

    let eps = 0.1;
    let rate = 0.1;

    for _n in 0..(300 * 1000) {
        let neurons_g = finite_diff(&neurons_m, eps, &train_type);
        for i in 0..neurons_m.len() {
            neurons_m[i].weight1 -= rate * neurons_g[i].weight1;
            neurons_m[i].weight2 -= rate * neurons_g[i].weight2;
            neurons_m[i].bias -= rate * neurons_g[i].bias;
        }
    }

    embed.description(format!("Cost: {}", cost(&neurons_m, &train_type)));

    let mut s = String::from("");

    for i in 0..2 {
        for j in 0..2 {
            s += format!(
                "{i} | {j} | {:.8}\n",
                forward(&neurons_m, f64::from(i), f64::from(j))
            )
            .as_str()
        }
    }

    embed.field("Output", s, false);

    let mut n1 = String::from("");

    for i in 0..2 {
        for j in 0..2 {
            n1 += format!(
                "{i} | {j} | {:.4}\n",
                sigmoid(
                    neurons_m[0].weight1 * f64::from(i)
                        + neurons_m[0].weight2 * f64::from(j)
                        + neurons_m[0].bias
                )
            )
            .as_str()
        }
    }

    embed.field("Neuron 1", n1, true);

    let mut n2 = String::from("");

    for i in 0..2 {
        for j in 0..2 {
            n2 += format!(
                "{i} | {j} | {:.4}\n",
                sigmoid(
                    neurons_m[1].weight1 * f64::from(i)
                        + neurons_m[1].weight2 * f64::from(j)
                        + neurons_m[1].bias
                )
            )
            .as_str()
        }
    }

    embed.field("Neuron 2", n2, true);

    let mut n3 = String::from("");

    for i in 0..2 {
        for j in 0..2 {
            n3 += format!(
                "{i} | {j} | {:.4}\n",
                sigmoid(
                    neurons_m[2].weight1 * f64::from(i)
                        + neurons_m[2].weight2 * f64::from(j)
                        + neurons_m[2].bias
                )
            )
            .as_str()
        }
    }

    embed.field("Neuron 3", n3, true);

    embed
}

async fn create_field_from_embed_types<'b>(
    embed_types: &Vec<MLPropertyTypes>,
    _params: &[String],
    embed: &'b mut CreateEmbed,
) -> &'b CreateEmbed {
    for i in embed_types {
        match i {
            MLPropertyTypes::And => train_and_test(MLPropertyTypes::And, embed),
            MLPropertyTypes::Or => train_and_test(MLPropertyTypes::Or, embed),
            MLPropertyTypes::Nand => train_and_test(MLPropertyTypes::Nand, embed),
            MLPropertyTypes::Nor => train_and_test(MLPropertyTypes::Nor, embed),
            MLPropertyTypes::Xor => train_and_test(MLPropertyTypes::Xor, embed),
        };
    }
    embed
}

async fn create_embed_single_stock(embed_type: &str, params: &Vec<String>) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(format!("ML ({})", embed_type));
    if let Ok(guild_user_embed_type) = MLPropertyTypes::from_str(embed_type) {
        let embed_types = vec![guild_user_embed_type];
        embed = create_field_from_embed_types(&embed_types, params, &mut embed)
            .await
            .to_owned();
    }
    embed
}

async fn create_response_stocks(
    embed_type: &str,
    stocks: &Vec<String>,
) -> (String, Vec<CreateEmbed>) {
    let mut embeds = vec![];
    let content = String::from("");

    embeds.push(create_embed_single_stock(embed_type, stocks).await);

    (content, embeds)
}

#[async_trait]
impl AppCmd for MLCmd {
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
                    .name(AND.en)
                    .description(AND_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(OR.en)
                    .description(OR_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(NAND.en)
                    .description(NAND_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(NOR.en)
                    .description(NOR_DESC.en)
            })
            .create_option(|opt| {
                opt.kind(CommandOptionType::SubCommand)
                    .name(XOR.en)
                    .description(XOR_DESC.en)
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
        let selected_stocks = Vec::new();

        let mut embeds = vec![];
        let mut content = String::from("");

        if let Some(response_type) = cmd.data.options.first() {
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
