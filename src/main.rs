
extern crate argparse;
extern crate dirs;
extern crate env_logger;
extern crate panty;
extern crate tempfile;

use argparse::{ArgumentParser, Store, StoreOption, List, Collect, StoreFalse, StoreTrue, Print};
use env_logger::LogBuilder;
use std::collections::HashSet;
use std::env::current_dir;
use std::env;
use std::io::{self, stderr, stdout, Read, Write};
use std::str::FromStr;

use dirs::home_dir;
use tempfile::NamedTempFile;

use panty::*;
use panty::gvim::SpawnOptions;



#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Command {
    summon,
    collector,
    renew,
    edit,
    tabEdit,
    clean,
    broadcast,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        match src {
            "summon" | "s"
                => Ok(Command::summon),
            "collector" | "c"
                => Ok(Command::collector),
            "renew" | "r"
                => Ok(Command::renew),
            "edit" | "e"
                => Ok(Command::edit),
            "tabedit" | "tedit" | "t"
                => Ok(Command::tabEdit),
            "clean"
                => Ok(Command::clean),
            "broadcast"
                => Ok(Command::broadcast),
            _ => Err(()),
        }
    }
}

const RELOAD_KEYS: &str = "<C-\\><C-n>:<C-u>source $MYVIMRC<CR>";


fn command_summon(silent: bool, socket_filepath: &str, args: Vec<String>) {
    let mut after: Option<String> = None;
    let mut before: Option<String> = None;
    let mut change_directory: bool = false;
    let mut expressions: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];
    let mut keys: Vec<String> = vec![];
    let mut nofork: bool = false;
    let mut pass_all_envs: bool = false;
    let mut pass_envs: Vec<String> = vec![];
    let mut role = None;
    let mut stdin: bool = false;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut after).add_option(&["--after", "-a"], StoreOption, "Run the command after summon");
        ap.refer(&mut before).add_option(&["--before", "-b"], StoreOption, "Run the command before summon");
        ap.refer(&mut change_directory).add_option(&["--cd", "-d"], StoreTrue, "Change directory to current directory");
        ap.refer(&mut expressions).add_option(&["--expr", "-e"], Collect, "Evaluate the expression");
        ap.refer(&mut files).add_argument("arguments", List, "Files");
        ap.refer(&mut keys).add_option(&["--send", "-s"], Collect, "Send key sequence");
        ap.refer(&mut nofork).add_option(&["--nofork", "-n"], StoreTrue, "Emulation gVim's --nofork");
        ap.refer(&mut pass_all_envs).add_option(&["--all-envs"], StoreTrue, "Pass all current env to remote");
        ap.refer(&mut pass_envs).add_option(&["--env", "-E"], Collect, "Pass the env");
        ap.refer(&mut role).add_option(&["--role", "-r"], StoreOption, "Set window role");
        ap.refer(&mut stdin).add_option(&["--stdin"], StoreTrue, "Input from STDIN");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    let working_directory = get_working_directory();

    let mut envs: Vec<(String, String)> = vec![];
    if pass_all_envs {
        for (key, value) in env::vars() {
            envs.push((key, value));
        }
    }
    for key in &pass_envs {
        if let Ok(v) = env::var(key) {
            envs.push((key.to_owned(), v));
        }
    }

    let stdin_file = if stdin { Some(make_stdin_tempfile()) } else { None };

    puts_result(
        silent,
        &spell::cast(
            socket_filepath,
            &spell::Spell::Summon { after, before, change_directory, envs, expressions, files, keys, nofork, role, stdin_file, working_directory }));
}


fn command_collector(socket_filepath: &str, args: Vec<String>) {
    let mut max_stocks = 1;
    let mut watch_targets: Vec<String> = vec![];
    let mut recursive_watch_targets: Vec<String> = vec![];
    let mut current_directory = None;
    let mut gvim_command = "gvim".to_string();
    let mut unmap = true;
    let mut desktop = None;
    let mut keys: Vec<String> = vec![];
    let mut reload: bool = false;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut max_stocks).add_option(&["--stocks", "-s"], Store, "Max gvim stocks");
        ap.refer(&mut watch_targets).add_option(&["--watch", "-w"], Collect, "Watch file or dirctory");
        ap.refer(&mut recursive_watch_targets).add_option(&["--recursive-watch", "-W"], Collect, "Watch dirctory (recursive)");
        ap.refer(&mut current_directory).add_option(&["--cd", "-c", "--current-directory"], StoreOption, "Current directory");
        ap.refer(&mut gvim_command).add_option(&["--gvim-command", "-g"], Store, "gVim command");
        ap.refer(&mut unmap).add_option(&["--no-unmap"], StoreFalse, "Do not unmap");
        ap.refer(&mut desktop).add_option(&["--desktop", "-d"], StoreOption, "Move spawned windows to the desktop (workspace)");
        ap.refer(&mut keys).add_option(&["--send", "-s"], Collect, "Send key sequence to renew gVim instances");
        ap.refer(&mut reload).add_option(&["--reload", "-r"], StoreTrue, "Send key sequence to reload $MYVIMRC");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    if reload {
        keys.push(RELOAD_KEYS.to_string());
    }

    let spawn_options = SpawnOptions { current_directory, command: gvim_command, unmap, desktop };

    let renew_options = {
        use panty::collector::RenewOptions;
        if keys.is_empty() {
            RenewOptions::Restart { max_stocks, spawn_options: spawn_options.clone() }
        } else {
            RenewOptions::Reload { keys }
        }
    };

    app::start(
        max_stocks,
        socket_filepath,
        watch_targets,
        recursive_watch_targets,
        renew_options,
        &spawn_options);
}


fn command_renew(silent: bool, socket_filepath: &str) {
    puts_result(
        silent,
        &spell::cast(socket_filepath, &spell::Spell::Renew));
}


fn command_edit(silent: bool, socket_filepath: &str, args: Vec<String>, tab: bool) {
    let mut change_directory: bool = false;
    let mut files: Vec<String> = vec![];
    let mut stdin: bool = false;
    let mut use_panty: bool = true;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Send files to gVim");

        ap.refer(&mut change_directory).add_option(&["--cd", "-d"], StoreTrue, "Change directory to current directory");
        ap.refer(&mut files).add_argument("arguments", List, "Files");
        ap.refer(&mut stdin).add_option(&["--stdin"], StoreTrue, "Input from STDIN");
        ap.refer(&mut use_panty).add_option(&["--no-panty", "-P"], StoreFalse, "I am no panty user");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    let working_directory = get_working_directory();
    let stdin_file = if stdin { Some(make_stdin_tempfile()) } else { None };

    let servername = {
        {
            let mut ref_files: Vec<&str> = files.iter().map(String::as_ref).collect();
            if let Some(stdin_file) = stdin_file.as_ref().map(AsRef::as_ref) {
                ref_files.push(stdin_file);
            }
            sender::send_files(&working_directory, &ref_files, tab, use_panty)
        } .or_else(move || {
            if use_panty {
                Some(
                    spell::cast(
                        socket_filepath,
                        &spell::Spell::Summon {
                            after: None,
                            before: None,
                            change_directory,
                            envs: vec![], // FIXME ?
                            expressions: vec![],
                            files,
                            keys: vec![],
                            nofork: false,
                            role: None,
                            stdin_file,
                            working_directory,
                        }))
            } else {
                None
            }
        })
    };

    if let Some(servername) = servername {
        puts_result(silent, &servername);
    }
}


fn command_clean(socket_filepath: &str) {
    spell::cast(
        socket_filepath,
        &spell::Spell::Clean);
}


fn command_broadcast(silent: bool, socket_filepath: &str, args: Vec<String>) {
    let mut keys: Vec<String> = vec![];
    let mut expressions: Vec<String> = vec![];
    let mut conditions: Option<String> = None;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Broadcast --remote-send or --remote-expr");

        ap.refer(&mut keys).add_option(&["--send", "-s"], Collect, "Send key sequence");
        ap.refer(&mut expressions).add_option(&["--expr", "-e"], Collect, "Evaluate the expression");
        ap.refer(&mut conditions).add_option(&["--conditions", "-c"], StoreOption, "Specify targets: visible, stocked, panty");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    let conditions =
        if let Some(s) = conditions {
            lister::parse_condition(&s).unwrap()
        } else {
            HashSet::new()
        };

    puts_result(
        silent,
        &spell::cast( socket_filepath, &spell::Spell::Broadcast { conditions, keys, expressions }));
}


fn puts_result(silent: bool, output: &str) {
    if !silent {
        print!("{}", output);
    }
}


fn main() {
    let mut command = Command::summon;
    let mut args = vec!();
    let mut silent = false;
    let mut socket_filepath: String = {
        let mut buf = home_dir().unwrap();
        buf.push(".stockings");
        buf.to_str().unwrap().to_string()
    };
    let mut log_level = "error".to_string();

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("panty and stocking");

        ap.refer(&mut socket_filepath).add_option(&["--socket", "-s"], Store, "Socket file path");
        ap.refer(&mut silent).add_option(&["-s", "--silent"], StoreTrue, "Shut up");
        ap.refer(&mut log_level).add_option(&["-l", "--log-level"], Store, "Log level");

        ap.refer(&mut command).required().add_argument("sub-command", Store, "summon|collector|renew|edit|tabedit|broadcast");

        ap.refer(&mut args).add_argument("arguments", List, "Arguments for command");

        ap.add_option(&["-v", "--version"], Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));

    let socket_filepath = socket_filepath.as_str();

    {
        let mut builder = LogBuilder::new();
        builder.parse(&log_level);
        builder.init().unwrap();
    }

    match command {
        Command::summon => command_summon(silent, socket_filepath, args),
        Command::collector => command_collector(socket_filepath, args),
        Command::renew => command_renew(silent, socket_filepath),
        Command::edit => command_edit(silent, socket_filepath, args, false),
        Command::tabEdit => command_edit(silent, socket_filepath, args, true),
        Command::clean => command_clean(socket_filepath),
        Command::broadcast => command_broadcast(silent, socket_filepath, args),
    }
}


fn get_working_directory() -> String {
    let working_directory = current_dir().expect("cwd");
    working_directory.to_str().unwrap().to_string()
}

fn make_stdin_tempfile() -> String {
    let mut file = NamedTempFile::new().unwrap(); // FIXME

    let mut buffer = vec![];
    let mut stdin = io::stdin();
    stdin.read_to_end(&mut buffer).unwrap();
    file.write_all(&buffer).unwrap();

    let path = file.into_temp_path();
    let path: String = path.keep().unwrap().to_str().unwrap().to_owned(); // FIXME

    path
}
