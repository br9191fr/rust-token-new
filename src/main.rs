extern crate reqwest;
extern crate serde_json;
extern crate data_encoding;

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use data_encoding::HEXLOWER;


use reqwest::{Body, Client};
use reqwest::multipart::{Form};
use ring::digest::{Context, Digest,SHA256};
use serde_json::{json, Error};
use serde::{Deserialize};

use tokio::fs::File as Tokio_File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Deserialize,Debug)]
struct Token {
    token: String,
}
#[derive(Deserialize,Debug)]
struct ArchiveTicket {
    archiveTicket: String,
}

#[derive(Deserialize,Debug)]
struct EasError {
    message: String,
}
#[derive(Deserialize,Debug)]
enum EasResult {
    Token (Token),
    ArchiveTicket (ArchiveTicket),
    EasError (EasError),
    None
}

struct EasInfo {
    token : String,
    filename : String,
    digest : String,
}
impl Token {
    fn get_token(&self) -> &String {
        let string = &self.token;
        string
    }
}
impl ArchiveTicket {
    fn get_archive_ticket(&self) -> &String {
        let string = &self.archiveTicket;
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
impl EasInfo {
    fn new(token : String, filename : String, digest : String) -> Self {
        EasInfo {
            token,filename,digest
        }
    }
}
fn get_inner_token(e : EasResult) -> Option<String> {
     match e {
         EasResult::Token  (t)  => Some(t.get_token().to_string()),
         _ => None
     }
}
fn get_inner_ticket(e : EasResult) -> Option<String> {
    match e {
        EasResult::ArchiveTicket  (a)  => Some(a.get_archive_ticket().to_string()),
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
// reqwest::Error
fn get_result_status<T>(opt_t : Result<EasResult, T>) -> (EasResult,bool)

    {
    let (eas_r,status) = match opt_t {
        Ok(EasResult::Token(t)) => {
            (EasResult::Token(t), true)
        },
        Ok(EasResult::ArchiveTicket(a)) => {
            (EasResult::ArchiveTicket(a), true)
        },
        Ok(EasResult::EasError(eas)) => {
            println!("eas error found {}", eas);
            (EasResult::EasError(eas), false)
        },
        _ => {
            println!("Error while operating.");
            (EasResult::None, false)
        }
    };
    (eas_r,status)
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

async fn eas_post_document(eas_info : EasInfo, display : bool) -> Result<EasResult, Box<dyn std::error::Error>> {
    let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents";
    if display { println!("Start post document"); }
    let auth_bearer = format!("Bearer {}", eas_info.token);
    //let fname = eas_info.filename.as_str();
    let fname = "/Users/bruno/dvlpt/rust/test.txt";
    let path = Path::new(fname);
    let file = Tokio_File::open(path).await?;
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_part = reqwest::multipart::Part::stream(Body::wrap_stream(stream))
        .file_name(path.file_name().unwrap().to_string_lossy())
        .mime_str("application/octet-stream")?;

    let meta = json!([{"name": "ClientId", "value": "1"},
     {"name": "CustomerId", "value": "2"},
     {"name": "Documenttype", "value": "Invoice"}]);

    let form = Form::new()
        .text("fingerprint","")
        .text("fingerprintAlgorithm","none")
        .text("metadata",meta.to_string())
        .part("document",file_part);

    let response = Client::new()
        .post(request_url)
        .header("Authorization", auth_bearer)
        .multipart(form)
        .send()
        .await?;
    //          .header("Content-Type", "application/json")
    //          .header("Accept", "application/json")
    let sc = response.status();
    //println!("Status : {:#?}", sc);
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
    // Extract archive_ticket


    let r: Result<ArchiveTicket, Error> = serde_json::from_str(&body);
    let a_ticket: EasResult = match r {
        Ok(res) => EasResult::ArchiveTicket(res),
        Err(_e) => EasResult::None
    };
    if display { println!("Stop post document"); }
    Ok(a_ticket)
}

async fn eas_process(filename: &str) -> Result<bool, reqwest::Error > {
    let digest_string : String ;
    let token_string;
    let archive_ticket : String ;
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
    let (eas_r,status) = get_result_status(opt_t);

    token_string = get_inner_token(eas_r).unwrap();
    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    println!("token found {}",token_string);
    println!("SHA256 Digest for {} is {}",filename,digest_string);
    // upload document now
    let eas_info = EasInfo::new(token_string,filename.to_string(),digest_string);
    let opt_a = eas_post_document(eas_info,true).await;
    let (eas_r, status) = get_result_status(opt_a);
    let archive_ticket = get_inner_ticket(eas_r).unwrap();
    println!("Archive ticket : {}",archive_ticket);
    return Ok(true);
}

#[tokio::main]
async fn main() {
    //send_sms().await;
    //get_data().await;
    let file_to_archive = "/Users/bruno/dvlpt/rust/test.txt";
    let final_result = eas_process(file_to_archive).await;
    match final_result {
        Ok(true) =>  println!("eas test is ok"),
        Ok(false) => println!("eas test failed"),
        Err(e) => println!("Reqwest error {:#}",e)
    }
    println!("end");
}