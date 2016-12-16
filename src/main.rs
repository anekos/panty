
extern crate panty;
extern crate argparse;
extern crate env_logger;

use argparse::{ArgumentParser, Store, StoreOption, List, Collect};
use std::env::home_dir;
use std::io::{stdout, stderr};
use std::str::FromStr;

use panty::*;



#[derive(Debug)]
enum Command {
    Summon,
    Collector,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "summon" => Ok(Command::Summon),
            "collector" => Ok(Command::Collector),
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

    summoner::cast(
        socket_filepath,
        summoner::Parameter {files: command_args, role: role});
}


fn command_collector(socket_filepath: String, args: Vec<String>) {
    let mut max_stocks = 1;
    let mut watch_targets: Vec<String> = vec![];

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Summon gVim window");

        ap.refer(&mut max_stocks).add_option(&["--stocks", "-s"], Store, "Max gvim stocks");
        ap.refer(&mut watch_targets).add_option(&["--watch", "-w"], Collect, "Watch file or dirctory");

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    app::start(max_stocks, socket_filepath, watch_targets);
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

        ap.refer(&mut command).required().add_argument("command", Store, "summon/collector");
        ap.refer(&mut args).add_argument("arguments", List, r#"Arguments for command"#);

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));

    match command {
        Command::Summon => command_summon(socket_filepath, args),
        Command::Collector => command_collector(socket_filepath, args)
    }
}
