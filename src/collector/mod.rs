
use std::io::prelude::Read;
use unix_socket::{UnixListener};
use std::thread;
use std::sync::mpsc::{Sender, channel};

use gvim;
use x;


enum Message {
    Summon(String)
}


pub fn start() {
    let (tx, rx) = channel();
    thread::spawn(|| listener(tx));

    loop {
        use self::Message::*;

        match rx.recv() {
            Ok(Summon(_)) => display_gvim(),
            Err(err) => println!("Error: {}", err)
        }
    }
}


fn listener(tx: Sender<Message>) {
    let listener = UnixListener::bind("stockings").unwrap();

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


fn display_gvim() {
    let desktop = x::get_current_desktop() as i64;
    println!("desktop is {}", desktop);
    x::set_desktop_for_window(39845894, desktop);
    gvim::spawn();
}
