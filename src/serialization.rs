use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::*;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct AudioClipData {
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config<T> {
    pub clips: T,
    pub volume: (f32, f32),
    pub device: (u32, u32),
}

pub struct Clips {
    clips: HashMap<usize, AudioClipData>,
}

impl Serialize for Clips {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let iter = self.clips.iter();
        let mut seq = serializer.serialize_seq(Some(iter.len()))?;
        for tuple in iter {
            seq.serialize_element(tuple.1)?;
        }
        seq.end()
    }
}

pub struct Serializer {
    pub writer: BufWriter<File>,
    pub config: Config<Clips>,
}

impl Serializer {
    pub fn new<P: AsRef<Path>>(path: P, volume: (f32, f32), device: (u32, u32)) -> Serializer {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .expect("unable to create/open config file");
        Serializer {
            writer: BufWriter::new(file),
            config: Config {
                clips: Clips {
                    clips: HashMap::new(),
                },
                volume,
                device,
            }
        }
    }

    pub fn save(&mut self) {
        self.writer
            .write(&ron::ser::to_string(&self.config).unwrap().as_bytes())
            .unwrap();
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
                return Deserializer {
                    config
                }
            }
        }
        Deserializer {
            config: Config {
                clips: Vec::new(),
                volume: (0.5, 0.5),
                device: (0, 0),
            }
        }
    }
}
