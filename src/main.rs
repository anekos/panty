
extern crate panty;
extern crate argparse;

use argparse::{ArgumentParser, Store, List};
use std::io::{stdout, stderr};
use std::str::FromStr;

use panty::*;



#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    summon,
    collector,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "summon" => Ok(Command::summon),
            "collector" => Ok(Command::collector),
            _ => Err(()),
        };
    }
}


fn command_summon(args: Vec<String>) {
    // let mut role = "".to_string();
    let mut command_args: Vec<String> = vec![];

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Plays a sound");
        // ap.refer(&mut role).add_option(&["--role"], Store, r#"Output sink to play to"#);
        ap.refer(&mut command_args).add_argument("arguments", List, r#"Arguments for command"#);

        ap.parse(args, &mut stdout(), &mut stderr()).map_err(|x| std::process::exit(x)).unwrap();
    }

    summoner::summon(command_args);
}


fn command_collector() {
    collector::start();
}


fn main() {
    let mut command = Command::summon;
    let mut args = vec!();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Plays or records sound");
        ap.refer(&mut command).required().add_argument("command", Store, "summon/collector");
        ap.refer(&mut args).add_argument("arguments", List, r#"Arguments for command"#);
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    args.insert(0, format!("command {:?}", command));
    match command {
        Command::summon => command_summon(args),
        Command::collector => command_collector(),
    }
}
