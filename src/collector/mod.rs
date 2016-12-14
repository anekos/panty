
use std::fs::remove_file;
use std::path::Path;
use std::io::prelude::Read;
use unix_socket::{UnixListener};
use std::thread;
use std::sync::mpsc::{Sender, channel};
use std::collections::LinkedList;
use x11::xlib::Window;

use gvim;
use x;


const MAX_STOCKS: usize = 1;
const SOCK_FILENAME: &'static str = "stockings";


enum Message {
    Summon(String)
}


struct Stock {
    window: Window,
    servername: String
}


pub fn start() {
    let mut current_gvims: LinkedList<Stock> = LinkedList::new();

    let (tx, rx) = channel();
    thread::spawn(|| listener(tx));

    fill(&mut current_gvims);

    loop {
        use self::Message::*;

        match rx.recv() {
            Ok(Summon(message)) => {
                let files: Vec<&str> = message.lines().collect();

                match current_gvims.pop_front() {
                    Some(stock) => display_gvim(stock, files),
                    None => {}
                }

                fill(&mut current_gvims);
            },
            Err(err) => println!("Error: {}", err)
        }
    }
}


fn listener(tx: Sender<Message>) {
    if Path::new(SOCK_FILENAME).exists() {
        remove_file(SOCK_FILENAME).expect("Faild: remove socket");
    }

    let listener = UnixListener::bind(SOCK_FILENAME).unwrap();

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


fn display_gvim(stock: Stock, files: Vec<&str>) {
    let desktop = x::get_current_desktop() as i64;

    println!("display_gvim: window = {}, desktop = {}, servername = {}", stock.window, desktop, stock.servername);

    x::set_window_role(stock.window, &"PANTY");
    x::map_window(stock.window);
    x::set_desktop_for_window(stock.window, desktop);

    gvim::send_files(&stock.servername, files);
}
