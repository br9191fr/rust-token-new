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
use ring::digest::{Context, Digest, SHA256};
use serde_json::{json, Error};
use serde::{Deserialize, Serialize};
use tokio::fs::File as Tokio_File;
use tokio::io::{self, AsyncReadExt};
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
            appId: id,
            appToken: token,
            accountName: name,
        }
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Ticket {
    ticket: String,
}

impl Ticket {
    fn get_ticket(&self) -> &String {
        let string = &self.ticket;
        string
    }
    fn new(ticket: String) -> Self {
        Ticket { ticket }
    }
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    errorCode : String,
    errorMessage : String,
    status : String,
}
impl ErrorResponse {
    fn get_error_code(&self) -> &String {
        let string = &self.errorCode;
        string
    }
    fn get_error_message(&self) -> &String {
        let string = &self.errorMessage;
        string
    }
    fn get_status(&self) -> &String {
        let string = &self.status;
        string
    }
    fn new(error_code: String, error_message: String, status: String) -> Self {
        ErrorResponse {errorCode: error_code, errorMessage: error_message, status: status}
    }
}
#[derive(Deserialize, Debug)]
pub struct EasError {
    message: String,
}

#[derive(Deserialize, Debug)]
pub enum EasResult {
    Token(Token),
    Ticket(Ticket),
    ErrorResponse(ErrorResponse),
    EasDocument(EasDocument),
    EasMetaData(EasMetaData),
    EasError(EasError),
    ApiOk,
    None,
}

impl EasResult {

    fn get_ticket(&self) -> Option<&String> {
        if let EasResult::Ticket(at) = self {
            Some(at.get_ticket())
        } else {
            None
        }
    }
    fn get_token(&self) -> Option<&String> {
        if let EasResult::Token(t) = self {
            Some(t.get_token())
        } else {
            None
        }
    }
    pub fn show(&self, msg: &str) {
        match &*self {
            EasResult::Token(t) => println!("[{}] Token: {}", msg, t),
            EasResult::Ticket(at) => println!("[{}] Ticket: {}", msg, at),
            EasResult::EasDocument(d) => println!("[{}] Document: {}", msg, d),
            EasResult::EasMetaData(m) => println!("[{}] MetaData: {}", msg, m),
            EasResult::ApiOk => println!("[{}] API Called OK", msg),
            _ => println!("[{}] Unknown or Not implemented", msg)
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
    fn new(token: String, filename: String, address: String, digest: String) -> Self {
        EasInfo {
            token,
            filename,
            address,
            digest,
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

#[derive(Deserialize, Debug)]
pub struct EasDocument {
    mimeType: String,
    base64Document: String,
}

impl EasDocument {
    fn new(mime_type: String, base64_document: String) -> Self {
        EasDocument {
            mimeType: mime_type,
            base64Document: base64_document,
        }
    }
}

pub struct EasAPI {
    credentials: Credentials,
    token: Option<Token>,
    digest: Option<String>,
    ticket: Option<Ticket>,
    error_response: Option<ErrorResponse>,
}

impl EasAPI {
    pub fn new(credentials: Credentials) -> Self {
        EasAPI { credentials: credentials, token: None, digest: None, ticket: None, error_response: None }
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
        self.ticket.as_ref().unwrap().get_ticket()
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
    pub async fn eas_get_token(&mut self, display: bool) -> Result<EasResult, reqwest::Error> {
        let request_url = "https://apprec.cecurity.com/eas.integrator.api/service/authenticate";
        if display { println!("Start get token"); }
        let cred_value = serde_json::to_value(&self.credentials).unwrap();
        let response = Client::new()
            .post(request_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&cred_value)
            .send().await?;
        if display { println!("wait for answer"); }
        let sc = response.status();
        if display {
            let headers = response.headers();
            for (key, value) in headers.iter() {
                println!("{:?}: {:?}", key, value);
            }
        }
        if display { println!("Decoding body"); }
        let body = response.text().await.unwrap();
        if !sc.is_success() {
            println!("Request failed => {}", sc);
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
            }
            Err(_e) => EasResult::None
        };
        if display { println!("stop get token"); }
        Ok(t)
    }
    // TODO add function to check result
    pub async fn file_as_part(&self, address: i32, mime_type : &str) -> Result<reqwest::multipart::Part, Box<dyn std::error::Error>> {
        let my_ref1 = LOCATIONS.lock().unwrap();
        let address = my_ref1.get(&address);
        let fname = match address {
            Some(f) => {
                f
            }
            _ => {
                "/Users/bruno/dvlpt/rust/archive.txt"
            }
        };
        let mut async_buffer = Vec::new();
        let path = Path::new(fname);
        let mut file = Tokio_File::open(path).await?;
        let _fcl = file.read_to_end(&mut async_buffer).await?;
        let file_content = str::from_utf8(&*async_buffer).unwrap().to_string();
        let file_part = reqwest::multipart::Part::text(file_content)
            .file_name(path.file_name().unwrap().to_string_lossy())
            .mime_str(mime_type).unwrap();
        Ok(file_part)
    }
    pub async fn eas_post_document(&mut self, address: i32, display: bool) -> Result<EasResult, Box<dyn std::error::Error>> {
        let request_url = "https://apprec.cecurity.com/eas.integrator.api/eas/documents";
        let request_url2 = "https://enkzri5ybwfri60.m.pipedream.net";
        if display { println!("Start post document"); }
        let auth_bearer = format!("Bearer {}", self.get_token_string());

        // compute digest of file 1
        let my_ref1 = LOCATIONS.lock().unwrap();
        let address_str = my_ref1.get(&address);
        let fname = match address_str {
            Some(f) => {
                if display { println!("ok nice use f == {}", f); }
                f
            }

            _ => {
                println!("ko use default value");
                "/Users/bruno/dvlpt/rust/archive.txt"
            }
        };
        let (digest_string, status) = compute_digest(fname);
        if !status { return Ok(EasResult::None); }
        self.set_digest(digest_string.clone());
        if display {
            println!("SHA256 Digest for {} is {}", fname, self.digest.as_ref().unwrap().clone());
        }

        // compute digest of file 2
        let fname2 = "/users/bruno/dvlpt/rust/archive1.txt";
        let (digest_string2, status2) = compute_digest(fname2);
        if !status2 { return Ok(EasResult::None); }
        if display {
            println!("SHA256 Digest for {} is {}", "/users/bruno/dvlpt/rust/archive1.txt", digest_string2);
        }

        let meta = json!([
            {"name": "ClientId", "value": "987654321"},
            {"name": "CustomerId", "value": "CLIENT-BRI"},
            {"name": "Documenttype", "value": "Validated invoice"}]);

        let upload_file_fingerprint = json!([
            {"fileName": fname, "value" : digest_string.clone(),"fingerPrintAlgorithm": "SHA-256"},
            {"fileName": "/users/bruno/dvlpt/rust/archive1.txt", "value" : digest_string2.clone(),"fingerPrintAlgorithm" : "SHA-256"}
        ]);
        //             .text("fingerPrint",self.digest.as_ref().unwrap().clone())
        //             .text("fingerprintAlgorithm","SHA-256")
        //let file_part_async = self.file_as_part(address,"application/octet-stream").await;

        // part for first file
        let mut async_buffer = Vec::new();
        let path = Path::new(fname);
        let mut file = Tokio_File::open(path).await?;
        let _fcl = file.read_to_end(&mut async_buffer).await?;
        let file_content = str::from_utf8(&*async_buffer).unwrap().to_string();
        let file_part_async = reqwest::multipart::Part::text(file_content)
            .file_name(path.file_name().unwrap().to_string_lossy())
            .mime_str("application/octet-stream").unwrap();

        // part for second file
        let mut sync_buffer = Vec::new();
        let path1 = Path::new("/users/bruno/dvlpt/rust/archive1.txt");
        let mut file1 = File::open(path1).unwrap();
        let _fcl = file1.read_to_end(&mut sync_buffer);
        let file_content = str::from_utf8(&*sync_buffer).unwrap().to_string();
        let file_part_sync2 = reqwest::multipart::Part::text(file_content)
            .file_name(path1.file_name().unwrap().to_string_lossy())
            .mime_str("application/octet-stream").unwrap();

        let form = Form::new()
            .part("document", file_part_async)
            .part("document", file_part_sync2)
            .text("metadata", meta.to_string())
            .text("fingerPrints", upload_file_fingerprint.to_string());

        let response = Client::new()
            .post(request_url)
            .header("Authorization", auth_bearer)
            .header("Accept", "application/json")
            .multipart(form)
            .send()
            .await?;
        let sc = response.status();
        let display2 = false;
        println!("Status code {} => {} {}",sc.as_u16() , sc.is_success(),sc.is_client_error());
        if display2 {
            let headers = response.headers();
            for (key, value) in headers.iter() {
                println!("{:?}: {:?}", key, value);
            }
        }
        let body = response.text().await.unwrap();
        if !sc.is_success() {
            println!("Request failed => {} {}", sc, &body);
            let er: Result<ErrorResponse, Error> = serde_json::from_str(&body);
            let a_er: EasResult = match er {
                Ok(res) => {
                    self.error_response = Some(res);
                    if display { println!("Body contains error_response"); }
                    EasResult::None
                }
                Err(e) => {
                    if display {
                        println!("Unable to deserialize body => ErrorResponse\nError {}", e);
                    };
                    EasResult::None
                }
            };
            return Ok(err_to_eas_result(body));
        }

        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }
        // Extract ticket

        let r: Result<Ticket, Error> = serde_json::from_str(&body);
        let a_ticket: EasResult = match r {
            Ok(res) => {
                self.ticket = Some(res);
                if display { println!("Body contains ticket"); }
                EasResult::ApiOk
            }
            Err(e) => {
                if display {
                    println!("Unable to deserialize body => ticket\nError {}", e);
                };
                EasResult::None
            }
        };
        if display { println!("Stop post document"); }
        Ok(a_ticket)
    }
    pub async fn eas_get_content_list(&self,display: bool) -> Result<EasResult, reqwest::Error> {
        let request_url = format!("https://apprec.cecurity.com/eas.integrator.api/eas/documents/{}/contentList",self.get_ticket_string());
        if display { println!("Start get content list"); }
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
            println!("Request failed => {}", sc);
            return Ok(err_to_eas_result(body));
        }
        if display {
            println!("Status : {:#?}\n{:#?}", sc, body);
        }
        let r: Result<Vec<String>, Error> = serde_json::from_str(&body);
        let eas_r: EasResult = match r {
            Ok(res) => {
                println!("Found {} documents",res.len());
                for st in &res {
                    println!("Found {}",st);
                }
                EasResult::ApiOk
            },
            Err(_e) => EasResult::None
        };
        Ok(eas_r)
    }
    pub async fn eas_download_document(&self, file_to_restore: String, display: bool) -> Result<EasResult, reqwest::Error> {
        let request_url = format!("{}/{}", "https://apprec.cecurity.com/eas.integrator.api/eas/documents", self.get_ticket_string());
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
            println!("Request failed => {}", sc);
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
            if display { println!("Document: {:#?}", final_document); }
            let mut file = File::create(file_to_restore).unwrap();
            // Write a slice of bytes to the file
            let final_result = match file.write_all(final_document.as_bytes()) {
                Ok(r1) => true,
                Err(_e) => false
            };
            if (final_result) {
                if display { println!("stop get document"); }
                return Ok(EasResult::EasDocument(EasDocument::new((*mime_type).to_string(), format!("Length {}", document_length))));
            }
        }
        Ok(eas_r)
    }
    pub async fn eas_get_document_metadata(&self, display: bool) -> Result<EasResult, reqwest::Error> {
        let request_url = format!("{}/{}/metadata", "https://apprec.cecurity.com/eas.integrator.api/eas/documents", self.get_ticket_string());
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
                if display { println!("Deserializing OK."); }
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

impl std::fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Ticket: {}", self.ticket)
    }
}
impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error Response: {} {} {} ", self.errorCode,self.errorMessage, self.status)
    }
}
impl std::fmt::Display for EasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message: {}", self.message)
    }
}

impl std::fmt::Display for EasDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "token: {:#?},{:#?}", self.mimeType, self.base64Document)
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
        writeln!(f, "[{}]", res)
    }
}

fn err_to_eas_result(body: String) -> EasResult {
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
    r_final
}


pub fn build_static_locations(w: i32, file_to_archive: &String) -> i32 {
    let ad_where = w;
    let mut locations = LOCATIONS.lock().unwrap();
    locations.insert(ad_where, string_to_static_str(file_to_archive.to_string()));
    return ad_where;
}


pub fn get_inner_token(e: EasResult) -> Option<String> {
    match e {
        EasResult::Token(t) => Some(t.get_token().to_string()),
        _ => None
    }
}

pub fn get_inner_ticket(e: EasResult) -> Option<String> {
    match e {
        EasResult::Ticket(a) => Some(a.get_ticket().to_string()),
        _ => None
    }
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, Error> {
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

pub fn get_result_status<T>(opt_t: Result<EasResult, T>) -> (EasResult, bool) {
    let (eas_r, status) = match opt_t {
        Ok(EasResult::ApiOk) => {
            (EasResult::ApiOk, true)
        }
        Ok(EasResult::Token(t)) => {
            (EasResult::Token(t), true)
        }
        Ok(EasResult::Ticket(a)) => {
            (EasResult::Ticket(a), true)
        }
        Ok(EasResult::EasDocument(d)) => {
            (EasResult::EasDocument(d), true)
        }
        Ok(EasResult::EasMetaData(m)) => {
            (EasResult::EasMetaData(m), true)
        }
        Ok(EasResult::EasError(eas)) => {
            println!("eas error found {}", eas);
            (EasResult::EasError(eas), false)
        }
        _ => {
            println!("Error while operating.");
            (EasResult::None, false)
        }
    };
    (eas_r, status)
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

lazy_static! {
    static ref LOCATIONS: Mutex<HashMap<i32, &'static str>> =
    Mutex::new(generate_static_locations());
}

fn generate_static_locations() -> HashMap<i32, &'static str> {
    let mut m = HashMap::new();
    m.insert(0, "data0");
    m
}

pub fn compute_digest(path: &str) -> (String, bool) {
    let digest_string: String;
    if let Ok(input_file) = File::open(path) {
        let reader = BufReader::new(input_file);
        if let Ok(digest) = sha256_digest(reader) {
            digest_string = HEXLOWER.encode(digest.as_ref());
        } else {
            println!("Error while digest computation");
            return ("".to_string(), false);
        }
    } else {
        println!("Error opening file {}", path);
        return ("".to_string(), false);
    }
    return (digest_string, true);
}

