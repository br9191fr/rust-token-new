extern crate easlib;

use easlib::{EasAPI, Credentials};
use easlib::{get_result_status, build_static_locations};

use std::env;
// Result <bool, reqwest::Error >
async fn eas_process(address: i32,path_to_restore: &str, display: bool) ->  Result <bool, reqwest::Error > {
    println!("Step1");
    let credentials = Credentials::new(
        "f33c398c-0f77-4351-9f92-1e20fa3fd2f8".to_owned(),
        "e1320735-e174-4150-9edb-b5daf85be6d1".to_owned(),
        "demoAccount".to_owned()
    );
    let mut api = EasAPI::new(credentials);

    println!("Step authenticate");
    // authenticate and get token
    let opt_t = api.eas_get_token(false).await;
    println!("Step get status");
    let (eas_r,status) = get_result_status(opt_t);
    if !status {
        println!("Failed to get token. End eas process !");
        return Ok(false);
    }
    if display {
        println!("token found {}",api.get_token_string());
    }
    eas_r.show("Get Token");

    // upload document now
    let opt_at = api.eas_post_document(
        address,
        true).await;
    let (eas_r, status) = get_result_status(opt_at);
    if !status {
        println!("Failed to get archive ticket. End eas process !");
        return Ok(false);
    }
    eas_r.show("Upload Doc");
    let opt_cl = api.eas_get_content_list(true).await;
    let (eas_r, status) = get_result_status(opt_cl);

    if !status {
        println!("Failed to get content list. End eas process !");
        return Ok(false);
    }
    eas_r.show("Content list");

    // TODO get contentList with /eas/documents/{ticket}/contentList

    // TODO download individual file with POST to /eas/documents/{ticket}/fileName
    // TODO filename in requestBody (schema downloadItemRequest)

    // TODO play with metadata with /eas/documents/{ticket}/metadata
    // TODO use get/post/patch http commands

    // TODO get matching documents

    // TODO download document
    // TODO Errors found => need corrections
    /*
    let opt_d = api.eas_download_document(
        path_to_restore.clone().to_string(),
        false).await;
    let (eas_r, status) = get_result_status(opt_d);
    if !status {
        println!("Failed to get and restore archive. End eas process !");
        return Ok(false);
    }
    eas_r.show("Download Doc");

    // get document metadata
    let opt_m = api.eas_get_document_metadata(false).await;
    let (eas_r, status) = get_result_status(opt_m);
    if !status {
        println!("Failed to get metadata of archive. End eas process !");
        return Ok(false);
    }
    eas_r.show("Get Metadata");
    */
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

    let address = build_static_locations(1,file_to_archive);
    let test = true;
    if test {
        let final_result = eas_process(
            address,
            file_to_restore,true).await;
        match final_result {
            Ok(true) =>  println!("eas test is ok"),
            Ok(false) => println!("eas test failed"),
            Err(e) => println!("Reqwest error {:#}",e)
        }
    }
    else {
        println!("infos file: {}\n, address: {}\n, restore: {}",
                 file_to_archive, address,file_to_restore);
    }

    println!("end");
}