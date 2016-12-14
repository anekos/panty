
use std::io::prelude::Read;
use unix_socket::{UnixListener};
use std::thread;
use std::sync::mpsc::{Sender, channel};
use std::collections::LinkedList;
use x11::xlib::Window;

use gvim;
use x;


const MAX_STOCKS: usize = 1;


enum Message {
    Summon(String)
}


pub fn start() {
    let mut current_gvims: LinkedList<Window> = LinkedList::new();

    let (tx, rx) = channel();
    thread::spawn(|| listener(tx));

    fill(&mut current_gvims);

    loop {
        use self::Message::*;

        match rx.recv() {
            Ok(Summon(_)) => {
                match current_gvims.pop_front() {
                    Some(win) => display_gvim(win),
                    None => {}
                }
                fill(&mut current_gvims);
            },
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


fn fill(current_gvims: &mut LinkedList<Window>) {
    for _ in current_gvims.len() .. MAX_STOCKS {
        current_gvims.push_back(gvim::spawn_in_secret());
    }
}


fn display_gvim(window: Window) {
    let desktop = x::get_current_desktop() as i64;
    x::set_window_role(window, &"PANTY");
    x::set_desktop_for_window(window, desktop);
}
