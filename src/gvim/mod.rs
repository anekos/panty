

use std::collections::HashSet;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use x11::xlib::Window;

use x;


pub fn send_files(servername: &String, files: Vec<&str>) {
    Command::new("gvim")
        .arg("--servername")
        .arg(servername)
        .arg("--remote-tab")
        .args(files.as_slice())
        .spawn().unwrap();
}


pub fn spawn(servername: &String) -> Window {
    let child = Command::new("gvim")
        .arg("--nofork")
        .arg("--echo-wid")
        .arg("--role=STOCKING")
        .arg("-iconic")
        .arg("--servername")
        .arg(servername)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    let mut echoed = String::new();
    BufReader::new(child.stdout.unwrap()).read_line(&mut echoed).unwrap();

    let wid_str: String = echoed.chars().skip_while(|c| *c != ' ').skip(1).collect();

    wid_str.trim().parse().unwrap()
}


pub fn spawn_in_secret(servername: &String) -> Window {
    let wid = spawn(servername);

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


pub fn new_servernames(windows: usize) -> Vec<String> {
    let output: Vec<u8> = Command::new("gvim")
        .arg("--serverlist")
        .output()
        .unwrap()
        .stdout;

    String::from_utf8(output).map(|s| -> Vec<String> {
        let mut result = vec![];

        let mut names = HashSet::new();
        for name in s.lines() {
            names.insert(name);
        }

        let mut id = 1;
        loop {
            id += 1;
            let name = make_servername(id);
            if !names.contains(&name.as_str()) {
                result.push(name);
                if result.len() >= windows {
                    break;
                }
            }
        }

        result
    }).unwrap()
}


fn make_servername(id: u64) -> String {
    let mut servername: String = "STOCKING-".to_string();
    servername.push_str(id.to_string().as_str());
    servername
}
