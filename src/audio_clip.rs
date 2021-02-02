use ez_audio::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub struct AudioClip {
    handles: (AudioHandle<usize>, AudioHandle<usize>),
    duration: Duration,
}

impl AudioClip {
    pub fn new<F: 'static + FnMut(&mut usize) + Send + Clone>(
        path: PathBuf,
        context: Context,
        devices: (&Device, &Device),
        id: usize,
        closure: F,
    ) -> Result<Self, AudioError> {
        let handles = (
            AudioLoader::new(&path, context.clone())
                .device(devices.0)
                .user_data(id)
                .on_end(closure.clone())
                .load()?,
            AudioLoader::new(&path, context.clone())
                .device(devices.1)
                .user_data(id)
                .on_end(closure)
                .load()?,
        );
        Ok(AudioClip {
            handles,
            duration: get_duration(&path),
        })
    }

    pub fn play(&self) {
        self.handles.0.play();
        self.handles.1.play();
    }

    pub fn restart(&self) {
        self.handles.0.reset();
        self.handles.1.reset();
        self.handles.0.play();
        self.handles.1.play();
    }

    pub fn stop(&self) {
        self.handles.0.stop();
        self.handles.1.stop();
    }

    pub fn set_volume(&self, volume_primary: f32, volume_secondary: f32) {
        self.handles.0.set_volume(volume_primary);
        self.handles.1.set_volume(volume_secondary);
    }

    pub fn is_playing(&self) -> bool {
        return self.handles.0.is_playing() || self.handles.1.is_playing();
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn set_primary_device(&self, device: &Device) {
        self.handles.1.stop();
        self.handles.0.set_output_device(device);
    }

    pub fn set_secondary_device(&self, device: &Device) {
        self.handles.0.stop();
        self.handles.1.set_output_device(device);
    }
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
