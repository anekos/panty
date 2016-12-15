
use rustc_serialize::json;
use std::fs::remove_file;
use std::io::prelude::Read;
use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use unix_socket::{UnixListener};
use x11::xlib::Window;

use gvim;
use summoner;


struct Stock {
    window: Window,
    servername: String
}


pub fn start(max_stocks: usize, socket_filepath: String) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket");
    }

    let (tx, rx) = channel();

    make_stocks(tx.clone(), max_stocks);

    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();

                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        let param: summoner::Parameter = json::decode(buf.as_str()).expect("Fail: json::decode");
                        let stock = rx.recv().unwrap();
                        summoner::summon(stock.servername, stock.window, param.clone());
                        make_stocks(tx.clone(), 1);
                    }
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    drop(listener);
}


fn make_stocks(tx: Sender<Stock>, n: usize) {
    let names = gvim::new_servernames(n);
    for servername in names {
        let ttx = tx.clone();
        thread::spawn(move || {
            ttx.send(Stock {window: gvim::spawn_in_secret(&servername), servername: servername.clone()})
        });
    }
}
