extern crate reqwest;
extern crate serde_json;

use std::fmt;
use reqwest::Client;
use serde_json::{json, Error};
use serde::{Deserialize};

#[derive(Deserialize,Debug)]
struct Token {
    token: String,
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {}", self.token)
    }
}
enum Maybe {
    Token(Token),
    None
}

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
Tests des web services d'EAS
 */
async fn get_token(display : bool) -> Result<Option<Token>, reqwest::Error> {
    let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/service/authenticate";
    println!("Start");
    let payload = json!({
    "appId":"f33c398c-0f77-4351-9f92-1e20fa3fd2f8",
    "appToken":"e1320735-e174-4150-9edb-b5daf85be6d1",
    "accountName":"demoAccount"
    });
    let response = Client::new()
        .post(request_url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send().await?;
    let sc = response.status();
    if (display) {
        let headers = response.headers();
        for (key, value) in headers.iter() {
            println!("{:?}: {:?}", key, value);
        }
    }
    let body = response.text().await.unwrap();

    if (display) {
        // Affiche le statut
        println!("Status : {:#?}", sc);
        // Affiche le body <=> jeton
        println!("Body : {:#?}", body);
    }

    // Conversion en jeton

    let r: Result<Token, Error> = serde_json::from_str(&body);
    let t = match r {
        Ok(res) => Some(res),
        Err(e) => None
    };
    println!("stop");
    Ok(t)
}
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
    //send_sms().await;
    //get_data().await;
    let opt_t = get_token(false).await;
    let t1 = match opt_t {
        Result::Ok(Some(t)) => {
            println!("token found {}",t);
            Some(t)
        },
        _ => {
            println!("Error");
            None
        }
    };
    let token = match t1 {
        Some(t) => t,
        _ => Token { token: String::from("myToken")}
    };
    println!("MyToken is {}",token);
    let t : Token = Token
        {token: String::from("myToken")};

    println!("ok");
}