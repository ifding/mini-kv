extern crate kvs;
extern crate assert_cmd;
extern crate tempfile;
extern crate predicates;
extern crate walkdir;

use std::process::Command;

use assert_cmd::prelude::*;
use assert_cmd::cargo::CommandCargoExt;
use predicates::ord::eq;
use predicates::str::{contains, is_empty, PredicateStrExt};
use tempfile::TempDir;
use walkdir::WalkDir;

use kvs::{KvStore, Result};

// `kvs` with no args should exit with a non-zero code.
#[test]
fn cli_no_args() {
    Command::cargo_bin("kvs")
        .unwrap()
        .assert()
        .failure()
        .stderr(contains("USAGE"));
}


// `kvs get <KEY>` should print "Key not found" for a non-existent key and exit with zero.
#[test]
fn cli_get_non_existent_key() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["get", "key"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("Key not found").trim());
}

// `kvs rm <KEY>` should print "Key not found" for an empty database and exit with non-zero.
#[test]
fn cli_rm_non_existent_key() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["rm", "key"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(eq("Key not found").trim());
}

// `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
#[test]
fn cli_set() {
    let temp_dir = TempDir::new().expect("unable to create temporary directory");
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["set", "key1", "value1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());
}

#[test]
fn cli_get_stored() {
    let temp_dir = TempDir::new().unwrap();

    let mut store = KvStore::open(temp_dir.path()).unwrap();
    store.set("key1".to_owned(), "value1".to_owned()).unwrap();
    store.set("key2".to_owned(), "value2".to_owned()).unwrap();
    drop(store);

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value1").trim());

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["get", "key2"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value2").trim());
}

#[test]
fn cli_invalid_get() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["get"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["get", "key", "value"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));
}

#[test]
fn cli_invalid_set() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["set"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["set", "key"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["set", "key", "value", "extra"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));
}

#[test]
fn cli_invalid_rm() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["rm"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["rm", "key", "value"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));
}

#[test]
fn cli_invalid_subcommand() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["invalid"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(contains("USAGE"));
}

#[test]
fn test() {


}