mod context;
use context::create_sandboxed_context;

use dotenv::dotenv;
use nu_engine::eval_expression_with_input;
use nu_parser::{parse, ParseError};
use nu_protocol::ast::{Block, Call};
use nu_protocol::engine::{EngineState, Stack, StateWorkingSet};
use nu_protocol::{PipelineData, ShellError, Span, Value};
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::prelude::Ready;
use serenity::prelude::GatewayIntents;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{env, thread};

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "A simple test bot").await?;

    Ok(())
}

#[group]
#[commands(about)]
struct General;

struct Handler;

#[derive(Debug)]
enum HandlerError {
    FormatError,
    ParseError(ParseError),
    ShellError(ShellError),
    TimeoutError,
}

fn parse_single_message<'a>(msg: &'a str) -> Result<&'a str, HandlerError> {
    let msg = msg.trim();

    if let Some(msg_content) = msg
        .strip_prefix("nu! `")
        .and_then(|msg| msg.strip_suffix("`"))
    {
        return Ok(msg_content);
    }

    return Err(HandlerError::FormatError);
}

fn parse_block_message<'a>(msg: &'a str) -> Result<&'a str, HandlerError> {
    let msg = msg.trim();

    if let Some(msg) = msg
        .strip_prefix("nu!\n```")
        .and_then(|msg| msg.strip_suffix("```"))
    {
        return Ok(msg);
    }

    return Err(HandlerError::FormatError);
}

fn parse_message<'a>(msg: &'a str) -> Result<&'a str, HandlerError> {
    parse_single_message(msg).or(parse_block_message(msg))
}

fn parse_command<'a>(
    engine_state: &mut EngineState,
    stack: &mut Stack,
    source: &'a [u8],
) -> Result<Block, HandlerError> {
    let mut working_set = StateWorkingSet::new(engine_state);

    let (output, err) = parse(
        &mut working_set,
        Some("entry #0"), // format!("entry #{}", entry_num)
        source,
        false,
        &[],
    );

    let cwd = PathBuf::from("/");

    let delta = working_set.render();

    engine_state
        .merge_delta(delta, Some(stack), &cwd)
        .map_err(HandlerError::ShellError)?;

    if let Some(err) = err {
        return Err(HandlerError::ParseError(err));
    }

    Ok(output)
}

fn eval_block(
    engine_state: &EngineState,
    stack: &mut Stack,
    block: &Block,
) -> Result<String, ShellError> {
    let mut input = PipelineData::new(Span { start: 0, end: 0 });
    let mut result = "".to_string();

    for pipeline in block.pipelines.iter() {
        for elem in pipeline.expressions.iter() {
            input = eval_expression_with_input(engine_state, stack, elem, input, false, false)?
        }

        match input {
            PipelineData::Value(Value::Nothing { .. }, ..) => {}
            PipelineData::ExternalStream {
                ref mut exit_code, ..
            } => {
                let exit_code = exit_code.take();

                // Drain the input to the screen via tabular output
                let config = engine_state.get_config();

                match engine_state.find_decl("table".as_bytes(), &[]) {
                    Some(decl_id) => {
                        let table = engine_state.get_decl(decl_id).run(
                            engine_state,
                            stack,
                            &Call::new(Span::new(0, 0)),
                            input,
                        )?;

                        for item in table {
                            if let Value::Error { error } = item {
                                return Err(error);
                            }

                            result.push_str(&item.into_string("\n", config));
                            result.push_str("\n");
                        }
                    }
                    None => {
                        for item in input {
                            if let Value::Error { error } = item {
                                return Err(error);
                            }

                            result.push_str(&item.into_string("\n", config));
                            result.push_str("\n");
                        }
                    }
                };

                if let Some(exit_code) = exit_code {
                    let mut v: Vec<_> = exit_code.collect();

                    if let Some(v) = v.pop() {
                        stack.add_env_var("LAST_EXIT_CODE".into(), v);
                    }
                }
            }
            _ => {
                // Drain the input to the screen via tabular output
                let config = engine_state.get_config();

                match engine_state.find_decl("table".as_bytes(), &[]) {
                    Some(decl_id) => {
                        let table = engine_state.get_decl(decl_id).run(
                            engine_state,
                            stack,
                            &Call::new(Span::new(0, 0)),
                            input,
                        )?;

                        for item in table {
                            if let Value::Error { error } = item {
                                return Err(error);
                            }

                            result.push_str(&item.into_string("\n", config));
                            result.push_str("\n");
                        }
                    }
                    None => {
                        for item in input {
                            if let Value::Error { error } = item {
                                return Err(error);
                            }

                            result.push_str(&item.into_string("\n", config));
                            result.push_str("\n");
                        }
                    }
                };
            }
        }

        input = PipelineData::new(Span { start: 0, end: 0 })
    }

    Ok(result)
}

fn handle_message(content: String) -> Result<String, HandlerError> {
    let source = parse_message(&content)?.as_bytes();

    let mut sandbox = create_sandboxed_context();
    let mut stack = Stack::new();

    let block = parse_command(&mut sandbox, &mut stack, source)?;
    let out = eval_block(&mut sandbox, &mut stack, &block).map_err(HandlerError::ShellError);

    out
}

fn try_handle_message(content: &str) -> Result<String, HandlerError> {
    let (sender, receiver) = mpsc::channel();

    let cloned_content = content.to_string();

    let message_handling_thread =
        thread::spawn(move || sender.send(handle_message(cloned_content)));

    match receiver.recv_timeout(std::time::Duration::new(1000, 0)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            drop(receiver);
            drop(message_handling_thread);
            // took more than 5 seconds
            Err(HandlerError::TimeoutError)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        let result = try_handle_message(&"nu! `3 + 4`");

        match result {
            Ok(result) => assert_eq!(result, "7\n".to_owned()),
            Err(error) => panic!("{:?}", error),
        }
    }

    #[test]
    fn parse_add() {
        let result = parse_message("nu! `3 + 4`");
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), "3 + 4");
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("nu!") {
            let reply = match try_handle_message(&msg.content) {
                Ok(res) => match res.is_empty() {
                  true => format!("```\n*Empty*\n```"),
                  false => format!("```\n{}\n```", res)
                }

                Err(HandlerError::FormatError) => "Improper formatting. Format as either \"nu! `[command]`\" or \"nu!\" followed by a code block.".to_string(),
                Err(HandlerError::ParseError(parse_error)) => format!("ParseError: {}", parse_error.to_string()),
                Err(HandlerError::ShellError(shell_error)) => format!("ShellError: {}", shell_error.to_string()),
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

    let gateway_intents = GatewayIntents::GUILD_MESSAGES.union(GatewayIntents::MESSAGE_CONTENT);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable expected.");
    let mut client = Client::builder(token, gateway_intents)
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
