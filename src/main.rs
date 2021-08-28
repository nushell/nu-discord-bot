use nu_cli::{create_default_context, parse_and_eval};
use nu_command::InputStream;
use nu_engine::script::process_script;
use nu_engine::{run_block, EvaluationContext};
use nu_errors::ShellError;
mod context;
mod run_external;
use context::create_sandboxed_context;

use dotenv::dotenv;
use nu_parser::ParserScope;
use nu_protocol::hir::ExternalRedirection;
use nu_source::Tag;
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
use std::time::Duration;

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

enum Command {
    One(String),
    Block(Vec<String>),
}

fn parse_command<'a>(msg: &'a str) -> Option<Command> {
    match msg.trim().strip_prefix("nu! `") {
        Some(cmd) => {
            // Single line format:
            // nu! `[command]`
            Some(Command::One(cmd.strip_suffix("`")?.to_string()))
        }
        None => {
            // Block format:
            // nu!
            // ```
            // [commands]
            // ```
            let mut cmds: Vec<String> = msg.trim().split("\n").map(|x| x.to_string()).collect();
            if cmds.get(0)?.trim() != "nu!"
                || !cmds.get(1)?.trim().starts_with("```")
                || cmds.last()?.trim() != "```"
            {
                None
            } else {
                cmds.remove(0);
                cmds.remove(0);
                cmds.pop();
                Some(Command::Block(cmds))
            }
        }
    }
}

fn run_cmd(cmd: &str, sandbox: &EvaluationContext) -> String {
    let cmd = format!("{} | to md --pretty \n", cmd);
    match parse_and_eval(&cmd, &sandbox) {
        Ok(res) => {
            println!("> {}{}", cmd, res);
            res
            // format!("> {}{}", cmd, res)
        }
        Err(why) => {
            println!("> {}{:#?}", cmd, why);
            // why.into_diagnostic()
            format!("{:#?}", why.into_diagnostic())
        }
    }
}

enum HandlerError {
    ParseError,
    TimeoutError,
    SandboxError,
}

async fn handle_message(msg: &Message) -> Result<String, HandlerError> {
    if let Ok(res) = tokio::time::timeout(Duration::new(5, 0), async {
        if let Some(command) = parse_command(&msg.content) {
            match create_sandboxed_context() {
                Ok(sandbox) => match command {
                    Command::One(cmd) => Ok(format!(
                        "```md\n> {} \n{}\n```",
                        cmd,
                        run_cmd(&cmd, &sandbox)
                    )),
                    Command::Block(cmds) => {
                        // Run commands w a semicolon b/w them. If they just run one by one
                        // the variables don't persis after each run.
                        let result = run_cmd(&cmds.join(";"), &sandbox);
                        Ok(format!("```md\n> {} \n{}\n```", cmds.join(";\n> "), result))
                    }
                },
                Err(e) => Err(HandlerError::SandboxError),
            }
        } else {
            Err(HandlerError::ParseError)
        }
    })
    .await
    {
        res
    } else {
        Err(HandlerError::TimeoutError)
    }
}

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

            if let Err(e) = msg.reply(ctx, reply).await {
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

    let framework = StandardFramework::default()
        .configure(|c| c.prefix("nu!").ignore_bots(true))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
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
