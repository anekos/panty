

use std::io::BufReader;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use x11::xlib::Window;

use x;


pub fn spawn() -> Window {
    let child = Command::new("gvim")
        .arg("--nofork")
        .arg("--echo-wid")
        .arg("--role=STOCKING")
        .arg("-iconic")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    let mut echoed = String::new();
    BufReader::new(child.stdout.unwrap()).read_line(&mut echoed).unwrap();

    let wid_str: String = echoed.chars().skip_while(|c| *c != ' ').skip(1).collect();

    wid_str.trim().parse().unwrap()
}


pub fn spawn_in_secret() -> Window {
    let wid = spawn();

    println!("spawn_in_secret: {}", wid);

    while !x::window_exists(wid) {
        thread::sleep(Duration::from_millis(1));
    }

    {
        let max_tries = 50;
        let mut tried = 0;

        while !x::is_window_visible(wid) && tried < max_tries {
            tried += 1;
            thread::sleep(Duration::from_millis(1));
        }

        if tried < max_tries {
            x::unmap_window(wid);
        }
    }

    println!("spawn_in_secret: unmapped");

    wid
}
