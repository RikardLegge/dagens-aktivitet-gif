#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate reqwest;
extern crate scraper;
extern crate serde_json;

use std::io::Read;
use std::path::{Path};

use rocket::response::NamedFile;
use scraper::{Html, Selector};
use serde_json::{from_str, to_string};

#[get("/gif/<id>")]
fn gif(id: String) -> String {
    if let Some(log) = get_log(id).take() {
        if let Ok(res) = to_string(&log) {
            res
        } else {
            String::from("{error: true, msg: \"Failed to stringify data\"}")
        }
    } else {
        String::from("{error: true, msg: \"Failed to fetch data\"}")
    }
}

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join("index.html")).ok()
}

fn get_log(id: String) -> Option<Vec<(String, String)>> {
    // Get website
    let url = format!("https://www.dagensaktivitet.se/users/{}", id);
    let mut resp = reqwest::get(&url[..]).ok()?;
    let mut content = String::new();
    resp.read_to_string(&mut content).ok()?;

    let document = Html::parse_document(content.as_ref());
    let selector = Selector::parse(".post-body-padding").ok()?;

    // Get translations
    let mut url = String::from("https://translate.googleapis.com/translate_a/single?client=gtx&sl=sv&tl=en&dt=t&q=");
    let quotes: Vec<&str> = document.select(&selector).into_iter().filter_map(|element| {
        let quote = element.text().next()?.trim();
        Some(quote)
    }).collect();

    for quote in &quotes {
        url.push_str(quote);
        url.push_str("{}");
    }
    let mut resp = reqwest::get(&url[..]).ok()?;
    let mut content = String::new();
    resp.read_to_string(&mut content).ok()?;

    let json: serde_json::Value = from_str(&content[..]).ok()?;
    let list = &json[0][0][0];
    let translations: Vec<&str> = list.as_str()?.split("{}").collect();
    println!("Translations: {:?}", translations);

    // Get gifs
    let api_key = "cKeQgVtxI7JeprZLWD2x4gI2WbBMZtJH";
    let images: Vec<(String, String)> = translations.iter().map(|query| {
        let url = format!("https://api.giphy.com/v1/gifs/search?api_key={}&q={}&limit=1&offset=0&rating=G&lang=en", api_key, query);
        let mut resp = reqwest::get(&url[..]).ok()?;
        let mut content = String::new();
        resp.read_to_string(&mut content).ok()?;

        let json: serde_json::Value = from_str(&content[..]).ok()?;
        let gif = json["data"][0]["images"]["original"]["url"].as_str()?;

        Some(String::from(gif))
    }).zip(quotes).map(|(image, quote)| {
        let quote = String::from(quote);
        if let Some(image) = image {
            (image, quote)
        } else {
            (String::from("https://giphy.com/embed/26xBIygOcC3bAWg3S"), quote)
        }
    }).collect();

    Some(images)
}

fn main() {
    rocket::ignite().mount("/", routes![index, gif]).launch();
}