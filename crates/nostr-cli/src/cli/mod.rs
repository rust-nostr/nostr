// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::large_enum_variant)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use nostr_database::nostr::secp256k1::XOnlyPublicKey;
use nostr_database::nostr::Kind;

pub mod parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Open,
}

#[derive(Debug, Parser)]
#[command(name = "")]
pub enum Command {
    /// Query
    Query {
        /// Kind
        #[clap(short, long)]
        kind: Option<Kind>,
        /// Author
        #[clap(short, long)]
        author: Option<XOnlyPublicKey>,
        /// Identifier (`d` tag)
        #[clap(short, long)]
        identifier: Option<String>,
        /// Limit
        #[clap(short, long)]
        limit: Option<usize>,
        /// Ascending order
        #[clap(long)]
        reverse: bool,
        /// Query only database
        #[clap(short, long)]
        database: bool,
        /// Count only
        #[clap(short, long)]
        count: bool,
    },
    /// Database
    #[command(arg_required_else_help = true)]
    Database {
        #[command(subcommand)]
        command: DatabaseCommand,
    },
    /// Developer tools
    Dev {},
    /// Exit
    Exit,
}

#[derive(Debug, Subcommand)]
pub enum DatabaseCommand {
    /// Populate database
    #[command(arg_required_else_help = true)]
    Populate {
        /// Path of JSON file
        path: PathBuf,
    },
    /// Database stats
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum DevCommands {}
