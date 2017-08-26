#![deny(rust_2018_idioms, deprecated)]

mod commands;

use std::collections::HashSet;

use commands::*;
use serde::Deserialize;
use serenity::async_trait;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::macros::*;
use serenity::framework::standard::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(Activity::playing("charades")).await;
    }
}

#[group("General")]
#[commands(ping, avatar_url)]
struct General;

#[help]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn dispatch_error(_: &Context, _: &Message, error: DispatchError) {
    error!("dispatch error: {:?}", error);
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command: &str, error: CommandResult) {
    info!("Command `{}` was used by {}", command, msg.author.name);

    if let Err(err) = error {
        let err = err.to_string();
        if err.starts_with("user : ") {
            let without_user = &err["user: ".len()..];
            let _ = msg.channel_id.say(&ctx.http, without_user).await;
        } else {
            error!("`{:?}`", err);
        }
    }
}

#[derive(Deserialize)]
struct Config {
    token: String,
    prefix: String,
}

async fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = std::env::var("KITTY_CONFIG").unwrap_or_else(|_| "config.json".to_string());
    let content = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&content)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = read_config().await?;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&config.prefix))
        .on_dispatch_error(dispatch_error)
        .after(after)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let mut client =
        Client::builder(&config.token).event_handler(Handler).framework(framework).await?;

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shard_manager.lock().await.shutdown_all().await;
    });

    client.start_autosharded().await?;

    Ok(())
}
