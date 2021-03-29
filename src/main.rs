extern crate easlib;

use easlib::{compute_digest, eas_get_token, eas_post_document, eas_download_document, eas_get_document_metadata};
use easlib::{get_result_status, get_inner_token, get_inner_ticket, build_static_locations};
use easlib::{EasInfo};
use std::env;





async fn eas_process(path_to_archive: &str, address: &str, path_to_restore: &str) -> Result<bool, reqwest::Error > {
    let token_string;
    let archive_ticket : String ;
    // compute digest of file

    let (digest_string, status) = compute_digest(path_to_archive);
    if !status  {return Ok(false);}
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
    //let eas_info = EasInfo::new(token_string.clone(),path_to_archive.to_string(), address.to_string(),digest_string);
    let opt_at = eas_post_document(token_string.clone(),address,true).await;
    let (eas_r, status) = get_result_status(opt_at);
    if !status {
        println!("Failed to get archive ticket. End eas process !");
        return Ok(false);
    }
    eas_r.show();
    archive_ticket = get_inner_ticket(eas_r).unwrap();

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

    let address = build_static_locations(file_to_archive);

    let final_result = eas_process(file_to_archive, address,file_to_restore).await;
    match final_result {
        Ok(true) =>  println!("eas test is ok"),
        Ok(false) => println!("eas test failed"),
        Err(e) => println!("Reqwest error {:#}",e)
    }
    println!("end");
}