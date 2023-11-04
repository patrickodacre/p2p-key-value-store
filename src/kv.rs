use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
// use serde_json::Deserializer;

pub use crate::prelude::*;
use std::ffi::OsStr;

#[derive(Debug)]
pub struct KvStore {
    file: File,
    // path: PathBuf,
    // readers: HashMap<u64, BufReaderWithPos<File>>,
    // writer: BufWriterWithPos<File>,
    current_gen: u64,
    values: HashMap<String, String>,
    // index: BTreeMap<String, CommandPos>,
    // the number of bytes representing "stale" commands that could be
    // deleted during a compaction
    uncompacted: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    command: String,
    key: String,
    value: String,
}

impl Entry {
    pub fn new(
        command: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Entry {
            command: command.into(),
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        let Ok(json) = serde_json::to_string(self) else {
            return Err(KvsError::Generic("failed json serialization".to_string()));
        };

        Ok(json)
    }
}

impl KvStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        let Some(db_path_as_str) = db_path.to_str() else {
            return Err(KvsError::Generic("Error opening log.".to_string()));
        };

        let Ok(file) = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(db_path_as_str)
        else {
            return Err(KvsError::Generic("Error opening log.".to_string()));
        };

        // load the log file into memory
        let mut values = HashMap::<String, String>::new();
        let reader = BufReader::new(&file);
        for (index, _line) in reader.lines().enumerate() {
            if let Ok(line) = _line {
                let parts = line.split(":").collect::<Vec<&str>>();
                let Some(command) = parts.get(0) else {
                    return Err(KvsError::Generic(format!("invalid log at {}", index)));
                };

                let Some(key) = parts.get(1) else {
                    return Err(KvsError::Generic(format!("invalid log at {}", index)));
                };

                let Some(value) = parts.get(2) else {
                    return Err(KvsError::Generic(format!("invalid log at {}", index)));
                };

                match *command {
                    "set" => {
                        values.insert(key.to_string(), value.to_string());
                    }
                    "rm" => {
                        values.remove(*key);
                    }
                    _ => {}
                }
            }
        }

        Ok(KvStore {
            file,
            current_gen: 0,
            values,
            uncompacted: 0,
        })
    }

    pub fn entries(&self) {
        for (key, value) in self.values.iter() {
            println!("{} : {}", key, value);
        }
    }

    /// Set a Key-Value pair in the store.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let entry = Entry::new("set", &key, &value);

        let Ok(entry_str) = serde_json::to_string(&entry) else {
            return Err(KvsError::Generic("Failed to serialize entry.".to_string()));
        };

        let Ok(bytes_written) = self.file.write(entry_str.as_bytes()) else {
            return Err(KvsError::Generic("Failed to write entry.".to_string()));
        };

        self.values.insert(key, value);

        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let Some(maybe_value) = self.values.get(&key) else {
            return Ok(None);
        };

        Ok(Some(maybe_value.to_string()))
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let entry = Entry::new("rm", &key, &"".to_string());
        let Ok(entry_str) = serde_json::to_string(&entry) else {
            return Err(KvsError::Generic("Failed to serialize entry.".to_string()));
        };

        let Ok(bytes_written) = self.file.write(entry_str.as_bytes()) else {
            return Err(KvsError::Generic("Failed to write entry.".to_string()));
        };

        self.values.remove(&key);

        Ok(())
    }
}

enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

