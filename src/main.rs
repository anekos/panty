
extern crate panty;
extern crate argparse;
#[macro_use]
extern crate log;
extern crate env_logger;

use argparse::{ArgumentParser, Store, StoreOption, List, Collect, StoreFalse, StoreTrue, Print};
use env_logger::LogBuilder;
use std::env::home_dir;
use std::collections::HashSet;
use std::fs;
use std::io::{stdout, stderr};
use std::path::PathBuf;
use std::str::FromStr;

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


fn command_summon(silent: bool, socket_filepath: &str, args: Vec<String>) {

    let mut role = None;
    let mut files: Vec<String> = vec![];
    let mut keys: Vec<String> = vec![];
    let mut expressions: Vec<String> = vec![];
    let mut after: Option<String> = None;
    let mut before: Option<String> = None;
    let mut nofork: bool = false;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut role).add_option(&["--role", "-r"], StoreOption, "Set window role");
        ap.refer(&mut nofork).add_option(&["--nofork", "-n"], StoreTrue, "Emulation gVim's --nofork");
        ap.refer(&mut keys).add_option(&["--send", "-s"], Collect, "Send key sequence");
        ap.refer(&mut expressions).add_option(&["--expr", "-e"], Collect, "Evaluate the expression");
        ap.refer(&mut after).add_option(&["--after", "-a"], StoreOption, "Run the command after summon");
        ap.refer(&mut before).add_option(&["--before", "-a"], StoreOption, "Run the command before summon");
        ap.refer(&mut files).add_argument("arguments", List, "Files");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    let files: Vec<String> = files.iter().map(|it| to_absolute_path(it)).collect();

    puts_result(
        silent,
        spell::cast(
            socket_filepath,
            spell::Spell::Summon {files: files, keys: keys, expressions: expressions, role: role, nofork: nofork, after: after, before: before}));
}


fn command_collector(socket_filepath: &str, args: Vec<String>) {
    let mut max_stocks = 1;
    let mut watch_targets: Vec<String> = vec![];
    let mut recursive_watch_targets: Vec<String> = vec![];
    let mut current_directory = None;
    let mut gvim_command = "gvim".to_string();
    let mut unmap = true;
    let mut desktop = None;

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

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    app::start(
        max_stocks,
        socket_filepath,
        watch_targets,
        recursive_watch_targets,
        SpawnOptions {current_directory: current_directory, command: gvim_command, unmap: unmap, desktop: desktop});
}


fn command_renew(silent: bool, socket_filepath: &str) {
    puts_result(
        silent,
        spell::cast(
            socket_filepath,
            spell::Spell::Renew));
}


fn command_edit(silent: bool, socket_filepath: &str, args: Vec<String>, tab: bool) {
    let mut files: Vec<String> = vec![];
    let mut use_panty: bool = true;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Send files to gVim");

        ap.refer(&mut files).add_argument("arguments", List, "Files");
        ap.refer(&mut use_panty).add_option(&["--no-panty"], StoreFalse, "I am no panty user");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    let files: Vec<String> = files.iter().map(|it| to_absolute_path(it)).collect();

    let servername =
        sender::send_files(files.clone(), tab, use_panty).or_else(|| {
            if use_panty {
                Some(
                    spell::cast(
                        socket_filepath,
                        spell::Spell::Summon {files: files, keys: vec![], expressions: vec![], role: None, nofork: false, after: None, before: None}))
            } else {
                None
            }
        });

    if let Some(servername) = servername {
        puts_result(silent, servername);
    }
}


fn command_clean(socket_filepath: &str) {
    spell::cast(
        socket_filepath,
        spell::Spell::Clean);
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
            lister::parse_condition(&*s).unwrap()
        } else {
            HashSet::new()
        };

    puts_result(
        silent,
        spell::cast(
            socket_filepath,
            spell::Spell::Broadcast {conditions: conditions, keys: keys, expressions: expressions}));
}


fn puts_result(silent: bool, output: String) {
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


fn to_absolute_path(path: &str) -> String {
    let buf = PathBuf::from(path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or(path.to_owned())
}
