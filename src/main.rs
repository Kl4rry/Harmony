use web_view::*;
use nfd::Response;
use tokio::prelude::*;

use std::path::PathBuf;
use std::thread;
use std::sync::{Mutex, Arc};
use std::sync::atomic::AtomicUsize;

use rodio::{Sink, DeviceTrait, Device};

mod audio_clip;
use audio_clip::*;

struct Player {
    id: usize,
    clips: Arc<Mutex<Vec<Option<AudioClip>>>>,
    tombstones: Vec<usize>,
}

impl Player {
    fn init() -> Player {
        Player {
            id: 0,
            clips: Arc::new(Mutex::new(Vec::new())),
            tombstones: Vec::new(),
        }
    }

    fn play(&self, id: usize) {
        self.clips.lock().unwrap()[id].as_ref().unwrap().play();
    }

    fn load_sound(&mut self, str_path: &str, handle: Handle<()>){
        let result: Result<PathBuf, std::convert::Infallible> = str_path.parse();
        

        if let Ok(path) = result {
            //let name: String = path.file_name().unwrap().to_string();

            let mut append = false;
            let id = self.tombstones.pop().unwrap_or_else(||{
                append = true;
                let temp = self.id.clone();
                self.id += 1;
                temp
            });

            handle.dispatch(move |webview| {
                webview.eval(&format!("new_sound({}, \"test\");", id));
                Ok(())
            });

            let clips = self.clips.clone();

            tokio::spawn(async move {
                let res = AudioClip::new(path);
                if let Ok(clip) = res {
                    if append {
                        clips.lock().unwrap().push(Some(clip));
                    } else {
                        clips.lock().unwrap()[id] = Some(clip);
                    }

                    handle.dispatch(move |webview| {
                        webview.eval(&format!("init_sound({}, \"test\");", id));
                        Ok(())
                    });
                } else {
                    handle.dispatch(move |webview| {
                        webview.eval(&format!("init_sound({}, \"test\");", id));
                        Ok(())
                    });
                }
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let raw_html = include_str!("index.html");
    let css = include_str!("style.css");
    let reset = include_str!("normalize.css");
    let js = include_str!("index.js");

    let mut html_content = raw_html.replace("{reset}", reset);
    html_content = html_content.replace("{css}", css);
    html_content = html_content.replace("{js}", js);

    let test = "hej"; 
    let mut player = Player::init();
    
    let window = WebViewBuilder::new()
        .title("Soundboard")
        .content(Content::Html(html_content))
        .size(1280, 720)
        .min_size(640, 360)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            let args: Vec<&str> = arg.split_whitespace().collect();
            match args[0] {
                "browse" => {
                    if let Ok(response) = nfd::open_file_multiple_dialog(None, None) {
                        match response {
                            Response::OkayMultiple(files) => {
                                for file in files {
                                    player.load_sound(&file, webview.handle());
                                }
                            },
                            _ => println!("User canceled{}", test),
                        }
                    }
                }
                "play" => {
                    let index: usize = args[1].parse().unwrap();
                    player.play(index);
                }
                _ => println!("{}", arg),
                //_ => unimplemented!(),
            }
            Ok(())
        })
        .build()
        .unwrap();

    init_devices();

    let _data = window.run();
}