extern crate reqwest;
extern crate serde_json;
extern crate data_encoding;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

use data_encoding::{HEXLOWER, BASE64};


use reqwest::{Body, Client};
use reqwest::multipart::{Form};
use ring::digest::{Context, Digest,SHA256};
use serde_json::{json, Error};
use serde::{Deserialize};
use tokio::io::AsyncReadExt;
use tokio::fs::File as Tokio_File;
use tokio_util::codec::{BytesCodec, FramedRead};
//static file_to_archive : &str;
//let file_to_restore;

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
    EasDocument (EasDocument),
    EasMetaData (EasMetaData),
    EasError (EasError),
    None
}

struct EasInfo {
    token: String,
    filename: String,
    digest: String,
}
#[derive(Deserialize, Debug)]
struct EasNVPair {
    name: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct EasMetaData {
    metadata: Vec<EasNVPair>,
}
#[derive(Deserialize,Debug)]
struct EasDocument {
    mimeType : String,
    base64Document : String,
}
// TODO Define constant with EAS parameters
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
impl EasResult {
    fn get_archive_ticket(&self) -> Option<&String> {
        if let EasResult::ArchiveTicket(at) = self {
            Some(at.get_archive_ticket())
        }
        else {
            None
        }
    }
    fn get_token(&self) -> Option<&String> {
        if let EasResult::Token(t) = self {
            Some(t.get_token())
        }
        else {
            None
        }
    }
    // TODO Add show implementation for EasResult::EasMetaData
    fn show(&self) {
        match &*self {
            EasResult::Token(t) => println!("Token: {}",t),
            EasResult::ArchiveTicket(at) => println!("ArchiveTicket: {}",at),
            EasResult::EasDocument(d) => println!("Document: {}",d),
            _ => println!("unknown or Not implemented")
        }
    }
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {}", self.token)
    }
}

impl std::fmt::Display for ArchiveTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "archiveTicket: {}", self.archiveTicket)
    }
}
impl std::fmt::Display for EasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message: {}", self.message)
    }
}

impl std::fmt::Display for EasDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {:#?},{:#?}", self.mimeType,self.base64Document)
    }
}

impl std::fmt::Display for EasNVPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}, value: {}", self.name, self.value)
    }
}

impl EasInfo {
    fn new(token : String, filename : String, digest : String) -> Self {
        EasInfo {
            token,filename,digest
        }
    }
}
impl EasDocument {
    fn new(mimeType : String, base64Document : String, ) -> Self {
        EasDocument {
            mimeType, base64Document
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

fn get_result_status<T>(opt_t : Result<EasResult, T>) -> (EasResult,bool) {
    let (eas_r,status) = match opt_t {
        Ok(EasResult::Token(t)) => {
            (EasResult::Token(t), true)
        },
        Ok(EasResult::ArchiveTicket(a)) => {
            (EasResult::ArchiveTicket(a), true)
        },
        Ok(EasResult::EasDocument(d)) => {
            (EasResult::EasDocument(d), true)
        },
        Ok(EasResult::EasMetaData(m)) => {
            (EasResult::EasMetaData(m), true)
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

async fn eas_post_document(eas_info : EasInfo, display : bool) -> Result<EasResult, Box<dyn std::error::Error>>  {
    let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents";
    if display { println!("Start post document"); }
    let auth_bearer = format!("Bearer {}", eas_info.token);
    //let fname = eas_info.filename.as_str();
    // TODO Pass original file to eas_post_document function
    let fname = "/Users/bruno/dvlpt/rust/archive.txt";
    let path = Path::new(fname);
    let file = Tokio_File::open(fname).await?;
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
async fn eas_download_document(token: String, ticket: String,file_to_restore: String, display: bool) -> Result<EasResult, reqwest::Error> {

    let request_url = format!("{}/{}","https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents",ticket);
    if display { println!("Start download document"); }
    let auth_bearer = format!("Bearer {}", token);

    let response = Client::new()
        .get(request_url)
        .header("Accept", "application/json")
        .header("Authorization", auth_bearer)
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

    // récupération du doc en b64

    let r: Result<EasDocument, Error> = serde_json::from_str(&body);
    let eas_r: EasResult = match r {
        Ok(res) => EasResult::EasDocument(res),
        Err(_e) => EasResult::None
    };
    // décodage base64 => [u8]
    // TODO pass destination file to eas_download_document function
    if let EasResult::EasDocument(res) = &eas_r {
        let mimeType = &*&res.mimeType;
        let b64_document = &res.base64Document;
        let document = BASE64.decode(b64_document.as_bytes()).unwrap();
        let document_length = document.len();
        let final_document = String::from_utf8(document).unwrap();
        if display { println!("Document: {:#?}", final_document);}
        let mut file = File::create(file_to_restore).unwrap();
        // Write a slice of bytes to the file
        file.write_all(final_document.as_bytes());
        if display { println!("stop get document"); }
        return Ok(EasResult::EasDocument(EasDocument::new((*mimeType).to_string(),format!("Length {}",document_length))));
    }
    else {
        return Ok(eas_r);
    }

    Ok(eas_r)
}

async fn eas_get_document_metadata(token: String, ticket: String, display: bool) -> Result<EasResult, reqwest::Error> {
    let request_url = format!("{}/{}/metadata", "https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents", ticket);
    if display { println!("Start retrieve document metadata"); }
    let auth_bearer = format!("Bearer {}", token);

    let response = Client::new()
        .get(request_url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", auth_bearer)
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
        println!("Request failed => {}", sc);

        let r: Result<EasError, Error> = serde_json::from_str(&body);
        let r_final = match r {
            Ok(res) => {
                //println!("EAS error: => {}",res);
                EasResult::EasError(res)
            }
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

    // récupération du json

    let r: Result<EasMetaData, Error> = serde_json::from_str(&body);
    let eas_m: EasResult = match r {
        Ok(res) => {
            println!("Deserializing OK.");
            EasResult::EasMetaData(res)
        }
        Err(e) => {
            println!("Error while deserializing: {}", e);
            EasResult::None
        }
    };
    if display { println!("MetaData: {:#?}", eas_m); }
    if display { println!("stop retrieve document metadata"); }
    Ok(eas_m)
}
async fn eas_process(path_to_archive: &str, path_to_restore: &str) -> Result<bool, reqwest::Error > {
    let digest_string : String ;
    let token_string;
    let archive_ticket : String ;
    if let Ok(input_file) = File::open(path_to_archive) {
        let reader = BufReader::new(input_file);
        if let Ok(digest) = sha256_digest(reader) {
            digest_string = HEXLOWER.encode(digest.as_ref());
        }
        else {
            println!("Error while digest computation");
            return Ok(false);
        }
    }
    else {
        println!("Error openning file {}",path_to_archive);
        return Ok(false);
    }
    // authenticate and get token
    let opt_t = eas_get_token(false).await;
    let (eas_r,status) = get_result_status(opt_t);

    token_string = get_inner_token(eas_r).unwrap();
    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    println!("token found {}",token_string);
    println!("SHA256 Digest for {} is {}",path_to_archive,digest_string);

    // upload document now
    let eas_info = EasInfo::new(token_string.clone(),path_to_archive.to_string(),digest_string);
    let opt_at = eas_post_document(eas_info,false).await;
    let (eas_r, status) = get_result_status(opt_at);
    if !status {
        println!("Failed to get archive ticket. End eas process !");
        return Ok(false);
    }
    eas_r.show();
    let archive_ticket = get_inner_ticket(eas_r).unwrap();

    println!("Archive ticket : {}",archive_ticket);

    // get matching documents

    // download document
    let opt_d = eas_download_document(
        token_string.clone(),
        archive_ticket.clone(),
        path_to_restore.clone().to_string(),
        false).await;
    let (eas_r, status) = get_result_status(opt_d);
    if !status {
        println!("Failed to get and restore archive. End eas process !");
        return Ok(false);
    }
    eas_r.show();
    // get document metadata
    let opt_m = eas_get_document_metadata(
        token_string.clone(),
        archive_ticket.clone(),
        true).await;
    let (eas_r, status) = get_result_status(opt_m);
    if !status {
        println!("Failed to get metadata of archive. End eas process !");
        return Ok(false);
    }
    eas_r.show();
    return Ok(true);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing arguments\nUsage: pgm file_to_archive file_to_restore");
        return;
    }
    let file_to_archive = &args[1];
    let file_to_restore = &args[2];

    //let file_to_archive  = "/Users/bruno/dvlpt/rust/test.txt";
    let final_result = eas_process(file_to_archive, file_to_restore).await;
    match final_result {
        Ok(true) =>  println!("eas test is ok"),
        Ok(false) => println!("eas test failed"),
        Err(e) => println!("Reqwest error {:#}",e)
    }
    println!("end");
}