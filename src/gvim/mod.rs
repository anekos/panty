
use core::iter::FromIterator;
use std::collections::HashSet;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Command, Stdio, ChildStdout};
use std::thread;
use std::time::Duration;
use x11::xlib::Window;
use libc;

use namer;


#[derive(Clone)]
pub struct SpawnOptions {
    pub current_directory: Option<String>,
    pub command: String,
    pub unmap: bool,
    pub desktop: Option<i64>
}


#[derive(Clone)]
pub struct Instance {
    pub window: Window,
    pub servername: String,
    pub title: String
}


pub const STOCKED_WINDOW_ROLE: &'static str = "STOCKING";
pub const SUMMONED_WINDOW_ROLE: &'static str = "PANTY";


pub fn find_instances(visibility: bool) -> Vec<Instance> {
    with_display!(display => {
        let windows = fetch_all_windows(display);
        let mut result = vec![];

        for window in windows {
            debug!("find_visible_gvim_instances: window = {}", window);
            if window == 0 || is_window_visible(display, window) != visibility {
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


pub fn fetch_window_id(servername: &str) -> Option<Window> {
    let output: Vec<u8> = Command::new("gvim")
        .arg("--servername")
        .arg(servername)
        .arg("--remote-expr")
        .arg("v:windowid")
        .output()
        .unwrap()
        .stdout;

    String::from_utf8(output).ok().and_then(|number| number.trim().parse().ok())
}


pub fn find_instances_without_panty(visibility: bool) -> Vec<Instance> {
    let servernames: Vec<String> = fetch_existing_servernames();

    let join_handles: Vec<_> =
        servernames.iter().map(|servername| {
            let servername = servername.clone();
            thread::spawn(move || {
                with_display!(display => {
                    if let Some(window) = fetch_window_id(&servername) {
                        if is_window_visible(display, window) == visibility {
                            if let Some(title) = get_text_property(display, window, "WM_NAME") {
                                return Some(Instance{
                                    window: window,
                                    servername: servername,
                                    title: title
                                })
                            }
                        }
                    }
                    None
                })
            })
        }).collect();

    {

        let mut result = vec![];

        for handle in join_handles {
            if let Ok(found) = handle.join() {
                if let Some(instance) = found {
                    result.push(instance);
                }
            }
        }

        result
    }
}



pub fn send_files(servername: &str, files: Vec<String>, tab: bool) {
    if files.is_empty() {
        return
    }

    let child = Command::new("gvim")
        .arg("--servername")
        .arg(servername)
        .arg(if tab {"--remote-tab"} else {"--remote"})
        .args(files.as_slice())
        .spawn().unwrap();
    zombie_killer(child.id());
}


pub fn remote(servername: &str, keys: &[String], expressions: &[String], use_output: bool) -> Option<BufReader<ChildStdout>> {
    fn gen_args(name: &str, items: &[String]) -> Vec<String> {
        let buffer: Vec<Vec<String>> = items.iter().map(|it| vec![name.to_string(), it.to_string()]).collect();
        buffer.concat()
    }

    if keys.is_empty() && expressions.is_empty() {
        return None
    }

    let mut command = Command::new("gvim");

    command.arg("--servername")
        .arg(servername)
        .args(gen_args("--remote-send", keys).as_slice())
        .args(gen_args("--remote-expr", expressions).as_slice());

    let (pid, result) = if use_output {
        command.stdout(Stdio::piped());
        let child = command.spawn().unwrap();
        (child.id(), Some(BufReader::new(child.stdout.unwrap())))
    } else {
        (command.spawn().unwrap().id(), None)
    };

    zombie_killer(pid);

    result
}


pub fn spawn(servername: &str, options: &SpawnOptions) -> (Window, BufReader<ChildStdout>) {
    let mut command = Command::new(options.command.clone());

    command.arg("--nofork")
        .arg("--echo-wid")
        .arg(format!("--role={}", STOCKED_WINDOW_ROLE))
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

    zombie_killer(child.id());

    let mut line: String = String::new();
    let stdout = child.stdout.unwrap();
    let mut reader = BufReader::new(stdout);

    for _ in 0..300 {

        if let Ok(len) = reader.read_line(&mut line) {
            if len > 0 {
                if let Some(pos) = line.find("WID: ") {
                    let (_, wid) = line.split_at(pos + 5);
                    return (wid.trim().parse().unwrap(), reader);
                }
                continue;
            }
        }
        break;
    }

    panic!("WID not found!")
}


pub fn spawn_secretly(servername: &str, options: &SpawnOptions) -> (Window, BufReader<ChildStdout>) {
    with_display!(display => {
        let (wid, reader) = spawn(servername, options);

        trace!("spawning: {}", wid);

        while !window_exists(display, wid) {
            thread::sleep(Duration::from_millis(1));
        }

        set_text_property(display, wid, "_PANTY_SERVERNAME", servername);

        if options.unmap || options.desktop.is_some() {
            {
                if wait_for_visible(display, wid, 50) {
                    if let Some(desktop) = options.desktop {
                        set_desktop_for_window(display, wid, desktop);
                    }
                    if options.unmap {
                        unmap_window(display, wid);
                    }
                }
            }
        }

        trace!("spawned: {}", wid);

        (wid, reader)
    })
}


pub fn new_servernames(windows: usize) -> Vec<String> {
    let mut result = vec![];
    let mut names: HashSet<String> = fetch_existing_servernames();

    loop {
        let name = namer::name();
        if !names.contains(&name) {
            result.push(name.clone());
            names.insert(name);
            if result.len() >= windows {
                break;
            }
        }
    }

    result
}


pub fn fetch_existing_servernames<T>() -> T
where T: FromIterator<String> {
    let output: Vec<u8> = Command::new("gvim")
        .arg("--serverlist")
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(output).unwrap().lines().map(|it| it.to_string()).collect()
}


fn zombie_killer(pid: u32) {
    thread::spawn(move || {
        let mut status = 1;
        unsafe { libc::waitpid(pid as i32, &mut status, 0) };
        trace!("process done: pid = {}", pid);
    });
}
