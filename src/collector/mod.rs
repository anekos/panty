
use std::io::prelude::Read;
use unix_socket::{UnixStream, UnixListener};
use std::thread;

use gvim;
use x;



pub fn start() {
    let listener = UnixListener::bind("stockings").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
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
    x::set_desktop_for_window(39845894, 1);
}


fn fill() {
    gvim::spawn();
}


fn handle_client(mut stream: UnixStream) {
    let mut buf: String = "".to_string();

    match stream.read_to_string(&mut buf) {
        Ok(_) => {
            display_gvim();
            fill()
        }
        Err(err) => println!("Error: {}", err)
    }
}
