use std::collections::HashMap;

use nu_ansi_term::{Color, Style};
use nu_data::primitive::get_color_config;
use nu_data::value::format_leaf;
use nu_engine::{CommandArgs, EvaluationContext, Example, UnevaluatedCallInfo, WholeStreamCommand};
use nu_errors::ShellError;
use nu_protocol::hir::{self, Expression, ExternalRedirection, Literal, SpannedExpression};
use nu_protocol::{format_primitive, Primitive, Signature, UntaggedValue, Value};

#[cfg(feature = "dataframe")]
use nu_protocol::dataframe::FrameStruct;
use nu_source::{PrettyDebug, Tag};
use nu_stream::{InputStream, IntoOutputStream};
use nu_table::TextStyle;

pub struct Command;

impl WholeStreamCommand for Command {
    fn name(&self) -> &str {
        "discordview"
    }

    fn signature(&self) -> Signature {
        Signature::build("discordview")
    }

    fn usage(&self) -> &str {
        "View the contents of the pipeline as a table or list (discord bot usage)."
    }

    fn run(&self, args: CommandArgs) -> Result<InputStream, ShellError> {
        discordview(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Automatically view the results",
                example: "ls | discordview",
                result: None,
            },
            Example {
                description: "Autoview is also implied (in the nushell discord bot context). The above can be written as",
                example: "ls",
                result: None,
            },
        ]
    }
}

pub fn discordview(args: CommandArgs) -> Result<InputStream, ShellError> {
    // let binary = args.scope().get_command("binaryview");
    // let text = args.scope().get_command("textview");
    // let table = args.scope().get_command("table");
    // let context = args.context;
    let input_stream = args.input;

    Ok(input_stream
        .map(|x| {
            println!("x: {:#?}", x);

            match x {
                Value {
                    value: UntaggedValue::Primitive(u),
                    tag,
                } => {
                    return format_primitive(&u, None);
                }
                Value {
                    value: UntaggedValue::Row(ref row),
                    tag,
                } => {
                    let term_width = 280;
                    println!("HERE");
                    let mut entries = vec![];
                    for (key, value) in row.entries.iter() {
                        entries.push(vec![
                            nu_table::StyledString::new(
                                key.to_string(),
                                TextStyle::new()
                                    .alignment(nu_table::Alignment::Left)
                                    .fg(nu_ansi_term::Color::Green)
                                    .bold(Some(true)),
                            ),
                            nu_table::StyledString::new(
                                format_leaf(value).plain_string(100_000),
                                nu_table::TextStyle::basic_left(),
                            ),
                        ]);
                    }

                    let color_hm = create_default_colormap();

                    let table = nu_table::Table::new(vec![], entries, nu_table::Theme::compact());

                    let table = format!("{}", nu_table::draw_table(&table, term_width, &color_hm));
                    println!("{}", table);
                    return table;
                }
                // #[cfg(feature = "dataframe")]
                // Value {
                //     value: UntaggedValue::DataFrame(df),
                //     tag,
                // } => {
                //     if let Some(table) = table {
                //         // TODO. Configure the parameter rows from file. It can be
                //         // adjusted to see a certain amount of values in the head
                //         let command_args =
                //             create_default_command_args(&context, df.print()?.into(), tag);
                //         let result = table.run(command_args)?;
                //         let _ = result.collect::<Vec<_>>();
                //     }
                // }
                // #[cfg(feature = "dataframe")]
                // Value {
                //     value: UntaggedValue::FrameStruct(FrameStruct::GroupBy(groupby)),
                //     tag,
                // } => {
                //     if let Some(table) = table {
                //         // TODO. Configure the parameter rows from file. It can be
                //         // adjusted to see a certain amount of values in the head
                //         let command_args =
                //             create_default_command_args(&context, groupby.print()?.into(), tag);
                //         let result = table.run(command_args)?;
                //         let _ = result.collect::<Vec<_>>();
                //     }
                // }
                Value {
                    value: ref item,
                    ref tag,
                } => {
                    return format!("{:?}", item);
                }
            }
        })
        .map(|s| UntaggedValue::Primitive(Primitive::String(s)).into_value(Tag::unknown()))
        .into_output_stream())
}

fn create_default_command_args(
    context: &EvaluationContext,
    input: InputStream,
    tag: Tag,
) -> CommandArgs {
    let span = tag.span;
    CommandArgs {
        context: context.clone(),
        call_info: UnevaluatedCallInfo {
            args: hir::Call {
                head: Box::new(SpannedExpression::new(
                    Expression::Literal(Literal::String(String::new())),
                    span,
                )),
                positional: None,
                named: None,
                span,
                external_redirection: ExternalRedirection::Stdout,
            },
            name_tag: tag,
        },
        input,
    }
}

fn create_default_colormap() -> HashMap<String, Style> {
    // create the hashmap
    let mut hm: HashMap<String, Style> = HashMap::new();
    // set some defaults
    hm.insert("primitive_int".to_string(), Color::White.normal());
    hm.insert("primitive_decimal".to_string(), Color::White.normal());
    hm.insert("primitive_filesize".to_string(), Color::White.normal());
    hm.insert("primitive_string".to_string(), Color::White.normal());
    hm.insert("primitive_line".to_string(), Color::White.normal());
    hm.insert("primitive_columnpath".to_string(), Color::White.normal());
    hm.insert("primitive_pattern".to_string(), Color::White.normal());
    hm.insert("primitive_boolean".to_string(), Color::White.normal());
    hm.insert("primitive_date".to_string(), Color::White.normal());
    hm.insert("primitive_duration".to_string(), Color::White.normal());
    hm.insert("primitive_range".to_string(), Color::White.normal());
    hm.insert("primitive_path".to_string(), Color::White.normal());
    hm.insert("primitive_binary".to_string(), Color::White.normal());
    hm.insert("separator_color".to_string(), Color::White.normal());
    hm.insert("header_align".to_string(), Color::Green.bold());
    hm.insert("header_color".to_string(), Color::Green.bold());
    hm.insert("header_bold".to_string(), Color::Green.bold());
    hm.insert("header_style".to_string(), Style::default());
    hm.insert("index_color".to_string(), Color::Green.bold());
    hm.insert(
        "leading_trailing_space_bg".to_string(),
        Style::default().on(Color::Rgb(128, 128, 128)),
    );

    return hm;
}
