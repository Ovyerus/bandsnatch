use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::vec::Vec;

#[derive(Deserialize)]
struct RawCookie {
    #[serde(rename = "Name raw")]
    name: String,
    #[serde(rename = "Content raw")]
    content: String,
}

/// Get hashmap of cookies from a `cookies.json` string.
fn get_json_cookies(json: &str) -> HashMap<String, String> {
    let raw = serde_json::from_str::<Vec<RawCookie>>(&json).unwrap();
    let mut map = HashMap::<String, String>::new();
    let cookie_iter = raw.iter();

    for c in cookie_iter {
        map.insert(c.name.clone(), c.content.clone());
    }

    map
}

fn get_text_cookies(content: &str) -> HashMap<String, String> {
    let lines = content.split('\n');
    let mut map = HashMap::<String, String>::new();

    for l in lines {
        if !l.starts_with('#') {
            let columns: Vec<&str> = l.split('\t').collect();
            // TODO: comment the significance of this
            if columns.len() == 7 {
                map.insert(String::from(columns[5]), String::from(columns[6]));
            }
        }
    }

    map
}

// get cookies from firefox?

pub fn get_bandcamp_cookies(path: Option<&str>) -> Result<HashMap<String, String>, String> {
    if let Some(path) = path {
        let data = fs::read_to_string(path).expect(&format!("Cannot read cookies file '{path}'"));
        // TODO: need to return results from these functions
        let cookies = if path.ends_with(".json") {
            get_json_cookies(&data)
        } else {
            get_text_cookies(&data)
        };

        return Ok(cookies);
    }

    // If no path provided, look for local cookies, or fallback to Firefox.

    get_bandcamp_cookies(Some("./cookies.json"))
        .or_else(|_| get_bandcamp_cookies(Some("./cookies.txt")))
        .or(Err(String::from("Failed to get cookies")))
}

pub fn cookies_to_string(cookies: &HashMap<String, String>) -> String {
    let mut strings = Vec::<String>::new();

    for (key, val) in cookies.iter() {
        strings.push(format!("{key}={val}"));
    }

    strings.join("; ")
}
