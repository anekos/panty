
extern crate panty;

use std::env;
use panty::*;



fn show_help() {
    println!("panty collector");
    println!("panty summon")
}


fn main() {
    match env::args().nth(1) {
        Some(sub) => match sub.as_str() {
            "collector" => collector::start(),
            "summon" => summoner::summon(),
            _ => show_help()
        },
        None => show_help()
    }
}
