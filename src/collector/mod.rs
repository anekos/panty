
use rustc_serialize::json;
use std::collections::LinkedList;
use std::fs::remove_file;
use std::io::prelude::Read;
use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use unix_socket::{UnixListener};
use x11::xlib::Window;

use gvim;
use summoner;


const MAX_STOCKS: usize = 1;


enum Message {
    Summon(String)
}


struct Stock {
    window: Window,
    servername: String
}


pub fn start(socket_filepath: String) {
    let mut current_gvims: LinkedList<Stock> = LinkedList::new();

    let (tx, rx) = channel();
    thread::spawn(|| listener(socket_filepath, tx));

    fill(&mut current_gvims);

    loop {
        use self::Message::*;

        match rx.recv() {
            Ok(Summon(message)) => {
                let param: summoner::Parameter = json::decode(message.as_str()).expect("Fail: json::decode");

                match current_gvims.pop_front() {
                    Some(stock) => summoner::summon(stock.servername, stock.window, param),
                    None => {}
                }

                fill(&mut current_gvims);
            },
            Err(err) => println!("Error: {}", err)
        }
    }
}


fn listener(socket_filepath: String, tx: Sender<Message>) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket");
    }

    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();

                match stream.read_to_string(&mut buf).unwrap() {
                    _ => tx.send(Message::Summon(buf)).unwrap()

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


fn fill(current_gvims: &mut LinkedList<Stock>) {
    let n = MAX_STOCKS - current_gvims.len();
    if n > 0 {
        let names = gvim::new_servernames(n);
        for servername in names {
            current_gvims.push_back(Stock {window: gvim::spawn_in_secret(&servername), servername: servername});
        }
    }
}
