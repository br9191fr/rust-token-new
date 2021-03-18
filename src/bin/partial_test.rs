extern crate reqwest;
extern crate serde_json;

use std::fmt;
use reqwest::{Client,StatusCode};
use serde_json::{json, Error};
use serde::{Deserialize};
use std::fs::read_to_string;

/*
Test de base appel http pour récupérer des data JSON
 */
async fn get_data() -> Result<(), reqwest::Error> {
    let request_url = "http://192.168.0.43:3000/data";
    println!("start");

    let response = Client::new()
        .get(request_url)
        .header("Accept", "application/json")
        .send().await?;
    let sc = response.status();

    let headers = response.headers();
    for (key, value) in headers.iter() {
        println!("{:?}: {:?}", key, value);
    }
    let body = response.text().await.unwrap();
    println!("Body : {:#?}", body);
    println!("Status : {:#?}", sc);
    println!("stop");
    Ok(())
}
/*

/*
Connexion à un service d'envoi de SMS via appel de web service
 */
async fn send_sms() -> Result<(), reqwest::Error> {
    let payload = json!({
    "sms": {
        "message": {
        "text": "Hello Bruno Rust test2 !",
        "sender": "Rust Demo",
    },
    "recipients": {
        "gsm":  [
                    {
                        "value": "33622696747",
                    }
                ]
        }
    }
    });

    println!("{}", payload.to_string());
    let request_url = "https://api.smsfactor.com/send";

    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI0MTQ3MSIsImlhdCI6MTYxNTU1OTM5Nn0.MHGpHL7UQa8VmXJtQ6xBSP6At19uzc2_fVJwVQsVvO0"; //Your first token must be generated on our plateform at https://secure.smsfactor.com/token.html
    println!("start");

    let response = Client::new()
        .post(request_url)
        .header("Accept", "application/json")
        .header("Authorization", format!("{}{}", "Bearer ", token))
        .json(&payload)
        .send().await?;

    let sc = response.status();
    let body = response.text().await.unwrap();

    println!("Body : {:#?}", body);
    println!("Status : {:#?}", sc);
    println!("stop");
    Ok(())
}

#[tokio::main]
async fn main() {
    send_sms().await;
    get_data().await;

    println!("end");
}