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
#[cfg(test)]
mod tests {
    use data_encoding::{HEXLOWER, BASE64};
    use error_chain::{error_chain};
    use std::fs::File;
    use std::io::Read;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt; // for read_to_end()

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
    async fn test_readasync() {
        let mut file = File::open("foo.txt").await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        println!("len = {}", contents.len());
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
}