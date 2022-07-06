extern crate kvs;

use std::env::current_dir;
use std::process::exit;

use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};
use kvs::KvError;

#[derive(Debug, Subcommand)]
enum Command {
    /// get <KEY>
    #[clap(arg_required_else_help = true)]
    Get { key: String },

    /// set <KEY> <VALUE>
    #[clap(arg_required_else_help = true)]
    Set { key: String, value: String },

    /// rm <KEY>
    #[clap(arg_required_else_help = true)]
    #[clap(name = "rm")]
    Remove { key: String },
}

#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    let args = Cli::parse();
    let mut kv_store = KvStore::open(current_dir().unwrap().as_path()).unwrap();

    match args.command {
        Command::Get { key } => {
            if let Some(value) = kv_store.get(key).unwrap() {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Command::Set { key, value } => {
            if let Err(e) = kv_store.set(key, value) {
                eprintln!("{}", e);
            }
        }
        Command::Remove { key } => {
            if let Err(KvError::KeyNotFound) = kv_store.remove(key) {
                println!("Key not found");
                exit(1);
            }
        }
    }
}