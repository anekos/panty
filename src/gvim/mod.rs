

use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;
use x11::xlib::Window;


pub fn spawn() -> Window {
    let mut child = Command::new("gvim")
        .arg("--nofork")
        .arg("--echo-wid")
        .arg("--role=STOCKING")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");


    let mut echoed = String::new();
    BufReader::new(child.stdout.unwrap()).read_line(&mut echoed).unwrap();

    let wid_str: String = echoed.chars().skip_while(|c| *c != ' ').skip(1).collect();

    wid_str.trim().parse().unwrap()
}
