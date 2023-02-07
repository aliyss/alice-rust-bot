mod commands;
mod builders;
pub mod handler;
pub mod util;

use std::time::{SystemTime};
use commands::CommandsEnum;
use handler::{Handler, HandlerError};
use tracing::*;

use serenity::{
    async_trait,
    model::prelude::{
        interaction::{application_command::ApplicationCommandInteraction, Interaction}, Message, Ready,
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

use serenity::model::gateway::Activity;
use serenity::prelude::TypeMapKey;
use tokio::{try_join};

#[derive(Clone, Debug)]
struct LastChanged(SystemTime);

impl TypeMapKey for LastChanged {
    type Value = LastChanged;
}

use crate::commands::{guild::GuildCommands};

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, _context))]
    async fn message(&self, _context: Context, _msg: Message) {
        info!("handling message");
    }

    #[instrument(skip(self, context))]
    async fn ready(&self, context: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);

        info!(
            guilds = ?ready.guilds.iter().map(|ug| ug.id).collect::<Vec<_>>()
        );

        if let Err(err) = try_join!(
            self.setup_guild_commands(&context, ready),
        ) {
            error!(?err, "could not setup application commands, shutting down");
            context.shard.shutdown_clean();
            return;
        }
    }

    #[instrument(skip(self, context))]
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {


            let last_changed = (context.data.read().await).get::<LastChanged>().cloned();
            let now = SystemTime::now();
            if let Some(last_changed) = last_changed {
                let elapsed = now.duration_since(last_changed.0).unwrap();
                if elapsed.as_secs() > 20 {
                    context.set_activity(Activity::watching(&cmd.member.as_ref().unwrap().user.name)).await;
                    let mut data = context.data.write().await;
                    data.insert::<LastChanged>(LastChanged(now));
                }
            } else {
                context.set_activity(Activity::watching(&cmd.member.as_ref().unwrap().user.name)).await;
                let mut data = context.data.write().await;
                data.insert::<LastChanged>(LastChanged(now));
            }


            let handle_res = match self
                .try_handle_commands::<GuildCommands>(&context, &cmd)
                .await
            {
                Some(r) => r,
                None => Err(HandlerError::UnrecognizedCommand(cmd.data.name.to_string())),
            };

            if let Err(err) = handle_res {
                error!(?err, "error during interaction processing");
                if err.should_followup() {
                    if let Err(e) = cmd
                        .create_followup_message(&context, |msg| {
                            msg.ephemeral(true).content(err.to_string())
                        })
                        .await
                    {
                        error!(
                            err = ?e,
                            "could not send follow-up message",
                        );
                    }
                }
            };
        }
    }
}

impl Handler {
    #[instrument(skip_all)]
    async fn try_handle_commands<'a, T>(
        &self,
        context: &Context,
        cmd: &ApplicationCommandInteraction,
    ) -> Option<Result<(), HandlerError>>
        where
            T: CommandsEnum,
    {
        let read = context.data.read().await;
        if let Some(cmd_map) = read.get::<T>() {
            if let Some(app_cmd) = cmd_map.get(&cmd.data.id) {
                trace!(?app_cmd, "handing off to app command handler");
                Some(app_cmd.handle(cmd, self, context).await)
            } else {
                None
            }
        } else {
            Some(Err(HandlerError::TypeMapNotFound))
        }
    }
}

pub async fn setup_client(token: String) -> Client {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES;

    let handler = Handler::new().expect("couldn't load log message data from xivapi");

    Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("error creating client")
}