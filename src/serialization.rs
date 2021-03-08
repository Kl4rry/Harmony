use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::*;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct AudioClipData {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config<T> {
    pub clips: T,
    pub volume: (f32, f32),
    pub device: (usize, usize),
}

pub struct Clips {
    pub inner: HashMap<usize, AudioClipData>,
}

impl Serialize for Clips {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let iter = self.inner.iter();
        let mut seq = serializer.serialize_seq(Some(iter.len()))?;
        for tuple in iter {
            seq.serialize_element(tuple.1)?;
        }
        seq.end()
    }
}

pub struct Serializer {
    file: Arc<Mutex<File>>,
    pub config: Config<Clips>,
}

impl Serializer {
    pub fn new<P: AsRef<Path>>(path: P, volume: (f32, f32), device: (usize, usize)) -> Serializer {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .expect("unable to create/open config file");
        Serializer {
            file: Arc::new(Mutex::new(file)),
            config: Config {
                clips: Clips {
                    inner: HashMap::new(),
                },
                volume,
                device,
            },
        }
    }

    pub fn save(&mut self) {
        let file = self.file.clone();
        let string = ron::ser::to_string(&self.config).unwrap();
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                let mut guard = file.lock().unwrap();
                guard.set_len(0).expect("unable to clear file");
                guard
                    .seek(SeekFrom::Start(0))
                    .expect("unable to clear file");
                guard.write_all(&string.as_bytes()).expect("unable to save");
                guard.flush().expect("unable to flush file");
                guard.sync_data().expect("unable to sync file data");
            });
        });
    }
}

pub struct Deserializer {
    pub config: Config<Vec<AudioClipData>>,
}

impl Deserializer {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let result = read_to_string(path);
        if let Ok(string) = result {
            if let Ok(config) = ron::de::from_str(&string) {
                return Deserializer { config };
            }
        }
        Deserializer {
            config: Config {
                clips: Vec::new(),
                volume: (0.5, 0.5),
                device: (0, 0),
            },
        }
    }
}
