extern crate reqwest;
extern crate serde_json;
extern crate error_chain;

use std::env;
use std::fmt;

use reqwest::{Client};
use serde_json::{json,Error};
use serde_json::Result as serde_result;
use serde::{Deserialize,Serialize};



#[derive(Serialize,Deserialize,Debug)]
pub struct Message {
    text: String,
    sender: String,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct PhoneNumber {
    value: String,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct Recipient {
    gsm: Vec<PhoneNumber>,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct Sms {
    message: Message,
    recipients: Recipient,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct FullSms {
    sms: Sms,
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sender -> {}, text -> {}", self.sender, self.text)
    }
}
impl std::fmt::Display for Recipient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = self.gsm.iter().fold(String::new(), |acc, arg|
            acc + "value ->" + arg.value.as_str() + ", ");
        write!(f, "gsm -> [{}]", res)
    }
}
impl std::fmt::Display for Sms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sms -> message:<{}>, recipients: <{}> ", self.message,self.recipients)
    }
}
impl std::fmt::Display for FullSms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sms)
    }
}
/*
Test de base appel http pour récupérer des data JSON
 */

async fn get_data(info: &str) -> Result<(), reqwest::Error> {
    let request_url = format!("http://192.168.0.42:3000/data?what=OkBruno&why={}",info
    );
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
Connexion à un service d'envoi de SMS via appel de web service
 */
//TODO Add additional functions to dynamically set recipients and message
fn build_sms() -> FullSms {
    let message = Message {text: "Hello Bruno Rust test2 !".to_string(),sender: "Rust Demo".to_string() };
    let phone1 = PhoneNumber {value: "33622696747".to_string()};
    let phone2 = PhoneNumber {value: "33601453037".to_string()};
    let v_phone = vec![phone1,phone2];
    let v_rec= Recipient {gsm: v_phone};
    let sms = Sms {message: message, recipients: v_rec};
    let fullSms = FullSms {sms: sms};
    println!("Build_sms -> {}",fullSms);
    fullSms
}
fn get_sms_string(sms : &FullSms) -> String {
    let sms_string = serde_json::to_string(&sms);
    let ss : String = match sms_string {
        Ok(s )=> s,
        Err(e) => "nothing".to_string()
    };
    ss
}
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
    let sms = build_sms();
    println!("payload -> {}", payload.to_string());
    println!("sms1     -> {}", get_sms_string(&sms));
    let sms_string = get_sms_string(&sms);
    println!("sms2     -> {:#?}", sms_string);
    let request_url = "https://api.smsfactor.com/send";

    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI0MTQ3MSIsImlhdCI6MTYxNTU1OTM5Nn0.MHGpHL7UQa8VmXJtQ6xBSP6At19uzc2_fVJwVQsVvO0"; //Your first token must be generated on our plateform at https://secure.smsfactor.com/token.html
    println!("start");
    /*
    let response = Client::new()
        .post(request_url)
        .header("Accept", "application/json")
        .header("Authorization", format!("{}{}", "Bearer ", token))
        .json(&payload)
        .send().await?;
    */
    let response2 = Client::new()
        .post(request_url)
        .header("Accept", "application/json")
        .header("Authorization", format!("{}{}", "Bearer ", token))
        .json(&sms)
        .send().await?;
    let sc = response2.status();
    let body = response2.text().await.unwrap();

    println!("Body : {:#?}", body);
    println!("Status : {:#?}", sc);
    println!("stop");
    Ok(())
}
fn compose<A,B,C,F,G>(f: F, g: G) -> impl Fn(A) -> C
    where F: 'static + Fn(A) -> B,
          G: 'static + Fn(B) -> C {
    move |x| g(f(x))
}
fn run_compose () {
    let fa = |x| x+1;
    let fb = |y| y*2;
    let fc = |z| z/3;
    let h = compose(fa, fb);
    let g = compose(compose(fa,fb),fc);
    let x= 5;
    println!("h(5) = {}",h(5));
    println!("g(x) = {}",g(x));
    println!("x = {}",x);
    println!("g(1) = {}", g(1));
    println!("g(12) = {}", g(12));
    println!("g(123) = {}", g(123));
}

#[tokio::main] async
fn main() {
    let args: Vec<String> = env::args().collect();
    let param: &str;
    if (args.len() >1 ) {
        param = &args[1];
    }
    else {
        param = "Nothing";
    }
    //send_sms().await;

    get_data(param).await;
    //run_compose();
    println!("end");
}
#[cfg(test)]
mod tests {
    use data_encoding::{BASE64};
    use error_chain::{error_chain};
    use std::fs::File;
    use std::io::Read;
    use serde_json::{json, Error};

    error_chain! {
    foreign_links {
        Io(std::io::Error);
        ParseInt(::std::num::ParseIntError);
        }
    }
    fn read_uptime() -> Result<u64> {
        let mut uptime = String::new();
        File::open("/proc/xxxuptime")?.read_to_string(&mut uptime)?;

        Ok(uptime
            .split('.')
            .next()
            .ok_or("Cannot parse uptime data")?
            .parse()?)
    }

    #[test]
    fn test_base64_encoding() {
        assert_eq!(BASE64.encode(b"Hello world"), "SGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_base64_decoding() {
        //
        //assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQK").unwrap(), b"Hello world");
        assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQ=").unwrap(), b"Hello world");
        assert_eq!(BASE64.decode(b"SGVsbA==byB3b3JsZA==").unwrap(), b"Hello world");
    }

    #[test]
    fn test_base64_decoding2() {
        let mut buffer = [0u8; 64];
        let input = b"SGVsbA==byB3b3JsZA==";
        let len = BASE64.decode_len(input.len()).unwrap();
        println!("len is {}", len);
        assert_eq!(len, 15);
        let output = &mut buffer[0..BASE64.decode_len(input.len()).unwrap()];
        let len = BASE64.decode_mut(input, output).unwrap();
        assert_eq!(&output[0..len], b"Hello world");
        //assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQ=").unwrap(), b"Hello world");
    }

    #[test]
    fn test_uptime() {
        match read_uptime() {
            Ok(uptime) => println!("uptime: {} seconds", uptime),
            Err(err) => eprintln!("error: {}", err),
        };
    }
    #[test]
    fn test_serde_json() {
        let val = serde_json::to_value("\"A_30006C88E2D54A2D9FFF5FBD9F25BBE8_1\"".to_string());
    }
}