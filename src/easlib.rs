#![allow(dead_code)]

mod lib;

use std::collections::HashMap;

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::str;
use std::sync::Mutex;

use data_encoding::{HEXLOWER, BASE64};
use lazy_static::lazy_static;

use reqwest::{Body, Client};
use reqwest::multipart::{Form};
use ring::digest::{Context, Digest,SHA256};
use serde_json::{json, Error};
use serde::{Deserialize,Serialize};
use tokio::fs::File as Tokio_File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    appId: String,
    appToken: String,
    accountName: String,
}
impl Credentials {
    pub fn new(id: String, token: String, name: String) -> Self {
        Credentials {
            appId: id, appToken: token, accountName: name
        }
    }

}
#[derive(Deserialize,Debug)]
pub struct Token {
    token: String,
}
impl Token {
    fn new(token: String) -> Self {
        Token { token }
    }
    fn get_token(&self) -> &String {
        let string = &self.token;
        string
    }
}
#[derive(Deserialize,Debug)]
pub struct ArchiveTicket {
    archiveTicket: String,
}
impl ArchiveTicket {
    fn get_archive_ticket(&self) -> &String {
        let string = &self.archiveTicket;
        string
    }
    fn new(archiveTicket: String) -> Self {
        ArchiveTicket { archiveTicket }
    }
}
#[derive(Deserialize,Debug)]
pub struct EasError {
    message: String,
}
#[derive(Deserialize,Debug)]
pub enum EasResult {
    Token (Token),
    ArchiveTicket (ArchiveTicket),
    EasDocument (EasDocument),
    EasMetaData (EasMetaData),
    EasError (EasError),
    ApiOk,
    None
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
    pub fn show(&self, msg: &str) {
        match &*self {
            EasResult::Token(t) => println!("[{}] Token: {}",msg,t),
            EasResult::ArchiveTicket(at) => println!("[{}] ArchiveTicket: {}",msg,at),
            EasResult::EasDocument(d) => println!("[{}] Document: {}",msg,d),
            EasResult::EasMetaData(m) => println!("[{}] MetaData: {}",msg,m),
            EasResult::ApiOk => println!("[{}] API Called OK",msg),
            _ => println!("[{}] Unknown or Not implemented",msg)
        }
    }
}
#[derive(Deserialize, Debug)]
pub struct EasInfo {
    token: String,
    filename: String,
    address: String,
    digest: String,
}
impl EasInfo {
    fn new(token : String, filename : String, address : String, digest : String) -> Self {
        EasInfo {
            token,filename,address, digest
        }
    }
}
#[derive(Deserialize, Debug)]
struct EasNVPair {
    name: String,
    value: String,
}
#[derive(Deserialize, Debug)]
pub struct EasMetaData {
    metadata: Vec<EasNVPair>,
}
#[derive(Deserialize,Debug)]
pub struct EasDocument {
    mimeType : String,
    base64Document : String,
}
impl EasDocument {
    fn new(mime_type : String, base64_document : String, ) -> Self {
        EasDocument {
            mimeType: mime_type , base64Document: base64_document
        }
    }
}
pub struct EasAPI {
    credentials: Credentials,
    token: Option<Token>,
    digest: Option<String>,
    ticket: Option<ArchiveTicket>,
}
impl EasAPI {
    pub fn new(credentials: Credentials) -> Self {
        EasAPI {credentials: credentials, token: None,digest: None, ticket: None}
    }
    pub fn set_credentials(&mut self, credentials: Credentials) {
        self.credentials = credentials;
    }
    pub fn set_token(&mut self, token: String) {
        self.token = Some(Token::new(token));
    }
    pub fn get_token_string(&self) -> &String {
        self.token.as_ref().unwrap().get_token()
    }
    pub fn get_ticket_string(&self) -> &String {
        self.ticket.as_ref().unwrap().get_archive_ticket()
    }
    pub fn get_token(&self) -> &Option<Token> {
        match &self.token {
            Some(_) => &self.token,
            _ => &None,
        }
    }
    pub fn set_digest(&mut self, digest: String) {
        self.digest = Some(digest)
    }
    pub fn get_digest(&self) -> &Option<String> {
        match &self.digest {
            Some(_) => &self.digest,
            _ => &None,
        }
    }
    pub async fn eas_get_token(&mut self, display : bool) -> Result<EasResult, reqwest::Error> {
        let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/service/authenticate";
        if display {println!("Start get token");}
        let cred_value = serde_json::to_value(&self.credentials).unwrap();
        let response = Client::new()
            .post(request_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&cred_value)
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
            return Ok(err_to_eas_result(body));
        }

        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }

        // deserialize to token

        let r: Result<Token, Error> = serde_json::from_str(&body);
        let t: EasResult = match r {
            Ok(res) => {
                self.token = Some(res);
                EasResult::ApiOk
            },
            Err(_e) => EasResult::None
        };
        if display { println!("stop get token"); }
        Ok(t)
    }
    // TODO use only one address address (string)  or address1 (integer)
    pub async fn eas_post_document(&mut self, address: &str, address1: i32, display: bool) -> Result<EasResult, Box<dyn std::error::Error>>  {
        let request_url = "https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents";
        if display { println!("Start post document"); }
        let auth_bearer = format!("Bearer {}", self.get_token_string());
        //let f1 : &str;
        // TODO Use default location if actual location is unknown
        // TODO choose between sync and async version
        let my_ref = LOCATIONS.lock().unwrap();
        let my_ref1 = LOCATIONS2.lock().unwrap();
        let address = my_ref.get(address);
        let address1 = my_ref1.get(&address1);
        let fname_ok = match address {
            Some (f) => {
                if display {println!("ok nice use f == {}",f);}
                f
            },

            _ => {
                println!("ko use default value");
                "/Users/bruno/dvlpt/rust/archive.txt"
            },
        };
        let fname_ok1 = match address1 {
            Some (f) => {
                if display {println!("ok nice use f == {}",f);}
                f
            },

            _ => {
                println!("ko use default value");
                "/Users/bruno/dvlpt/rust/archive.txt"
            },
        };
        if display {
            println!("digest: {}",self.digest.as_ref().unwrap().clone());
        }
        // async version
        let _path_old = Path::new(fname_ok);
        let path_ok = Path::new(fname_ok1);
        let file = Tokio_File::open(path_ok).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let _file_part = reqwest::multipart::Part::stream(Body::wrap_stream(stream))
            .file_name(path_ok.file_name().unwrap().to_string_lossy())
            .mime_str("application/octet-stream")?;
        // sync version
        let mut buffer = Vec::new();
        let path1 = Path::new(fname_ok);
        let mut file1 = File::open(path1).unwrap();
        let _file_content_length = file1.read_to_end(&mut buffer);
        let file_content = str::from_utf8(&*buffer).unwrap().to_string();
        let file_part1 = reqwest::multipart::Part::text(file_content)
            .file_name(path1.file_name().unwrap().to_string_lossy())
            .mime_str("application/octet-stream").unwrap();
        let meta = json!([{"name": "ClientId", "value": "1"},
     {"name": "CustomerId", "value": "2"},
     {"name": "Documenttype", "value": "Invoice"}]);

        // TODO choose most appropriate part :
        // async => file_part
        // sync  => file_part1
        let form = Form::new()
            .text("fingerprint",self.digest.as_ref().unwrap().clone())
            .text("fingerprintAlgorithm","SHA-256")
            .text("metadata",meta.to_string())
            .part("document",file_part1);
        let response = Client::new()
            .post(request_url)
            .header("Authorization", auth_bearer)
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;
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
            return Ok(err_to_eas_result(body));
        }
        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }
        // Extract archive_ticket

        let r: Result<ArchiveTicket, Error> = serde_json::from_str(&body);
        let a_ticket: EasResult = match r {
            Ok(res) => {
                self.ticket = Some(res);
                if display { println!("Body contains ticket"); }
                EasResult::ApiOk
            },
            Err(e) => {
                if display {
                    println!("Unable to deserialize body => ticket\nError {}",e);
                };
                EasResult::None
            }
        };
        if display { println!("Stop post document"); }
        Ok(a_ticket)
    }
    pub async fn eas_download_document(&self, file_to_restore: String, display: bool) -> Result<EasResult, reqwest::Error> {

        let request_url = format!("{}/{}","https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents",self.get_ticket_string());
        if display { println!("Start download document"); }
        let auth_bearer = format!("Bearer {}", self.get_token_string());

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
            return Ok(err_to_eas_result(body));
        }


        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }

        // deserialize doc from b64

        let r: Result<EasDocument, Error> = serde_json::from_str(&body);
        let eas_r: EasResult = match r {
            Ok(res) => EasResult::EasDocument(res),
            Err(_e) => EasResult::None
        };
        // Transform base64 => [u8] and save
        if let EasResult::EasDocument(res) = &eas_r {
            let mime_type = &*&res.mimeType;
            let b64_document = &res.base64Document;
            let document = BASE64.decode(b64_document.as_bytes()).unwrap();
            let document_length = document.len();
            let final_document = String::from_utf8(document).unwrap();
            if display { println!("Document: {:#?}", final_document);}
            let mut file = File::create(file_to_restore).unwrap();
            // Write a slice of bytes to the file
            file.write_all(final_document.as_bytes());
            if display { println!("stop get document"); }
            return Ok(EasResult::EasDocument(EasDocument::new((*mime_type).to_string(),format!("Length {}",document_length))));
        }
        Ok(eas_r)
    }
    pub async fn eas_get_document_metadata(&self, display: bool) -> Result<EasResult, reqwest::Error> {
        let request_url = format!("{}/{}/metadata", "https://appdev.cecurity.com/EAS.INTEGRATOR.API/eas/documents", self.get_ticket_string());
        if display { println!("Start retrieve document metadata"); }
        let auth_bearer = format!("Bearer {}", self.get_token_string());

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
            return Ok(err_to_eas_result(body));
        }
        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }
        // deserialize json to metadata

        let r: Result<EasMetaData, Error> = serde_json::from_str(&body);
        let eas_m: EasResult = match r {
            Ok(res) => {
                if display {println!("Deserializing OK.");}
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

}
impl std::fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "credentials: {}\n{}\n{}",
                 self.appId,
                 self.appToken,
                 self.accountName)
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
impl std::fmt::Display for EasMetaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = self.metadata.iter().fold(String::new(), |acc, arg|
            acc + arg.name.as_str() + "->" + arg.value.as_str() + ", ");
        writeln!(f,"[{}]",res)
    }
}

fn err_to_eas_result(body: String) -> EasResult {
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
    r_final
}
pub fn build_static_locations(file_to_archive: &String) -> &str {
    let address = "address1";
    let mut locations = LOCATIONS.lock().unwrap();
    locations.insert(address, string_to_static_str(file_to_archive.to_string()));
    return address;
}

pub fn build_static_locations1 (w: i32, file_to_archive: &String) -> i32 {
    let ad_where = w;
    let mut locations = LOCATIONS2.lock().unwrap();
    locations.insert(ad_where, string_to_static_str(file_to_archive.to_string()));
    return ad_where;
}


pub fn get_inner_token(e : EasResult) -> Option<String> {
    match e {
        EasResult::Token  (t)  => Some(t.get_token().to_string()),
        _ => None
    }
}
pub fn get_inner_ticket(e : EasResult) -> Option<String> {
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
pub fn get_result_status<T>(opt_t : Result<EasResult, T>) -> (EasResult,bool) {
    let (eas_r,status) = match opt_t {
        Ok(EasResult::ApiOk) => {
            (EasResult::ApiOk, true)
        },
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
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
lazy_static! {
    static ref LOCATIONS: Mutex<HashMap<&'static str, &'static str>> =
    Mutex::new(generate_static_locations());
    }
lazy_static! {
    static ref LOCATIONS2: Mutex<HashMap<i32, &'static str>> =
    Mutex::new(generate_static_locations2());
}
fn generate_static_locations() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("default_location", "/Users/bruno/dvlpt/rust/archive.txt");
    m
}
fn generate_static_locations2() -> HashMap<i32, &'static str> {
    let mut  m = HashMap::new();
    m.insert(0, "data0");
    m
}
pub fn compute_digest(path: & str) -> (String,bool) {
    let digest_string : String ;
    if let Ok(input_file) = File::open(path) {
        let reader = BufReader::new(input_file);
        if let Ok(digest) = sha256_digest(reader) {
            digest_string = HEXLOWER.encode(digest.as_ref());
        }
        else {
            println!("Error while digest computation");
            return ("".to_string(),false);
        }
    }
    else {
        println!("Error opening file {}",path);
        return ("".to_string(),false);
    }
    return (digest_string, true);
}

