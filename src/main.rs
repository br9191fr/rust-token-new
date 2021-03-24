extern crate reqwest;
extern crate serde_json;
extern crate data_encoding;

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use data_encoding::HEXLOWER;
use reqwest::{Client};
use ring::digest::{Context, Digest,SHA256};
use serde_json::{json, Error};
use serde::{Deserialize};

#[derive(Deserialize,Debug)]
struct Token {
    token: String,
}
#[derive(Deserialize,Debug)]
struct EasError {
    message: String,
}
#[derive(Deserialize,Debug)]
enum EasResult {
    Token (Token),
    EasError (EasError),
    None
}

impl Token {
    fn get_token(&self) -> &String {
        let string = &self.token;
        string
    }
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {}", self.token)
    }
}

impl std::fmt::Display for EasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message: {}", self.message)
    }
}

fn get_inner_token(e : EasResult) -> Option<String> {
     match e {
         EasResult::Token  (t)  => Some(t.get_token().to_string()),
         _ => None
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
    if display {println!("Start get token");}
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
    if display { println!("stop get token"); }
    Ok(t)
}

async fn eas_process(filename: &str) -> Result<bool, reqwest::Error > {
    let digest_string : String ;
    let token_string;
    if let Ok(input_file) = File::open(filename) {
        //println!("Digest computation step 2");
        let reader = BufReader::new(input_file);
        //println!("Digest computation step 3");
        if let Ok(digest) = sha256_digest(reader) {
            //println!("SHA256 Digest is {:#?}",digest);
            digest_string = HEXLOWER.encode(digest.as_ref());
            //println!("SHA256 Digest for {} is {}",filename,digest_string);
        }
        else {
            println!("Error while digest computation");
            return Ok(false);
        }
    }
    else {
        println!("Error openning file {}",filename);
        return Ok(false);
    }
    let opt_t = eas_get_token(false).await;

    let (eas_r,status) = match opt_t {
        Ok(EasResult::Token(t)) => {
            //println!("token found {}",t);
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
    token_string = get_inner_token(eas_r).unwrap();
    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    println!("token found {}",token_string);
    println!("SHA256 Digest for {} is {}",filename,digest_string);
    // upload document now

    return Ok(true);
}
/*
use reqwest::multipart;

let form = multipart::Form::new()
    // Adding just a simple text field...
    .text("username", "seanmonstar")
    // And a file...
    .file("photo", "/path/to/photo.png")?;

// Customize all the details of a Part if needed...
let bio = multipart::Part::text("hallo peeps")
    .file_name("bio.txt")
    .mime_str("text/plain")?;

// Add the custom part to our form...
let form = form.part("biography", bio);

// And finally, send the form
let client = reqwest::Client::new();
let resp = client
    .post("http://localhost:8080/user")
    .multipart(form)
    .send()?;
 */
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