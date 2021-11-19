extern crate easlib;

use easlib::easlib::{EasAPI};
use easlib::easlib::{get_result_status, build_static_locations};
use easlib::bri_cred::{get_credentials};
use std::env;

async fn eas_process(address: i32, display: bool) ->  Result <bool, reqwest::Error > {
    //let credentials = Credentials::new("xxxxx".to_owned(),"tttt".to_owned(),"myAccount".to_owned());
    let credentials_ok = get_credentials();
    let mut api = EasAPI::new(credentials_ok);

    if display { println!("Step authenticate"); }
    // authenticate and get token
    let opt_t = api.eas_get_token(false).await;
    if display { println!("Step get status"); }
    let (eas_r,status) = get_result_status(opt_t);
    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    if display { println!("token found {}",api.get_token_string()); }
    eas_r.show("Get Token");

    // upload document now
    let opt_at = api.eas_post_document(
        address,
        false).await;
    let (eas_r, status) = get_result_status(opt_at);
    if !status {
        println!("Failed to get archive ticket. End eas process !");
        return Ok(false);
    }
    eas_r.show("Upload Doc");
    let opt_cl = api.eas_get_content_list(false).await;
    let (eas_r, status) = get_result_status(opt_cl);

    if !status {
        println!("Failed to get content list. End eas process !");
        return Ok(false);
    }
    eas_r.show("Content list");

    let opt_ar = api.eas_get_archive(false).await;
    let (eas_r, status) = get_result_status(opt_ar);

    if !status {
        println!("Failed to get full archive. End eas process !");
        return Ok(false);
    }
    eas_r.show("Archive Info");

    // TODO download individual file with POST to /eas/documents/{ticket}/fileName
    // TODO filename in requestBody (schema downloadItemRequest)

    // TODO play with metadata with /eas/documents/{ticket}/metadata
    // TODO use get/post/patch http commands

    // TODO get matching documents

    return Ok(true);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        println!("Missing arguments\nUsage: pgm file_to_archive");
        return;
    }
    let file_to_archive = &args[1];

    let address = build_static_locations(1,file_to_archive);
    let test = true;
    if test {
        let final_result = eas_process(
            address,false).await;
        match final_result {
            Ok(true) =>  println!("eas test is ok"),
            Ok(false) => println!("eas test failed"),
            Err(e) => println!("Reqwest error {:#}",e)
        }
    }
    else {
        println!("infos file: {}\n, address: {}",
                 file_to_archive, address);
    }

    println!("end");
}