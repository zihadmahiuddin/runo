mod commands;
use commands::{uno::button::UnoButton, *};

use poise::serenity_prelude::{self as serenity, ChannelId, UserId};
use runo::{error::UnoError, uno::Uno};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env::var,
};
use tokio::sync::Mutex;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub enum UnoGame {
    Pending {
        channel_id: ChannelId,
        host: UserId,
        queued_users: BTreeMap<UserId, String>,
    },
    Ongoing {
        channel_id: ChannelId,
        host: UserId,
        game: Uno,
    },
}

impl UnoGame {
    pub fn get_player_ids(&self) -> HashSet<UserId> {
        match self {
            UnoGame::Pending { queued_users, .. } => queued_users.keys().cloned().collect(),
            UnoGame::Ongoing { game, .. } => {
                game.get_player_ids().into_iter().map(UserId).collect()
            }
        }
    }

    pub fn into_ongoing(&mut self) -> Result<(), UnoError> {
        match self {
            UnoGame::Pending {
                channel_id,
                queued_users,
                host,
            } => {
                match Uno::new_with_ids(
                    queued_users
                        .iter_mut()
                        .map(|(k, v)| (k.0, v.clone()))
                        .collect(),
                ) {
                    Ok(game) => {
                        *self = UnoGame::Ongoing {
                            channel_id: *channel_id,
                            host: *host,
                            game,
                        };
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            UnoGame::Ongoing { .. } => Ok(()),
        }
    }
}

// Custom user data passed to all command functions
pub struct Data {
    matches: Mutex<HashMap<ChannelId, UnoGame>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}")
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: vec![
            autocomplete::greet(),
            general::help(),
            uno::uno(),
            owner::register(),
        ],
        prefix_options: poise::PrefixFrameworkOptions::default(),
        /// The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        /// This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        /// Every command invocation must pass this check to continue execution
        command_check: None,
        /// Enforce command checks even for owners (enforced by default)
        /// Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, _framework, data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.name());

                UnoButton::handle_event(ctx, event, data).await;

                Ok(())
            })
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(
            var("DISCORD_TOKEN")
                .expect("Missing `DISCORD_TOKEN` env var, see README for more information."),
        )
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    matches: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .intents(serenity::GatewayIntents::non_privileged())
        .run()
        .await
        .unwrap();
}
