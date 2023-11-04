use clap::{arg, Arg, ArgAction, Command, Parser};
use kvs::kv::KvStore;
pub use kvs::prelude::*;
use std::path::Path;
use std::{env::current_dir, io::Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

fn main() -> Result<()> {
    let db_path = Path::new("test.log");
    let index_path = Path::new("test_index.log");

    let matches = Command::new("kvs")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("set")
                .about("Set a Key/Value in the KvStore")
                .arg(arg!(<KEY> "The Key to set."))
                .arg(arg!(<VALUE> "The Value to set."))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Get a Value from the KvStore")
                .arg(arg!(<KEY> "The Key to get."))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("list").about("List values in the KvStore."))
        .subcommand(
            Command::new("rm")
                .about("Remove a key / value from the store")
                .arg(arg!(<KEY> "The Key to remove."))
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("list", _set_matches)) => {
            let store = KvStore::open(&db_path)?;

            store.entries();

            Ok(())
        }
        Some(("set", set_matches)) => {
            let key = set_matches.get_one::<String>("KEY").unwrap();
            let value = set_matches.get_one::<String>("VALUE").unwrap();

            let mut store = KvStore::open(&db_path)?;

            store.set(key.to_string(), value.to_string())?;

            Ok(())
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("KEY").unwrap();

            let mut store = KvStore::open(&db_path)?;

            if let Ok(Some(value)) = store.get(key.to_string()) {
                println!("{:?}", value);
            } else {
                println!("NOT FOUND");
            }
            Ok(())
        }
        Some(("rm", matches)) => {
            let key = matches.get_one::<String>("KEY").unwrap();
            let mut store = KvStore::open(&db_path)?;

            store.remove(key.to_string())?;
            Ok(())
        }
        _ => unreachable!(),
    }
}
