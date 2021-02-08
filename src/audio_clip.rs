use ez_audio::*;
use std::path::Path;
use std::time::Duration;

pub struct AudioClip {
    handles: (AudioHandle<usize>, AudioHandle<usize>),
}

impl AudioClip {
    pub fn new<F: 'static + FnMut(&mut usize) + Send + Clone, P: AsRef<Path>>(
        path: P,
        context: Context,
        devices: (&Device, &Device),
        id: usize,
        closure: F,
    ) -> Result<Self, AudioError> {
        let handles = (
            AudioLoader::new(path.as_ref(), context.clone())
                .device(devices.0)
                .user_data(id)
                .on_end(closure.clone())
                .load()?,
            AudioLoader::new(&path, context)
                .device(devices.1)
                .user_data(id)
                .on_end(closure)
                .load()?,
        );
        Ok(AudioClip { handles })
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
        self.handles.0.is_playing() || self.handles.1.is_playing()
    }

    pub fn duration(&self) -> Duration {
        self.handles.0.duration()
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
