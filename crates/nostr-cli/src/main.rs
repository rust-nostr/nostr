// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;
use cli::DatabaseCommand;
use nostr_sdk::prelude::*;
use rayon::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tokio::time::Instant;
use tracing_subscriber::fmt::format::FmtSpan;

mod cli;
mod util;

use self::cli::{parser, Cli, CliCommand, Command};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
    }
}

async fn run() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let args = Cli::parse();

    match args.command {
        CliCommand::Open => {
            //let db = RocksDatabase::open("./db/nostr").await?;
            let db = SQLiteDatabase::open("nostr.db").await?;
            let client = ClientBuilder::new().database(db).build();

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
                                if let Err(e) = handle_command(command, &client).await {
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

async fn handle_command(command: Command, client: &Client) -> Result<()> {
    match command {
        Command::Query {
            kind,
            author,
            identifier,
            search,
            limit,
            reverse,
            database,
            print,
        } => {
            let db = client.database();

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

            if let Some(search) = search {
                filter = filter.search(search);
            }

            if let Some(limit) = limit {
                filter = filter.limit(limit);
            }

            if filter.is_empty() {
                eprintln!("Filters empty!");
            } else if database {
                // Query database
                let now = Instant::now();
                let events = db
                    .query(vec![filter], if reverse { Order::Asc } else { Order::Desc })
                    .await?;

                let duration = now.elapsed();
                println!(
                    "{} results in {}",
                    events.len(),
                    if duration.as_secs() == 0 {
                        format!("{:.6} ms", duration.as_secs_f64() * 1000.0)
                    } else {
                        format!("{:.2} sec", duration.as_secs_f64())
                    }
                );
                if print {
                    // Print events
                    util::print_events(events);
                }
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
                    let events: BTreeSet<Event> = reader
                        .lines()
                        .par_bridge()
                        .flatten()
                        .filter_map(|msg| {
                            if let Ok(RelayMessage::Event { event, .. }) =
                                serde_json::from_str(&msg)
                            {
                                Some(*event)
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Bulk load
                    let db = client.database();
                    println!("Indexing {} events", events.len());
                    let now = Instant::now();
                    db.bulk_import(events).await?;
                    println!("Indexed in {:.6} secs", now.elapsed().as_secs_f64());
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
