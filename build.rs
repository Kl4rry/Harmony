use base64::encode;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use winres::WindowsResource;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let resource_regex = Regex::new(r"\{(\w+):(\w+).(\w+)\}").unwrap();

    let mut prefixes = HashMap::new();
    prefixes.insert("svg", r#"data:image/svg+xml;base64,"#);
    prefixes.insert("png", r#"data:image/png;base64,"#);

    let raw_html = include_str!("src/index.html");
    let css = include_str!("src/style.css");
    let reset = include_str!("src/normalize.css");
    let js = include_str!("src/index.js");

    let mut html_content = html_minifier::minify(raw_html).unwrap();
    html_content = html_content.replace("{reset}", &html_minifier::css::minify(reset).unwrap());
    html_content = html_content.replace("{css}", &html_minifier::css::minify(css).unwrap());
    html_content = html_content.replace("{js}", js);

    let temp_html = html_content.clone();
    let captures = resource_regex.captures_iter(&temp_html);

    for cap in captures {
        let mut path: PathBuf = PathBuf::from(&cap[1]);
        path.push(format!("{}.{}", &cap[2], &cap[3]));

        let resource = fs::read(path).unwrap();
        let b64 = encode(resource);

        let prefix = prefixes.get(&cap[3]).unwrap_or(&"");

        let data = format!("{}{}", prefix, b64);

        html_content = html_content.replace(&cap.get(0).unwrap().as_str(), &data);
    }

    let dest_path = Path::new(&out_dir).join("html_content.html");
    fs::write(&dest_path, html_content).unwrap();

    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set_language(winapi::um::winnt::MAKELANGID(
            winapi::um::winnt::LANG_ENGLISH,
            winapi::um::winnt::SUBLANG_ENGLISH_US,
        ));
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}
