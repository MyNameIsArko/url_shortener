use mongodb::{Client, Collection};
use std::env;
use std::fs;
use std::error::Error;
use serde_json::Value;
use reqwest::Client as ReqwestClient;
use String;
use bson::Document;
use mongodb::bson::doc;

// Create body for the request
fn create_data(url: &str) -> String {
    String::from("{ \"long_url\": \"") + url + "\" }"
}

// Create headers for the request
fn create_headers(token: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    let auth =String::from("Bearer ") + token;
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth).unwrap(),
    );
    headers
}

// Function that posts given url with data and headers to the bitly api and returns the response
async fn shorten_url(url: &str, token: &str) -> Result<String, reqwest::Error> {
    let text = ReqwestClient::new()
        .post("https://api-ssl.bitly.com/v4/shorten")
        .headers(create_headers(token))
        .body(create_data(url))
        .send()
        .await?
        .text()
        .await?;
    Ok(text)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Read settings from config.json
    let settings: Value = serde_json::from_str(&fs::read_to_string("config.json").unwrap()).unwrap();
    let bitly_access_token: &str = settings["bitlyAccessToken"].as_str().unwrap();
    let client_uri: &str = settings["clientURI"].as_str().unwrap();

    // A Client is needed to connect to MongoDB:
    let client = Client::with_uri_str(client_uri).await?;

    // Connect to the "bitly" collection
    let database : Collection<Document> = client.database("test_db").collection("bitly");

    // Get the url from the command line
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    // Check if the url is already in the database
    let already_created = database.find_one(doc! { "url": url }, None).await.unwrap();
    match already_created {
        Some(doc) => {
            // If the url is already in the database, print the short url of it
            println!("From database: {}", doc.get("short").unwrap().as_str().unwrap());
        }
        None => {
            // If the url is not in the database, create a new short url, print it and insert it into the database
            let js : Value = serde_json::from_str(&shorten_url(url, bitly_access_token).await.unwrap()).unwrap();
            let short = js["id"].as_str().unwrap();
            println!("Newly created: {}", short);
            let record = doc! { "url": url, "short": short };
            database.insert_one(record, None).await?;
        }
    }

    Ok(())
}

// Check if the shorten_url function returns the correct short url
#[tokio::test]
async fn check_shortener() {
    let settings: Value = serde_json::from_str(&fs::read_to_string("config.json").unwrap()).unwrap();
    let bitly_access_token: &str = settings["bitlyAccessToken"].as_str().unwrap();
    let data = shorten_url("https://www.rust-lang.org/", bitly_access_token).await.unwrap();
    let js : Value = serde_json::from_str(&data).unwrap();
    assert_eq!(js["id"].as_str().unwrap(), "bit.ly/3y5R16y");
}

// Check if it's possible to connect to the database
#[tokio::test]
async fn connect_to_database() {
    let settings: Value = serde_json::from_str(&fs::read_to_string("config.json").unwrap()).unwrap();
    let client_uri: &str = settings["clientURI"].as_str().unwrap();
    Client::with_uri_str(client_uri).await.unwrap();
}

// Check if the database properly returns the short url when searching for it
#[tokio::test]
async fn read_value_from_database() {
    let settings: Value = serde_json::from_str(&fs::read_to_string("config.json").unwrap()).unwrap();
    let client_uri: &str = settings["clientURI"].as_str().unwrap();
    Client::with_uri_str(client_uri).await.unwrap();
    let database : Collection<Document> = Client::with_uri_str(client_uri).await.unwrap().database("test_db").collection("bitly");
    let doc = database.find_one(doc! { "url": "bitly" }, None).await.unwrap().unwrap();
    assert_eq!(doc.get("short").unwrap().as_str().unwrap(), "b");
}