use ez_audio::*;
use nfd::Response;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use web_view::*;

mod audio_clip;
use audio_clip::*;

struct Player {
    id: usize,
    clips: Arc<RwLock<HashMap<usize, AudioClip>>>,
    tombstones: Vec<usize>,
}

impl Player {
    fn new() -> Self {
        Player {
            id: 0,
            clips: Arc::new(RwLock::new(HashMap::new())),
            tombstones: Vec::new(),
        }
    }

    fn play(&self, id: usize) {
        self.clips.read().unwrap()[&id].play();
    }

    fn restart(&self, id: usize) {
        self.clips.read().unwrap()[&id].restart();
    }

    fn load_sound(
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
                    let devices = devices.read().unwrap();
                    let result = AudioClip::new(
                        path,
                        context.clone(),
                        (&devices[device_indexes.0], &devices[device_indexes.1]),
                        id,
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

    fn set_primary_device(&self, device: &Device) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.set_primary_device(device);
        }
    }

    fn set_secondary_device(&self, device: &Device) {
        for clip in self.clips.read().unwrap().iter() {
            clip.1.set_secondary_device(device);
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

#[tokio::main]
async fn main() {
    let html_content = include_str!(concat!(env!("OUT_DIR"), "/html_content.html"));
    //println!("{}", include_str!(concat!(env!("OUT_DIR"), "/print")));

    let context = tokio::spawn(async move { Context::new().unwrap() })
        .await
        .unwrap();

    let mut player = Player::new();

    let browsing = AtomicBool::new(false);

    let mut devices: Arc<RwLock<Vec<Device>>> =
        Arc::new(RwLock::new(vec![default_output_device(context.clone())]));

    let mut temp: Vec<Device> = output_devices(context.clone()).collect();
    devices.write().unwrap().append(&mut temp);
    let mut primary_device_index: usize = 0;
    let mut secondary_device_index: usize = 0;

    let window = WebViewBuilder::new()
        .title("Soundboard")
        .content(Content::Html(html_content))
        .size(1280, 720)
        .min_size(640, 360)
        .resizable(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            let args: Vec<&str> = arg.split_whitespace().collect();
            match args[0] {
                "browse" => {
                    if !browsing.swap(true, Ordering::Relaxed) {
                        if let Ok(response) = nfd::dialog_multiple().open() {
                            if let Response::OkayMultiple(files) = response {
                                for file in files {
                                    player.load_sound(
                                        &file,
                                        webview.handle(),
                                        context.clone(),
                                        devices.clone(),
                                        (primary_device_index, secondary_device_index),
                                    );
                                }
                            }
                        }
                        browsing.store(false, Ordering::Relaxed);
                    }
                }
                "play" => {
                    let index: usize = args[1].parse().unwrap();
                    player.play(index);
                }
                "restart" => {
                    let index: usize = args[1].parse().unwrap();
                    player.restart(index);
                }
                "update_device_list" => {
                    let mut list = String::from(r#"["Default""#);
                    let mut skip = true;
                    for device in devices.read().unwrap().iter() {
                        if skip {
                            skip = false;
                            continue;
                        }
                        list.push_str(&format!(r#","{}""#, device.name()));
                    }
                    list.push_str("]");

                    webview.eval(&format!("set_device_list({})", list));
                }
                "select_primary" => {
                    let index: usize = args[1].parse().unwrap();
                    println!("{}", index);
                    let devices = &*devices.read().unwrap();
                    player.set_primary_device(&devices[index]);
                    primary_device_index = index;
                }
                "select_secondary" => {
                    let index: usize = args[1].parse().unwrap();
                    println!("{}", index);
                    let devices = &*devices.read().unwrap();
                    player.set_secondary_device(&devices[index]);
                    secondary_device_index = index;
                }
                _ => println!("{}", arg),
                //_ => unimplemented!(),
            }
            Ok(())
        })
        .build()
        .unwrap();
    
        let _ = window.run();
}
