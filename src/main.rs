use directories::*;
use ez_audio::*;
use nfd::Response;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use web_view::*;

mod audio_clip;
mod player;
use player::*;
mod serialization;
use serialization::*;

#[tokio::main]
async fn main() {
    let html_content = include_str!(concat!(env!("OUT_DIR"), "/html_content.html"));
    //println!("{}", include_str!(concat!(env!("OUT_DIR"), "/print")));

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

    let window = WebViewBuilder::new()
        .title("Soundboard")
        .content(Content::Html(html_content))
        .size(1280, 720)
        .min_size(640, 360)
        .resizable(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            let args: Vec<&str> = arg.split_whitespace().collect();

            #[allow(unused_must_use)]
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
                    list.push_str("]");

                    webview.eval(&format!("set_device_list({})", list));
                }
                "select_primary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();
                    player.set_primary_device(&devices[index]);
                    primary_device_index = index;
                }
                "select_secondary" => {
                    let index: usize = args[1].parse().unwrap();
                    let devices = &*devices.read().unwrap();
                    player.set_secondary_device(&devices[index]);
                    secondary_device_index = index;
                }
                "set_volume" => {
                    let volume_primary: f32 = args[1].parse().unwrap();
                    let volume_secondary: f32 = args[2].parse().unwrap();
                    player.set_volume(volume_primary, volume_secondary);
                }
                _ => println!("{}", arg),
            }
            Ok(())
        })
        .build()
        .unwrap();

    tokio::spawn(async move {
        if let Some(user_dirs) = UserDirs::new() {
            let mut path = user_dirs.document_dir().unwrap().to_path_buf();
            path.push("config.ron");
            let de = Deserializer::load(&path);
            for clip in de.config.clips.iter() {

            }
            let mut ser = Serializer::new(&path, (0.0, 0.0), (0, 0));
            ser.save();
        }
    });

    let _ = window.run();
}
