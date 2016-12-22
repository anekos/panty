
use std::collections::HashSet;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use x11::xlib::Window;

use namer;


#[derive(Clone)]
pub struct Options {
    pub current_directory: Option<String>,
    pub command: String,
    pub unmap: bool
}


pub struct Instance {
    pub window: Window,
    pub servername: String,
    pub title: String
}


pub fn find_visible_instances() -> Vec<Instance> {
    with_display!(display => {
        let windows = fetch_all_windows(display);
        let mut result = vec![];

        for window in windows {
            debug!("find_visible_gvim_instances: window = {}", window);
            if window == 0 || !is_window_visible(display, window) {
                continue;
            }

            if let Some(class) = get_window_class(display, window) {
                if class.as_str() == "Gvim" {
                    if let Some(servername) = get_text_property(display, window, "_PANTY_SERVERNAME") {
                        if let Some(title) = get_text_property(display, window, "WM_NAME") {
                            result.push(Instance {
                                window: window,
                                servername: servername.clone(),
                                title: title
                            });
                        }
                    }
                }
            }
        }

        result
    })
}



pub fn send_files(servername: &String, files: Vec<String>, tab: bool) {
    if files.is_empty() {
        return
    }

    Command::new("gvim")
        .arg("--servername")
        .arg(servername)
        .arg(if tab {"--remote-tab"} else {"--remote"})
        .args(files.as_slice())
        .spawn().unwrap();
}


pub fn spawn(servername: &String, options: &Options) -> Window {
    let mut command = Command::new(options.command.clone());

    command.arg("--nofork")
        .arg("--echo-wid")
        .arg("--role=STOCKING")
        .arg("-iconic")
        .arg("--servername")
        .arg(servername);

    if let Some(current_directory) = options.current_directory.clone() {
        command.current_dir(current_directory);
    }

    let child = command
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    let mut line: String = String::new();
    let mut reader = BufReader::new(child.stdout.unwrap());

    for _ in 0..300 {

        if let Ok(len) = reader.read_line(&mut line) {
            if len > 0 {
                if let Some(pos) = line.find("WID: ") {
                    let (_, wid) = line.split_at(pos + 5);
                    return wid.trim().parse().unwrap();
                }
                continue;
            }
        }
        break;
    }

    panic!("WID not found!")
}


pub fn spawn_secretly(servername: &String, options: &Options) -> Window {
    with_display!(display => {
        let wid = spawn(servername, options);

        trace!("spawning: {}", wid);

        while !window_exists(display, wid) {
            thread::sleep(Duration::from_millis(1));
        }

        set_text_property(display, wid, "_PANTY_SERVERNAME", servername.as_str());

        if options.unmap {
            {
                let max_tries = 50;
                let mut tried = 0;

                while !is_window_visible(display, wid) && tried < max_tries {
                    tried += 1;
                    thread::sleep(Duration::from_millis(1));
                }

                if tried < max_tries {
                    // TODO?? set_desktop_for_window(display, wid, 5);
                    unmap_window(display, wid);
                }
            }
        }

        trace!("spawned: {}", wid);

        wid
    })
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

        loop {
            let name = namer::name();
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
