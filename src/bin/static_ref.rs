use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
lazy_static! {
    static ref LOCATIONS: Mutex<HashMap<&'static str, &'static str>> =
    Mutex::new(generate_static_locations());
}
fn generate_static_locations() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("default_location", "/Users/bruno/dvlpt/rust/archive.txt");
    m
}
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        println!("Missing arguments\nUsage: Static_ref filename");
        return;
    }
    let filename = &args[1];
    {
        let mut locations = LOCATIONS.lock().unwrap();
        locations.insert("address1","/Users/bruno/dvlpt/rust/archive.txt");
        locations.insert("address2",string_to_static_str(filename.to_string()));
    }
}