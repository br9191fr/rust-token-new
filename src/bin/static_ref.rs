use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

lazy_static! {
    static ref LOCATIONS: Mutex<HashMap<usize, &'static str>> =
    Mutex::new(generate_static_locations());
}
fn generate_static_locations() -> HashMap<usize, &'static str> {
    let mut m = HashMap::new();
    m.insert(0, "default value");
    m
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing arguments\nUsage: Static_ref filename");
        return;
    }
    let mut k = 0;
    let mut locations = LOCATIONS.lock().unwrap();
    let mut k = 0;
    for st in &args {
        if (k > 0) {
            locations.insert(k, string_to_static_str(st.to_string()));
            println!("Insert {} at position {}", &st, k);
        }
        k += 1;
    }
    println!(" insert OK");
    let my_ref = locations;
    for k in 0..args.len() {
        let _add_ok = match my_ref.get(&k) {
            Some(f) => {
                println!("At {} => {}",k, f);
            }
            _ => {
                println!("{} => Bad Address",k);
            }
        };
    }
}