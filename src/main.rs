use std::{io::{self, Write}, process::Command};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

type Error = Box<dyn std::error::Error>;

fn main() {
    println!("Grabbert started.");
    println!("Press CTRL+C to exit.\n");

    loop {
        // let url = String::from("https://www.dumpert.nl/item/100011687_d47ffbc1");
        // let url = String::from("https://www.dumpert.nl/item/100011750_e1a24802");
        let url = prompt_url();

        match process(&url) {
            Err(e) => println!("Failed: {}", e),
            Ok(_) => println!("Success."),
        };

        println!("Finished.");
        println!();
    }
}

fn prompt_url() -> String {
    print!("Please, provide a dumpert.nl video URL: ");
    io::stdout().flush().unwrap();

    let mut url = String::new();

    io::stdin()
        .read_line(&mut url)
        .expect("Failed to accept URL.")
    ;

    println!();

    url.trim().into()
}

fn process(url: &str) -> Result<(), Error> {
    println!("Processing `{}`.", url);
    let response = request_url(url)?;

    println!("Checking domain...");
    let domain = match response.url().domain() {
        None => return Err("Domain not found. Did you provide an IP address?".into()),
        Some(x) => String::from(x),
    };

    let domain = domain.trim_start_matches("www.");
    if domain != "dumpert.nl" {
        return Err(format!("Domain `{}` not supported.", domain).into())
    }

    println!("Parsing HTML...");
    let fragment = Html::parse_document(&response.text()?);
    let selector = Selector::parse("body > script:nth-of-type(1)").unwrap();

    let json = match fragment.select(&selector).next() {
        None => return Err("Failed finding data.".into()),
        Some(element) => element.inner_html(),
    };
    let json = json.trim_start_matches("window.__DUMPERT_STATE__ = JSON.parse(\"");
    let json = json.split("\");window.__DUMPERT_SETTINGS__ =").next().unwrap();
    let json = json.replace(r#"\""#, r#"""#);

    println!("Deserializing media data...");
    let json: Value = serde_json::from_str(&json)?;
    let json = json["items"]["item"]["item"].to_owned();

    let dumpert_item: DumpertItem = serde_json::from_value(json)?;

    println!("Searching for video...");
    let medium = dumpert_item.media.iter()
        .find(|m| matches!(m, DumpertMedium::Video(_)))
    ;
    let medium = match medium {
        None => return Err("Failed finding a video.".into()),
        Some(x) => x,
    };

    let video = match medium {
        DumpertMedium::Video(x) => x,
        _ => return Err("Failed finding a video.".into()),
    };

    println!("Searching video variant `stream`...");
    let variant = video.variants.iter()
        .find(|v| v.version == "stream")
    ;
    let variant = match variant {
        None => return Err("Failed finding video variant `stream`.".into()),
        Some(x) => x,
    };
    println!("Stream found: `{}`.", &variant.uri);
    println!("{:#?}", &variant.uri);

    println!("Downloading to mp4...");
    let output_destination = &format!("{}.mp4", dumpert_item.title);

    Command::new("ffmpeg")
        .args(["-i", &variant.uri])
        .args(["-c", "copy"])
        .arg(output_destination)
        .arg("-y")
        .output()
        .expect(&format!("Failed copying stream to `{}`.", output_destination))
    ;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct DumpertItem {
    title: String,
    description: String,
    date: String,
    media: Vec<DumpertMedium>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag="mediatype")]
#[serde(rename_all="UPPERCASE")]
enum DumpertMedium {
    Video(DumpertVideo),
    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
struct DumpertVideo {
    duration: u16,
    variants: Vec<DumpertMediumVariant>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DumpertMediumVariant {
    version: String,
    uri: String,
}

fn request_url(url: &str) -> Result<reqwest::blocking::Response, Error> {
    let response = reqwest::blocking::get(url)?;

    match response.status().is_success() {
        false => return Err(format!("Unexpected status: `{}`.", response.status()).into()),
        true => {},
    }

    Ok(response)
}
