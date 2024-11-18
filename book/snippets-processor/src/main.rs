// Copied from https://github.com/breez/breez-sdk-docs/tree/74f018a647820ce515a564083b9986157ee7f894/snippets-processor

use std::io;

use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;

fn main() -> Result<()> {
    // set up app
    let matches = make_app().get_matches();
    let pre = SnippetsProcessor;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        handle_supports(&pre, sub_args)
    } else {
        // handle preprocessing
        handle_preprocessing(&pre)
    }
}

/// Parse CLI options.
pub fn make_app() -> Command {
    Command::new("mdbook-snippets")
        .version(crate_version!())
        .about("A preprocessor that removes leading whitespace from code snippets.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

/// Tell mdBook if we support what it asks for.
fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<()> {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);
    if supported {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "The snippets preprocessor does not support the '{renderer}' renderer",
        )))
    }
}

/// Preprocess `book` using `pre` and print it out.
fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    check_mdbook_version(&ctx.mdbook_version);

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

/// Produce a warning on mdBook version mismatch.
fn check_mdbook_version(version: &str) {
    if version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "This mdbook-snippets was built against mdbook v{}, \
            but we are being called from mdbook v{version}. \
            If you have any issue, this might be a reason.",
            mdbook::MDBOOK_VERSION,
        )
    }
}

struct SnippetsProcessor;
impl Preprocessor for SnippetsProcessor {
    fn name(&self) -> &str {
        "snippets"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let mut resulting_lines: Vec<String> = vec![];
                let mut in_block = false;
                let mut block_lines: Vec<String> = vec![];
                let mut min_indentation: usize = 0;
                for line in chapter.content.lines() {
                    if line.starts_with("```") {
                        if in_block {
                            // This is end of block
                            // Replace previous lines
                            for block_line in block_lines.iter().cloned() {
                                let indent = std::cmp::min(min_indentation, block_line.len());
                                resulting_lines.push(block_line[indent..].to_string())
                            }
                            in_block = false;
                        } else {
                            // Start of block
                            in_block = true;
                            block_lines = vec![];
                            min_indentation = usize::MAX;
                        }

                        resulting_lines.push(line.to_string());
                        continue;
                    }

                    if in_block {
                        let line = line.replace('\t', "    ");
                        block_lines.push(line.clone());
                        let trimmed = line.trim_start_matches(' ');
                        if !trimmed.is_empty() {
                            min_indentation =
                                std::cmp::min(min_indentation, line.len() - trimmed.len())
                        }
                    } else {
                        resulting_lines.push(line.to_string());
                    }
                }

                chapter.content = resulting_lines.join("\n");
            }
        });
        Ok(book)
    }
}
