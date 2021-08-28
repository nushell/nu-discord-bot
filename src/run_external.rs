// use derive_new::new;
use nu_command::InputStream;
use nu_engine::WholeStreamCommand;
use nu_engine::{CommandArgs, Example};
use nu_errors::ShellError;
use nu_protocol::{Signature, SyntaxShape};

// #[derive(new)]
pub struct RunExternalCommand {}

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
        return Ok(InputStream::one(
            "External commands are not supported yet\n",
        ));
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
