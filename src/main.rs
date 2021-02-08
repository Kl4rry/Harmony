//#![windows_subsystem = "windows"]

use directories::*;
use ez_audio::*;
use nfd::Response;
use single_instance::SingleInstance;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use web_view::*;
use std::panic;

mod audio_clip;
mod player;
use player::*;
mod serialization;
use serialization::*;

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
           let _ = msgbox::create("Error", &format!("panic occurred: {:?}", s), msgbox::IconType::Error);
        } else {
           let _ = msgbox::create("Error", "panic occurred", msgbox::IconType::Error);
        }
    }));

    let instance = SingleInstance::new("Soundboard-rs").unwrap();
    if !instance.is_single() {
        return;
    }

    let context = tokio::spawn(async move { Context::new().unwrap() })
        .await
        .unwrap();

    let mut player = Player::new();

    let browsing = AtomicBool::new(false);

    let devices: Arc<RwLock<Vec<Device>>> =
        Arc::new(RwLock::new(vec![default_output_device(context.clone())]));

    let mut temp: Vec<Device> = output_devices(context.clone()).collect();
    devices.write().unwrap().append(&mut temp);
    let mut primary_device_index: usize = 0;
    let mut secondary_device_index: usize = 0;

    let ser: Option<Arc<Mutex<Serializer>>> = None;

    let html_content = include_str!(concat!(env!("OUT_DIR"), "/html_content.html"));
    //println!("{}", include_str!(concat!(env!("OUT_DIR"), "/print")));
    let mut window = WebViewBuilder::new()
        .title("Soundboard")
        .content(Content::Html(html_content))
        .size(1280, 720)
        .min_size(640, 360)
        .resizable(true)
        .user_data(ser)
        .invoke_handler(|webview, arg| {
            let args: Vec<&str> = arg.split_whitespace().collect();

            #[allow(unused_must_use)]
            match args[0] {
                "browse" => {
                    if !browsing.swap(true, Ordering::Relaxed) {
                        if let Ok(Response::OkayMultiple(files)) = nfd::dialog_multiple().open() {
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
                        browsing.store(false, Ordering::Relaxed);
                    }
                }
                "play_pause" => {
                    let index: usize = args[1].parse().unwrap();
                    if player.is_playing(index) {
                        player.stop(index);
                        webview.eval(&format!(r#"set_icon({}, "play-icon")"#, index));
                    } else {
                        player.play(index);
                        webview.eval(&format!(r#"set_icon({}, "pause-icon")"#, index));
                    }
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
                    list.push(']');

                    webview.eval(&format!("set_device_list({})", list));
                }
                "select_primary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();
                    player.set_primary_device(&devices[index]);
                    primary_device_index = index;

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.device.0 = index;
                    lock.save();
                }
                "select_secondary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();
                    player.set_secondary_device(&devices[index]);
                    secondary_device_index = index;

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.device.1 = index;
                    lock.save();
                }
                "set_volume" => {
                    let volume_primary: f32 = args[1].parse().unwrap();
                    let volume_secondary: f32 = args[2].parse().unwrap();
                    player.set_volume(volume_primary, volume_secondary);

                    let mut lock = webview.user_data().as_ref().unwrap().lock().unwrap();
                    lock.config.volume = (volume_primary, volume_secondary);
                    lock.save();
                }
                "ready" => {
                    let user_dirs = UserDirs::new().expect("unable to find user directory");
                    let mut path = user_dirs
                        .document_dir()
                        .expect("unable to find document directory")
                        .to_path_buf();
                    path.push("config.ron");
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

                    {
                        let devices = &*devices.read().unwrap();
                        player.set_primary_device(&devices[de.config.device.0]);
                        primary_device_index = de.config.device.0;
                        player.set_secondary_device(&devices[de.config.device.1]);
                        secondary_device_index = de.config.device.1;
                    }

                    for clip in de.config.clips.iter() {
                        player.load_sound(
                            &clip.path,
                            webview.handle(),
                            context.clone(),
                            devices.clone(),
                            (primary_device_index, secondary_device_index),
                        );
                    }
                }
                _ => println!("{}", arg),
            }
            Ok(())
        })
        .build()
        .unwrap();

    let _ = window.run();
}
