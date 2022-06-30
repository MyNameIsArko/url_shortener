use std::env;
use serde_json::Value;
use reqwest::Client;
use String;


const BITLY_ACCESS_TOKEN: &str = "BITLY_TOKEN";


fn create_data(url: &str) -> String {
    String::from("{ \"long_url\": \"") + url + "\" }"
}

fn create_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    let auth =String::from("Bearer ") + BITLY_ACCESS_TOKEN;
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth).unwrap(),
    );
    headers
}

async fn shorten_url(url: &str) -> Result<String, reqwest::Error> {
    let text = Client::new()
        .post("https://api-ssl.bitly.com/v4/shorten")
        .headers(create_headers())
        .body(create_data(url))
        .send()
        .await?
        .text()
        .await?;
    Ok(text)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let url = &args[1];
    let js : Value = serde_json::from_str(&shorten_url(url).await.unwrap()).unwrap();
    println!("{}", js["id"].as_str().unwrap());
}

#[tokio::test]
async fn check_shortener() {
    let data = shorten_url("https://www.rust-lang.org/").await.unwrap();
    let js : Value = serde_json::from_str(&data).unwrap();
    assert_eq!(js["id"].as_str().unwrap(), "bit.ly/3y5R16y");
}