use nu_engine::EvaluationContext;
use nu_errors::ShellError;
use nu_cli::{create_default_context, parse_and_eval};
mod context;
mod run_external;
use context::create_sandboxed_context;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{

        command,
        group
    }
};
use dotenv::dotenv;
use serenity::model::prelude::Ready;
use std::env;
use std::error::Error;

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

struct Handler {
    sandbox: EvaluationContext
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(command) = msg.content.trim().strip_prefix("nu! ") {
            println!("Received command: {}", command);
            let result = match parse_and_eval(command, &self.sandbox) {
                Ok(res) => {
                    println!("Response: {}", res);
                    format!("```sh\n{}```", res)
                },
                Err(why) => {
                    println!("Response: {:#?}", why);
                    format!("```sh\n{:#?}```", why.into_diagnostic())
                }
            };

            if let Err(e) = msg.reply(ctx, result).await {
                println!("Error when replying to message: {}", e);
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

    let sandbox = create_sandboxed_context()?;
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("nu! ")) 
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler {sandbox})
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}