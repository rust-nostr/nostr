// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::large_enum_variant)]

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use nostr_sdk::prelude::*;

pub mod io;
pub mod parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Open nostr shell
    Shell {
        #[clap(long)]
        relays: Vec<RelayUrl>,
        // tor: bool,
        // proxy: Option<SocketAddr>,
    },
    /// Serve a local relay for test purpose
    Serve {
        /// Port
        #[clap(long)]
        port: Option<u16>,
    },
    /// Serve Nostr Connect signer
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    Bunker,
}

#[derive(Debug, Parser)]
#[command(name = "")]
pub enum ShellCommand {
    /// Generate random keys
    Generate,
    /// Sync public key's event with specified relays (negentropy)
    #[command(arg_required_else_help = true)]
    Sync {
        /// Public key
        public_key: PublicKey,
        /// Relays
        #[clap(long)]
        relays: Vec<RelayUrl>,
        /// Direction
        #[clap(short, long, value_enum, default_value_t = ShellSyncDirection::Down)]
        direction: ShellSyncDirection,
    },
    /// Query
    Query {
        /// Event ID
        #[clap(long)]
        id: Option<EventId>,
        /// Author
        #[clap(short, long)]
        author: Option<PublicKey>,
        /// Kind
        #[clap(short, long)]
        kind: Option<Kind>,
        /// Identifier (`d` tag)
        #[clap(long)]
        identifier: Option<String>,
        /// Full-text search
        #[clap(long)]
        search: Option<String>,
        /// Since
        #[clap(short, long)]
        since: Option<Timestamp>,
        /// Until
        #[clap(short, long)]
        until: Option<Timestamp>,
        /// Limit
        #[clap(short, long)]
        limit: Option<usize>,
        /// Query only database
        #[clap(long)]
        database: bool,
        /// Print result
        #[clap(long)]
        print: bool,
        /// Print result as JSON (require `print` flag!)
        #[clap(long)]
        json: bool,
    },
    /// Database
    #[command(arg_required_else_help = true)]
    Database {
        #[command(subcommand)]
        command: ShellCommandDatabase,
    },
    /// Exit
    Exit,
}

#[derive(Debug, Subcommand)]
pub enum ShellCommandDatabase {
    /// Populate database
    #[command(arg_required_else_help = true)]
    Populate {
        /// Path of JSON file
        path: PathBuf,
    },
    /// Database stats
    Stats,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ShellSyncDirection {
    /// Send events to relay
    Up,
    /// Get events from relay
    Down,
    /// Both send and get events from relay (bidirectional sync)
    Both,
}

impl From<ShellSyncDirection> for SyncDirection {
    fn from(value: ShellSyncDirection) -> Self {
        match value {
            ShellSyncDirection::Up => Self::Up,
            ShellSyncDirection::Down => Self::Down,
            ShellSyncDirection::Both => Self::Both,
        }
    }
}
