use ez_audio::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use web_view::*;

use super::audio_clip::*;
use super::serialization::*;

pub struct Player {
    id: usize,
    clips: Arc<RwLock<HashMap<usize, AudioClip>>>,
    tombstones: Vec<usize>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            id: 0,
            clips: Arc::new(RwLock::new(HashMap::new())),
            tombstones: Vec::new(),
        }
    }

    pub fn play(&self, id: usize) {
        self.clips.read().unwrap()[&id].play();
    }

    pub fn restart(&self, id: usize) {
        self.clips.read().unwrap()[&id].restart();
    }

    pub fn stop(&self, id: usize) {
        self.clips.read().unwrap()[&id].stop();
    }

    pub fn load_sound<P: AsRef<Path>>(
        &mut self,
        path: P,
        handle: Handle<Option<Arc<Mutex<Serializer>>>>,
        context: Context,
        devices: Arc<RwLock<Vec<Device>>>,
        device_indexes: (usize, usize),
    ) {
        let name: Arc<String> = Arc::new(
            path.as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );

        let id = self.tombstones.pop().unwrap_or_else(|| {
            let temp = self.id;
            self.id += 1;
            temp
        });

        let temp_name = name.clone();
        let res = handle.dispatch(move |webview| {
            webview.eval(&format!(r#"new_sound({}, "{}");"#, id, &temp_name))
        });

        #[allow(unused_must_use)]
        if res.is_ok() {
            let clips = self.clips.clone();
            let path_buf: PathBuf = path.as_ref().into();
            tokio::spawn(async move {
                let local_handle = handle.clone();
                let temp_buf = path_buf.clone();

                let result = tokio::task::spawn_blocking(move || {
                    let devices = devices.read().unwrap();
                    AudioClip::new(
                        &temp_buf,
                        context.clone(),
                        (&devices[device_indexes.0], &devices[device_indexes.1]),
                        id,
                        move |userdata| {
                            let data = *userdata;
                            local_handle.dispatch(move |webview| {
                                webview.eval(&format!(r#"set_icon({}, "play-icon")"#, data))
                            });
                        },
                    )
                }).await.unwrap();

                if let Ok(clip) = result {
                    let duration = clip.duration();
                    clips.write().unwrap().insert(id, clip);

                    handle.dispatch(move |webview| {
                        {
                            let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                            lock.config.clips.inner.insert(id, {
                                AudioClipData {
                                    name: name.to_string(),
                                    path: path_buf,
                                }
                            });
                            lock.save();
                        }
                        webview.eval(&format!(
                            r#"init_sound({}, "{}", "{}");"#,
                            id,
                            &name,
                            duration_to_string(duration)
                        ));
                        Ok(())
                    });
                } else {
                    handle.dispatch(move |webview| webview.eval(&format!("remove_sound({});", id)));
                }
            });
        }
    }

    pub fn set_primary_device(&self, device: &Device) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.set_primary_device(device);
        }
    }

    pub fn set_secondary_device(&self, device: &Device) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.set_secondary_device(device);
        }
    }

    pub fn is_playing(&self, id: usize) -> bool {
        self.clips.read().unwrap()[&id].is_playing()
    }

    pub fn set_volume(&self, volume_primary: f32, volume_secondary: f32) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.set_volume(volume_primary, volume_secondary);
        }
    }

    pub fn stop_all(&self) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.stop();
            clip.1.stop();
        }
    }
}

fn duration_to_string(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;

    let temp_minutes;
    if minutes < 10 {
        temp_minutes = format!("0{}", minutes);
    } else {
        temp_minutes = format!("{}", minutes);
    }

    let temp_seconds;
    if seconds < 10 {
        temp_seconds = format!("0{}", seconds);
    } else {
        temp_seconds = format!("{}", seconds);
    }

    format!("{}:{}", temp_minutes, temp_seconds)
}
