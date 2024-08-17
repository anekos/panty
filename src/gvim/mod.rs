
extern crate regex;

use core::iter::FromIterator;
use libc;
use std::collections::HashSet;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Command, Stdio, ChildStdout, ChildStderr};
use std::thread;
use std::time::Duration;
use x11::xlib::Window;

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


#[derive(Clone)]
pub struct SendFileOptions<'a> {
    pub change_directory: bool,
    pub envs: &'a[(String, String)],
    pub files: &'a[&'a str],
    pub servername: &'a str,
    pub tab: bool,
    pub working_directory: &'a str,
}


pub const STOCKED_WINDOW_ROLE: &str = "STOCKING";
pub const SUMMONED_WINDOW_ROLE: &str = "PANTY";


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
                                servername: servername.clone(),
                                title,
                                window,
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
                                return Some(Instance { window, servername, title })
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
            if let Ok(Some(instance)) = handle.join() {
                result.push(instance);
            }
        }

        result
    }
}



pub fn send_files(options: SendFileOptions) {
    let mut child = Command::new("gvim");
    child.current_dir(options.working_directory);

    fn attach_envs(cmd: &mut Command, envs: &[(String, String)]) {
        for chunk in envs.chunks(10) {
            cmd.arg("--remote-expr");
            let mut buf = "0".to_owned();
            for (key, value) in chunk {
                buf.push_str(&format!("+setenv('{}', '{}')", key, escape_str_in_expression(value)));
            }
            cmd.arg(&buf);
        }
    }

    if options.files.is_empty() {
        if options.change_directory {
            child.arg("--servername")
                .arg(options.servername)
                .arg("--remote-expr");
            child.arg(format!("execute('cd {}')", escape_str_in_expression(options.working_directory)));
            attach_envs(&mut child, options.envs);
        } else {
            return
        }
    } else {
        child.arg("--servername")
            .arg(options.servername);
        attach_envs(&mut child, options.envs);
        child.arg(if options.tab {"--remote-tab"} else {"--remote"});
        if options.change_directory {
            child.arg(format!("+cd {}", escape_str_in_command(options.working_directory)));
        }
    }

    let spawned = child.args(options.files).spawn().unwrap();
    zombie_killer(spawned.id());
}


pub fn remote(servername: &str, keys: &[String], expressions: &[String], use_output: bool) -> Option<(BufReader<ChildStdout>, BufReader<ChildStderr>)> {
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
        command.stderr(Stdio::piped());
        let child = command.spawn().unwrap();
        let id = child.id();
        let outputs = (BufReader::new(child.stdout.unwrap()), BufReader::new(child.stderr.unwrap()));
        (id, Some(outputs))
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


pub fn escape_str_in_command(s: &str) -> String {
    let re = regex::Regex::new(r"[\\|#]").unwrap();
    re.replace_all(s, "\\$0").to_string()
}

#[test]
fn test_escape_str() {
    assert_eq!(escape_str_in_command("hoge"), "hoge");
    assert_eq!(escape_str_in_command("ho#ge"), r"ho\#ge");
    assert_eq!(escape_str_in_command(r"ho\ge"), r"ho\\ge");
}

pub fn escape_str_in_expression(s: &str) -> String {
    let re = regex::Regex::new(r"'").unwrap();
    re.replace_all(s, "''").to_string()
}

fn zombie_killer(pid: u32) {
    thread::spawn(move || {
        let mut status = 1;
        unsafe { libc::waitpid(pid as i32, &mut status, 0) };
        trace!("process done: pid = {}", pid);
    });
}
