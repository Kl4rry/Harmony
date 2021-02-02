use ez_audio::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use web_view::*;

use super::audio_clip::*;

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

    pub fn load_sound(
        &mut self,
        str_path: &str,
        handle: Handle<()>,
        context: Context,
        devices: Arc<RwLock<Vec<Device>>>,
        device_indexes: (usize, usize),
    ) {
        let result: Result<PathBuf, std::convert::Infallible> = str_path.parse();

        if let Ok(path) = result {
            let name: Arc<String> =
                Arc::new(path.file_name().unwrap().to_str().unwrap().to_string());

            let id = self.tombstones.pop().unwrap_or_else(|| {
                let temp = self.id.clone();
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
                tokio::spawn(async move {
                    let local_handle = handle.clone();
                    let devices = devices.read().unwrap();
                    let result = AudioClip::new(
                        path,
                        context.clone(),
                        (&devices[device_indexes.0], &devices[device_indexes.1]),
                        id,
                        move |userdata| {
                            let data = userdata.clone();
                            local_handle.dispatch(move |webview| {
                                webview.eval(&format!(r#"set_icon({}, "play-icon")"#, data))
                            });
                        },
                    );
                    if let Ok(clip) = result {
                        let duration = clip.duration();
                        clips.write().unwrap().insert(id, clip);

                        handle.dispatch(move |webview| {
                            webview.eval(&format!(
                                r#"init_sound({}, "{}", "{}");"#,
                                id,
                                &name,
                                duration_to_string(duration)
                            ))
                        });
                    } else {
                        handle.dispatch(move |webview| {
                            webview.eval(&format!("remove_sound({});", id))
                        });
                    }
                });
            }
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
}

fn duration_to_string(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;

    let mut duration_string = String::from("");

    let temp;
    if minutes < 10 {
        temp = format!("0{}:", minutes);
    } else {
        temp = format!("{}", minutes);
    }

    duration_string.push_str(&temp);
    duration_string.push_str(&format!("{}", seconds));
    duration_string
}
