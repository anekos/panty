
extern crate panty;
extern crate argparse;
extern crate env_logger;

use argparse::{ArgumentParser, Store, StoreOption, List, Collect, StoreFalse};
use std::env::home_dir;
use std::io::{stdout, stderr};
use std::str::FromStr;

use panty::*;



#[derive(Debug)]
enum Command {
    Summon,
    Collector,
    Renew,
    Edit,
    TabEdit,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "summon" | "s"
                => Ok(Command::Summon),
            "collector" | "c"
                => Ok(Command::Collector),
            "renew" | "r"
                => Ok(Command::Renew),
            "edit" | "e"
                => Ok(Command::Edit),
            "tabedit" | "tedit" | "t"
                => Ok(Command::TabEdit),
            _ => Err(()),
        };
    }
}


fn command_summon(socket_filepath: String, args: Vec<String>) {
    let mut role = None;
    let mut command_args: Vec<String> = vec![];

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut role).add_option(&["--role", "-r"], StoreOption, "Set window role");
        ap.refer(&mut command_args).add_argument("arguments", List, "Files");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    spell::cast(
        socket_filepath,
        spell::Spell::Summon {files: command_args, role: role});
}


fn command_collector(socket_filepath: String, args: Vec<String>) {
    let mut max_stocks = 1;
    let mut watch_targets: Vec<String> = vec![];
    let mut recursive_watch_targets: Vec<String> = vec![];
    let mut current_directory = None;
    let mut gvim_command = "gvim".to_string();
    let mut unmap = true;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut max_stocks).add_option(&["--stocks", "-s"], Store, "Max gvim stocks");
        ap.refer(&mut watch_targets).add_option(&["--watch", "-w"], Collect, "Watch file or dirctory");
        ap.refer(&mut recursive_watch_targets).add_option(&["--recursive-watch", "-W"], Collect, "Watch dirctory (recursive)");
        ap.refer(&mut current_directory).add_option(&["--cd", "-c", "--current-directory"], StoreOption, "Current directory");
        ap.refer(&mut gvim_command).add_option(&["--gvim-command", "-g"], Store, "gVim command");
        ap.refer(&mut unmap).add_option(&["--no-unmap"], StoreFalse, "Do not unmap");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    app::start(
        max_stocks,
        socket_filepath,
        watch_targets,
        recursive_watch_targets,
        gvim::Options {current_directory: current_directory, command: gvim_command, unmap: unmap});
}


fn command_renew(socket_filepath: String) {
    spell::cast(
        socket_filepath,
        spell::Spell::Renew);
}


fn command_edit(args: Vec<String>, tab: bool) {
    let mut files: Vec<String> = vec![];
    let mut use_panty: bool = true;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Send files to gVim");

        ap.refer(&mut files).add_argument("arguments", List, "Files");
        ap.refer(&mut use_panty).add_option(&["--no-panty"], StoreFalse, "I am no panty user");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    sender::send_files(files, tab, use_panty);
}


fn main() {
    env_logger::init().unwrap();

    let mut command = Command::Summon;
    let mut args = vec!();
    let mut socket_filepath: String = {
        let mut buf = home_dir().unwrap();
        buf.push(".stockings");
        buf.to_str().unwrap().to_string()
    };

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("panty and stocking");

        ap.refer(&mut socket_filepath).add_option(&["--socket", "-s"], Store, "Socket file path");

        ap.refer(&mut command).required().add_argument("command", Store, "summon|collector|renew|edit|tabedit");
        ap.refer(&mut args).add_argument("arguments", List, "Arguments for command");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));

    match command {
        Command::Summon => command_summon(socket_filepath, args),
        Command::Collector => command_collector(socket_filepath, args),
        Command::Renew => command_renew(socket_filepath),
        Command::Edit => command_edit(args, false),
        Command::TabEdit => command_edit(args, true),
    }
}
