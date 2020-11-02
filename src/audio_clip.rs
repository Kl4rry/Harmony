use std::fs::File;
use rodio::{Sink, DeviceTrait, Device};
use std::path::PathBuf;
use std::path::Path;
use std::boxed::Box;
use rodio::source::Source;
use std::time::Duration;
use std::io::BufReader;
use rodio::source::Buffered;
use rodio::buffer::SamplesBuffer;
use std::thread;
use std::process::Command;

use std::error;
use std::fmt;

static mut DEVICES: Option<Box<(Device, Device)>> = None;

#[derive(Debug)]
pub enum LoadError {
    FormatError(rodio::decoder::DecoderError),
    FileError(std::io::Error),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::FileError(ref e) => e.fmt(f),
            LoadError::FormatError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for LoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            LoadError::FileError(ref e) => Some(e),
            LoadError::FormatError(ref e) => Some(e),
        }
    }
}

impl From<rodio::decoder::DecoderError> for LoadError {
    fn from(err: rodio::decoder::DecoderError) -> LoadError {
        LoadError::FormatError(err)
    }
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> LoadError {
        LoadError::FileError(err)
    }
}


pub struct AudioClip {
    pub path: PathBuf,
    sinks: (Sink, Sink),
    buffer: Buffered<SamplesBuffer<u16>>,
    duration: Duration,
}

 //this is a stupid workaround because SamplesBuffer takes a type which implements Into<vec<T>> not a normal vector
struct IntoVec<T>(Vec<T>);
impl<T> From<IntoVec<T>> for Vec<T> {
    fn from(w: IntoVec<T>) -> Vec<T> {
        w.0
    }
}

impl AudioClip {
    //https://doc.rust-lang.org/stable/rust-by-example/error/multiple_error_types/define_error_type.html
    pub fn new(path: PathBuf) -> Result<AudioClip, LoadError> {
        thread::spawn(move||{
            let file = File::open(path.clone())?;
            let decoder = rodio::Decoder::new(BufReader::new(file))?;

            let channels = decoder.channels();
            let sample_rate = decoder.sample_rate();
        
            let samples: Vec<u16> = decoder.convert_samples().collect();

            let buffer = SamplesBuffer::new(channels, sample_rate, IntoVec(samples)).buffered();

            unsafe {
                Ok(AudioClip {
                    duration: get_duration(&path), 
                    path: path,
                    sinks: (Sink::new(&DEVICES.as_ref().unwrap().0), Sink::new(&DEVICES.as_ref().unwrap().1)),
                    buffer,
                })
            }
        }).join().unwrap()
    }

    pub fn play(&self) {
        if self.sinks.0.empty() && self.sinks.1.empty() {
            self.sinks.0.append(self.buffer.clone());
            self.sinks.1.append(self.buffer.clone());
        } else {
            self.sinks.0.play();
            self.sinks.1.play();
        }
    }

    pub fn pause(&self) {
        self.sinks.0.pause();
        self.sinks.1.pause();
    }

    pub fn stop(&self) {
        self.sinks.0.stop();
        self.sinks.1.stop();
    }

    pub fn set_volume(&self, volume_primary: f32, volume_secondary: f32) {
        self.sinks.0.set_volume(volume_primary);
        self.sinks.1.set_volume(volume_secondary);
    }

    pub fn duration_left(&self) -> Duration {
        //implement duration calculation
        Duration::from_secs(0)
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn wait_for(&self) {
        self.sinks.0.sleep_until_end();
        self.sinks.1.sleep_until_end();
    }
}

pub fn init_devices() -> bool {
    thread::spawn(move||{
        let device1 = rodio::default_output_device();
        let device2 = rodio::default_output_device();

        if device1.is_some() && device2.is_some() {
            unsafe {DEVICES = Some(Box::new((device1.unwrap(), device2.unwrap())))};
            return true;
        } else {
            return false;
        }
    }).join().unwrap()
}

pub fn get_devices() -> (&'static Device, &'static Device) {
    unsafe {
        (&DEVICES.as_ref().unwrap().0, &DEVICES.as_ref().unwrap().1)
    }
}

pub fn set_devices(devices: (Device, Device)) {
    unsafe {DEVICES = Some(Box::new(devices));};
}

fn get_duration(path: &Path) -> Duration {
    let output = Command::new("powershell")
            .args(&["/C", &format!("ffprobe.exe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 \"{}\"", path.display())])
            .output()
            .expect("failed to execute process");

    if let Ok(string) = std::str::from_utf8(&output.stdout) {
        let result: Result<f64, std::num::ParseFloatError> = string.trim_end().parse();
        if let Ok(seconds) = result {
            return Duration::from_millis((seconds * 1000f64) as u64);
        }
    }
    Duration::from_secs(0)
}