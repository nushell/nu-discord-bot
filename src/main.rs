use eval::{handle_message, HandlerError};
mod context;
mod run_external;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::prelude::Ready;
use std::env;
use std::error::Error;

mod discordview;
mod eval;

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "A simple test bot").await?;

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[group]
#[commands(about, ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("nu!") {
            let reply = match handle_message(&msg).await {
                Ok(res) => res,
                Err(HandlerError::ParseError) => "Improper formatting. Format as either \"nu! `[command]`\" or \"nu!\" followed by a code block.".to_string(),
                Err(HandlerError::SandboxError) => "Could not create a sandbox. This is a bug.".to_string(),
                Err(HandlerError::TimeoutError) => "Timeout on command (5s).".to_string()
            };

            if let Err(e) = msg.reply(&ctx, reply).await {
                let message = format!("Error when replying to message: {}", e);
                // Try to reply with the error message.
                if let Err(e) = msg.reply(ctx, message).await {
                    println!("Error when replying to message: {}", e);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let framework = StandardFramework::default()
        .configure(|c| c.prefix("nu!").ignore_bots(true))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN_LOCAL").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
