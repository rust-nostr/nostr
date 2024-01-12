// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;
use cli::DatabaseCommand;
use nostr_database::nostr::{Event, Filter, RelayMessage, Result};
use nostr_database::{DatabaseIndexes, Order};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tokio::time::Instant;

mod cli;

use self::cli::{parser, Cli, CliCommand, Command};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
    }
}

async fn run() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        CliCommand::Open => {
            let mut db = DatabaseIndexes::new();

            let rl = &mut DefaultEditor::new()?;

            loop {
                let readline = rl.readline("nostr> ");
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str())?;
                        let mut vec: Vec<String> = parser::split(&line)?;
                        vec.insert(0, String::new());
                        match Command::try_parse_from(vec) {
                            Ok(command) => {
                                if let Err(e) = handle_command(command, &mut db).await {
                                    eprintln!("Error: {e}");
                                }
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                        continue;
                    }
                    Err(ReadlineError::Interrupted) => {
                        // Ctrl-C
                        continue;
                    }
                    Err(ReadlineError::Eof) => break,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }

            Ok(())
        }
    }
}

async fn handle_command(command: Command, db: &mut DatabaseIndexes) -> Result<()> {
    match command {
        Command::Query {
            kind,
            author,
            identifier,
            limit,
            reverse,
            database,
        } => {
            let mut filter = Filter::new();

            if let Some(kind) = kind {
                filter = filter.kind(kind);
            }

            if let Some(author) = author {
                filter = filter.author(author);
            }

            if let Some(identifier) = identifier {
                filter = filter.identifier(identifier);
            }

            if let Some(limit) = limit {
                filter = filter.limit(limit);
            }

            if filter.is_empty() {
                eprintln!("Filters empty!");
            } else if database {
                // Query database
                let now = Instant::now();
                let ids = db
                    .query([filter], if reverse { Order::Asc } else { Order::Desc })
                    .await;
                let duration = now.elapsed();
                println!(
                    "{} results in {}",
                    ids.len(),
                    if duration.as_secs() == 0 {
                        format!("{:.6} ms", duration.as_secs_f64() * 1000.0)
                    } else {
                        format!("{:.2} sec", duration.as_secs_f64())
                    }
                );
            } else {
                // Query relays
            }

            Ok(())
        }
        Command::Database { command } => match command {
            DatabaseCommand::Populate { path } => {
                if path.exists() && path.is_file() {
                    // Open JSON file
                    let file = File::open(path)?;

                    let metadata = file.metadata()?;
                    let reader = BufReader::new(file);

                    println!("File size: {} bytes", metadata.len());

                    // Deserialize events
                    let mut events: BTreeSet<Event> = BTreeSet::new();

                    for line in reader.lines() {
                        match line {
                            Ok(line_content) => {
                                if let Ok(RelayMessage::Event { event, .. }) =
                                    serde_json::from_str(&line_content)
                                {
                                    events.insert(*event);
                                }
                            }
                            Err(e) => eprintln!("Error reading line: {}", e),
                        }
                    }

                    // Bulk load
                    println!("Indexing {} events", events.len());
                    db.bulk_index(events).await;
                } else {
                    println!("File not found")
                }

                Ok(())
            }
            DatabaseCommand::Stats => {
                println!("TODO");
                Ok(())
            }
        },
        Command::Dev {} => Ok(()),
        Command::Exit => std::process::exit(0x01),
    }
}
