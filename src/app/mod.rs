
use rustc_serialize::json;
use std::collections::LinkedList;
use std::fs::remove_file;
use std::io::prelude::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use unix_socket::{UnixListener};

use summoner;
use police;
use collector;
use executioner;


pub fn start(max_stocks: usize, socket_filepath: String, watch_targets: Vec<String>) {
    let stocks: collector::Stocks = Arc::new(Mutex::new(LinkedList::new()));
    initialize(&socket_filepath);
    collector::collect(stocks.clone(), max_stocks);
    police::patrol(stocks.clone(), max_stocks, watch_targets);
    executioner::watch(stocks.clone());
    listen(stocks.clone(), socket_filepath);
}


fn initialize(socket_filepath: &String) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket");
    }
}


fn listen(stocks: collector::Stocks, socket_filepath: String) {
    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();

                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        let param: summoner::Parameter = json::decode(buf.as_str()).expect("Fail: json::decode");
                        let stock = collector::emit(stocks.clone());
                        summoner::summon(stock.servername, stock.window, param);
                        collector::collect(stocks.clone(), 1);
                    }
                }
            }
            Err(err) => {
                error!("Error: {}", err);
                break;
            }
        }
    }

    drop(listener);
}
