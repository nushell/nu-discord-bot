// use derive_new::new;
use std::path::PathBuf;
use nu_command::{ActionStream, InputStream};
use nu_engine::{whole_stream_command, EvaluationContext, Example, CommandArgs};
use nu_engine::WholeStreamCommand;
use nu_engine::{evaluate_baseline_expr, shell::CdArgs};
use nu_errors::ShellError;
use nu_protocol::{
    hir::{ExternalArgs, ExternalCommand, SpannedExpression},
    Primitive, UntaggedValue,
};
use nu_protocol::{Signature, SyntaxShape};
use nu_source::Tagged;

// #[derive(new)]
pub struct RunExternalCommand {
}

fn spanned_expression_to_string(
    expr: SpannedExpression,
    ctx: &EvaluationContext,
) -> Result<String, ShellError> {
    let value = evaluate_baseline_expr(&expr, ctx)?;

    if let UntaggedValue::Primitive(Primitive::String(s)) = value.value {
        Ok(s)
    } else {
        Err(ShellError::labeled_error(
            "Expected string for command name",
            "expected string",
            expr.span,
        ))
    }
}

impl WholeStreamCommand for RunExternalCommand {
    fn name(&self) -> &str {
        "run_external"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).rest(SyntaxShape::Any, "external command arguments")
    }

    fn usage(&self) -> &str {
        "Runs external command (not a nushell builtin)"
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Run the external echo command",
            example: "run_external echo 'nushell'",
            result: None,
        }]
    }

    fn is_private(&self) -> bool {
        true
    }

    fn run(&self, _: CommandArgs) -> Result<nu_command::InputStream, ShellError> {
        return Ok(InputStream::one("External commands are not supported yet\n"))
    }

    // fn run_with_actions(&self, args: CommandArgs) -> Result<ActionStream, ShellError> {
    //     return Ok(ActionStream {values: Box::new(["External commands are not supported yet"])});
    //     // let positionals = args.call_info.args.positional.clone().ok_or_else(|| {
    //     //     ShellError::untagged_runtime_error("positional arguments unexpectedly empty")
    //     // })?;

    //     // let mut positionals = positionals.into_iter();

    //     // let external_redirection = args.call_info.args.external_redirection;

    //     // let expr = positionals.next().ok_or_else(|| {
    //     //     ShellError::untagged_runtime_error("run_external called with no arguments")
    //     // })?;

    //     // let name = spanned_expression_to_string(expr, &args.context)?;

    //     // let mut external_context = args.context.clone();

    //     // let command = ExternalCommand {
    //     //     name,
    //     //     name_tag: args.call_info.name_tag.clone(),
    //     //     args: ExternalArgs {
    //     //         list: positionals.collect(),
    //     //         span: args.call_info.args.span,
    //     //     },
    //     // };

    //     // let input = args.input;
    //     // let result = external::run_external_command(
    //     //     command,
    //     //     &mut external_context,
    //     //     input,
    //     //     external_redirection,
    //     // );

    //     // // When externals return, don't let them mess up the ansi escapes
    //     // #[cfg(windows)]
    //     // {
    //     //     let _ = nu_ansi_term::enable_ansi_support();
    //     // }

    //     // Ok(result?.into_action_stream())
    // }
}