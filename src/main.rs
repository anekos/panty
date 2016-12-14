
extern crate panty;

use std::env::args;
use panty::*;



fn show_help() {
    println!("panty collector");
    println!("panty summon")
}


fn main() {
    match args().nth(1) {
        Some(sub) => match sub.as_str() {
            "collector" => collector::start(),
            "summon" => summoner::summon(args().skip(2).collect()),
            _ => show_help()
        },
        None => show_help()
    }
}
