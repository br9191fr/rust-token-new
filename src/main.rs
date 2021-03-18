extern crate reqwest;
extern crate serde_json;

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use reqwest::{Client,StatusCode};
use ring::digest::{Context, Digest, SHA256};
use serde_json::{json, Error};
use serde::{Deserialize};
use std::fs::read_to_string;



#[derive(Deserialize,Debug)]
struct Token {
    token: String,
}
#[derive(Deserialize,Debug)]
struct EasError {
    Message: String,
}
#[derive(Deserialize,Debug)]
enum EasResult {
    Token (Token),
    EasError (EasError),
    None
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {}", self.token)
    }
}

impl std::fmt::Display for EasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message: {}", self.Message)
    }
}
fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest,Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        if let Ok(count) = reader.read(&mut buffer) {
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }
    }

    Ok(context.finish())
}
/*
Tests des web services d'EAS
 */
async fn eas_get_token(display : bool) -> Result<EasResult, reqwest::Error> {
    let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/service/authenticate";
    println!("Start get token");
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
    if display {
        let headers = response.headers();
        for (key, value) in headers.iter() {
            println!("{:?}: {:?}", key, value);
        }
    }
    let body = response.text().await.unwrap();
    if !sc.is_success() {
        println!("Request failed => {}",sc);

        let r: Result<EasError, Error> = serde_json::from_str(&body);
        let r_final = match r {
            Ok(res) => {
                //println!("EAS error: => {}",res);
                EasResult::EasError(res)
            },
            Err(_e) => {
                //println!("EAS error???: => {}",e);
                EasResult::None
            }
        };
        return Ok(r_final);
    }



    if display {
        // Affiche le statut
        println!("Status : {:#?}", sc);
        // Affiche le body <=> jeton
        println!("Body : {:#?}", body);
    }

    // Conversion en jeton

    let r: Result<Token, Error> = serde_json::from_str(&body);
    let t: EasResult = match r {
        Ok(res) => EasResult::Token(res),
        Err(_e) => EasResult::None
    };
    println!("stop get token");
    Ok(t)
}

async fn eas_process(filename: &str) -> Result<bool, reqwest::Error > {
    let opt_t = eas_get_token(false).await;

    let (eas_r,status) = match opt_t {
        Ok(EasResult::Token(t)) => {
            println!("token found {}",t);
            (EasResult::Token(t), true)
        },
        Ok(EasResult::EasError(eas )) => {
            println!("eas error found {}",eas);
            (EasResult::EasError(eas), false)
        },
        Ok(EasResult::None) => {
            println!("Error (?) while getting token");
            (EasResult::None, false)
        },
        Err(e) => {
            println!("Error while getting token : => {}",e);
            (EasResult::None, false)
        }
    };

    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    println!("Start digest computation for {}", filename);
    if let Ok(input_file) = File::open((filename)) {
        println!("Digest computation step 2");
        let reader = BufReader::new(input_file);
        println!("Digest computation step 3");
        if let Ok(digest) = sha256_digest(reader) {
            println!("SHA256 Digest is {:#?}",digest);
        }
        else {
            println!("Error while digest computation");
        }
    }
    else {
        println!("Error openning file {}",filename);
    }
    return Ok(true);
}

#[tokio::main]
async fn main() {
    //send_sms().await;
    //get_data().await;
    let final_result = eas_process("/Users/bruno/dvlpt/rust/test.txt").await;
    match final_result {
        Ok(true) =>  println!("eas test is ok"),
        Ok(false) => println!("eas test failed"),
        Err(e) => println!("Reqwest error {:#}",e)
    }
    println!("end");
}