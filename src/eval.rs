use std::time::Duration;

use nu_command::{config::NuConfig, InputStream};
use nu_engine::{run_block, EvaluationContext};
use nu_errors::ShellError;
use nu_parser::ParserScope;
use nu_protocol::{
    format_primitive, hir::ExternalRedirection, Primitive, Type, UntaggedValue, Value,
};
use nu_source::{PrettyDebug, Tag, Tagged, TaggedItem};
use serenity::model::channel::Message;

use crate::context::create_sandboxed_context;

enum Command {
    One(String),
    Block(Vec<String>),
}

fn parse_command<'a>(msg: &'a str) -> Option<Command> {
    match msg.trim().strip_prefix("nu! `") {
        Some(cmd) => {
            // Single line format:
            // nu! `[command]`
            Some(Command::One(format!(
                "{} | discordview",
                cmd.strip_suffix("`")?.to_string(),
            )))
        }
        None => {
            // Block format:
            // nu!
            // ```
            // [commands]
            // ```
            let mut cmds: Vec<String> = msg.trim().split("\n").map(|x| format!("{}", x)).collect();
            if cmds.get(0)?.trim() != "nu!"
                || !cmds.get(1)?.trim().starts_with("```")
                || cmds.last()?.trim() != "```"
            {
                None
            } else {
                cmds.remove(0);
                cmds.remove(0);
                cmds.pop();
                let last = cmds.pop();
                match last {
                    Some(mut s) => {
                        s.push_str(" | discordview");
                        cmds.push(s);
                        Some(Command::Block(cmds))
                    }
                    None => Some(Command::One("\n".to_owned())),
                }
            }
        }
    }
}

// fn convert_to_string(value: &UntaggedValue, ctx: &EvaluationContext) -> String {
//     let to = ctx
//         .scope
//         .get_command("to")
//         .expect("Internal error: expected `md` command");
//     match value {
//         UntaggedValue::Primitive(p) => format_primitive(p, None),
//         UntaggedValue::Row(r) => to.run(),
//         UntaggedValue::Table(t) => "TABLE".to_owned(),
//         UntaggedValue::Error(e) => "<error>".to_owned(),
//         UntaggedValue::Block(b) => "<block>".to_owned(),
//     }
// }

fn parse_and_eval(line: &str, ctx: &EvaluationContext) -> Result<String, ShellError> {
    // FIXME: do we still need this?
    let line = if let Some(s) = line.strip_suffix('\n') {
        s
    } else {
        line
    };

    // TODO ensure the command whose examples we're testing is actually in the pipeline
    ctx.scope.enter_scope();
    let (classified_block, err) = nu_parser::parse(line, 0, &ctx.scope);
    if let Some(err) = err {
        ctx.scope.exit_scope();
        return Err(err.into());
    }

    let input_stream = InputStream::empty();

    let result = run_block(
        &classified_block,
        ctx,
        input_stream,
        ExternalRedirection::Stdout,
    );
    ctx.scope.exit_scope();

    let mut result = result?;

    let mut output = String::from("");
    loop {
        match result.next() {
            Some(v) => {
                // println!("{:?}", v);
                output.push_str(v.expect_string())
            }
            None => break,
        }
    }

    Ok(output)

    // Ok(result?.map(|x| convert_to_string(&x.value)).collect())
}

fn run_cmd(cmd: &str, sandbox: &EvaluationContext) -> String {
    let cmd = format!("{}\n", cmd);

    match parse_and_eval(&cmd, &sandbox) {
        Ok(res) => {
            res
            // format!("> {}{}", cmd, res)
        }
        Err(why) => {
            // println!("> {}{:#?}", cmd, why);
            // why.into_diagnostic()
            format!("{:#?}", why.into_diagnostic())
        }
    }
}

pub enum HandlerError {
    ParseError,
    TimeoutError,
    SandboxError,
}

pub async fn handle_message(msg: &Message) -> Result<String, HandlerError> {
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
                        // the variables don't persist after each run.
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
