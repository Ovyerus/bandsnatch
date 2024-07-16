// use cookie_store::{Cookie, CookieStore};
use reqwest::cookie::Jar;
use serde::Deserialize;
use std::fs;
use std::vec::Vec;

#[derive(Deserialize, Debug)]
pub struct RawCookie {
    #[serde(rename = "Host raw")]
    host: String,
    #[serde(rename = "Name raw")]
    name: String,
    #[serde(rename = "Content raw")]
    content: String,
}

/// Get hashmap of cookies from a `cookies.json` string.
fn get_json_cookies(json: &str) -> Vec<RawCookie> {
    let raw = serde_json::from_str::<Vec<RawCookie>>(json).unwrap();
    let mut vec = Vec::<RawCookie>::new();
    let cookie_iter = raw.iter();

    for c in cookie_iter {
        // TODO: better way than clone?
        vec.push(RawCookie {
            host: c.host.clone(),
            name: c.name.clone(),
            content: c.content.clone(),
        })
    }

    vec
}

fn get_text_cookies(content: &str) -> Vec<RawCookie> {
    let lines = content.split('\n');
    let mut vec = Vec::<RawCookie>::new();

    for l in lines {
        if !l.starts_with('#') {
            let columns: Vec<&str> = l.split('\t').collect();
            if columns.len() == 7 {
                // Fix problem where cookies.txt only gives us raw domains.
                let mut host = "https://".to_owned();
                host.push_str(columns[0]);

                vec.push(RawCookie {
                    host: host,
                    name: String::from(columns[5]),
                    content: String::from(columns[6]),
                })
            }
        }
    }

    vec
}

// get cookies from firefox?

pub fn get_bandcamp_cookies(path: Option<&str>) -> Result<Vec<RawCookie>, String> {
    if let Some(path) = path {
        let data = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Cannot read cookies file '{path}'"));
        // TODO: need to return results from these functions
        let cookies = if path.ends_with(".json") {
            get_json_cookies(&data)
        } else {
            get_text_cookies(&data)
        };

        return Ok(cookies);
    }

    // If no path provided, look for local cookies

    get_bandcamp_cookies(Some("./cookies.json"))
        .or_else(|_| get_bandcamp_cookies(Some("./cookies.txt")))
        .or(Err(String::from("Failed to get cookies")))
}

pub fn fill_cookie_jar(cookies: Vec<RawCookie>) -> Jar {
    let jar = Jar::default();

    for RawCookie {
        host,
        name,
        content,
    } in cookies
    {
        let host = url::Url::parse(&host).expect("failed to unwrap cookies");
        let cookie = format!("{name}={content}; Domain={}", host.domain().unwrap());
        jar.add_cookie_str(&cookie, &host);
    }

    jar
}
