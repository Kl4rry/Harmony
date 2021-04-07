#![feature(panic_info_message)]
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

use directories::*;
use ez_audio::*;
use single_instance::SingleInstance;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use web_view::*;

mod audio_clip;
mod player;
use player::*;
mod serialization;
use serialization::*;

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|panic_info| {
        println!("{:?}", panic_info);
        if let Some(message) = panic_info.message() {
            let _ = msgbox::create(
                "Error",
                &format!("panic occurred: {:?}", message),
                msgbox::IconType::Error,
            );
        } else {
            let _ = msgbox::create(
                "Error",
                &format!("panic occurred: {:?}", panic_info.message()),
                msgbox::IconType::Error,
            );
        }
    }));

    let instance = SingleInstance::new("Harmony").unwrap();
    if !instance.is_single() {
        return;
    }

    let context = tokio::spawn(async move { Context::new().unwrap() })
        .await
        .unwrap();

    let player = Arc::new(RwLock::new(Player::new()));

    let browsing = Arc::new(AtomicBool::new(false));

    let devices: Arc<RwLock<Vec<Device>>> =
        Arc::new(RwLock::new(vec![default_output_device(context.clone())]));

    let mut temp: Vec<Device> = output_devices(context.clone()).collect();
    devices.write().unwrap().append(&mut temp);
    let mut primary_device_index: usize = 0;
    let mut secondary_device_index: usize = 0;

    let ser: Option<Arc<Mutex<Serializer>>> = None;

    let html_content = include_str!(concat!(env!("OUT_DIR"), "/html_content.html"));
    //println!("{}", include_str!(concat!(env!("OUT_DIR"), "/print")));
    let window = WebViewBuilder::new()
        .title("Harmony")
        .content(Content::Html(html_content))
        .size(1280, 720)
        .min_size(640, 400)
        .resizable(true)
        .user_data(ser)
        .invoke_handler(|webview, arg| {
            let args: Vec<&str> = arg.split_whitespace().collect();

            #[allow(unused_must_use)]
            match args[0] {
                "exit" => {
                    webview.exit();
                }
                "browse" => {
                    let local_handle = webview.handle();
                    let local_browsing = browsing.clone();
                    let local_context = context.clone();
                    let local_devices = devices.clone();
                    let local_player = player.clone();

                    tokio::spawn(async move {
                        if !local_browsing.swap(true, Ordering::Relaxed) {
                            if let Some(files) = tinyfiledialogs::open_file_dialog_multi(
                                "Select audio files",
                                "",
                                Some((&["*.wav", "*.mp3", "*.flac", "*.ogg"], r#"*.wav, *.mp3", *.flac, *.ogg"#)),
                            ) {
                                let mut guard = local_player.write().unwrap();
                                for file in files {
                                    guard.load_sound(
                                        &file,
                                        local_handle.clone(),
                                        local_context.clone(),
                                        local_devices.clone(),
                                        (primary_device_index, secondary_device_index),
                                    );
                                }
                            }
                            local_browsing.store(false, Ordering::Relaxed);
                        }
                    });
                }
                "remove" => {
                    let index: usize = args[1].parse().unwrap();
                    player
                        .write()
                        .unwrap()
                        .remove(index, webview.user_data().as_ref().unwrap().clone());
                }
                "play_pause" => {
                    let index: usize = args[1].parse().unwrap();
                    let guard = player.read().unwrap();
                    if guard.is_playing(index) {
                        guard.stop(index);
                        webview.eval(&format!(r#"set_icon({}, "play-icon")"#, index));
                    } else {
                        guard.play(index);
                        webview.eval(&format!(r#"set_icon({}, "pause-icon")"#, index));
                    }
                }
                "restart" => {
                    let index: usize = args[1].parse().unwrap();
                    player.read().unwrap().restart(index);
                    webview.eval(&format!(r#"set_icon({}, "pause-icon")"#, index));
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
                    list.push(']');

                    webview.eval(&format!("set_device_list({})", list));
                }
                "select_primary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();

                    let guard = player.read().unwrap();
                    guard.set_primary_device(&devices[index]);
                    primary_device_index = index;
                    guard.stop_all();

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.device.0 = index;
                    lock.save();
                }
                "select_secondary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();

                    let guard = player.read().unwrap();
                    guard.set_secondary_device(&devices[index]);
                    secondary_device_index = index;
                    guard.stop_all();

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.device.1 = index;
                    lock.save();
                }
                "set_volume" => {
                    let volume_primary: f32 = args[1].parse().unwrap();
                    let volume_secondary: f32 = args[2].parse().unwrap();

                    player
                        .read()
                        .unwrap()
                        .set_volume(volume_primary, volume_secondary);

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.volume = (volume_primary, volume_secondary);
                    lock.save();
                }
                "ready" => {
                    let local_handle = webview.handle();
                    let local_context = context.clone();
                    let local_devices = devices.clone();
                    let local_player = player.clone();

                    tokio::spawn(async move {
                        let user_dirs = UserDirs::new().expect("unable to find user directory");
                        let mut path = user_dirs
                            .document_dir()
                            .expect("unable to find document directory")
                            .to_path_buf();
                        path.push("Harmony");
                        if !path.is_dir() {
                            std::fs::create_dir(&path).expect("unable to create config directory");
                        }
                        path.push("harmony.ron");

                        local_handle.dispatch(move |webview| {
                            let de = Deserializer::load(&path);
                            *webview.user_data_mut() = Some(Arc::new(Mutex::new(Serializer::new(
                                &path,
                                de.config.volume,
                                de.config.device,
                            ))));

                            webview.eval(&format!(
                                "update_volume({},{});",
                                de.config.volume.1, de.config.volume.0
                            ));
                            webview.eval(&format!(
                                "update_device({},{});",
                                de.config.device.0, de.config.device.1
                            ));

                            let mut player_guard = local_player.write().unwrap();
                            {
                                let devices_guard = &*local_devices.read().unwrap();
                                player_guard.set_primary_device(&devices_guard[de.config.device.0]);
                                //primary_device_index = de.config.device.0;
                                player_guard.set_secondary_device(&devices_guard[de.config.device.1]);
                                //secondary_device_index = de.config.device.1;
                            }

                            for clip in de.config.clips.iter() {
                                player_guard.load_sound(
                                    &clip.path,
                                    webview.handle(),
                                    local_context.clone(),
                                    local_devices.clone(),
                                    (primary_device_index, secondary_device_index),
                                );
                            }

                            Ok(())
                        });
                    });
                }
                _ => println!("{}", arg),
            }
            Ok(())
        })
        .build()
        .unwrap();

    let _ = window.run();

    player.read().unwrap().stop_all();
}
