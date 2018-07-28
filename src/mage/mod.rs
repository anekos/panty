
use rustc_serialize::json;
use std::io::{Read, Write};
use unix_socket::{UnixListener, UnixStream};
use std::net::Shutdown;
use std::thread;

use summoner;
use collector;
use collector::RenewOptions;
use broadcaster;
use spell::Spell::*;
use gvim::SpawnOptions;



pub fn meditate(stocks: &collector::Stocks, max_stocks: usize, socket_filepath: &str, spawn_options: &SpawnOptions) {
    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        let spawn_options = spawn_options.clone();

        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();
                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        match json::decode(buf.as_str()).expect("Fail: json::decode") {
                            Summon {working_directory, files, keys, expressions, after, before, role, nofork} =>
                                summon(
                                    &stocks.clone(),
                                    summoner::SummonOptions { working_directory, files, role, keys, expressions, after, before },
                                    &spawn_options,
                                    nofork,
                                    stream),
                            Broadcast {keys, expressions, conditions} => {
                                let stocks = stocks.clone();
                                thread::spawn(move || {
                                    let output = broadcaster::broadcast(stocks, &conditions, &keys, &expressions);
                                    stream.write_all(output.as_bytes()).unwrap();
                                });
                            }
                            Renew => {
                                let renew_options = RenewOptions::Restart { max_stocks, spawn_options };
                                collector::renew(stocks.clone(), renew_options);
                            }
                            Clean =>
                                collector::clean(&stocks.clone())
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error: {}", err);
                break;
            }
        }
    }

    drop(listener);
}


fn summon(stocks: &collector::Stocks, summon_options: summoner::SummonOptions, spawn_options: &SpawnOptions, nofork: bool, mut stream: UnixStream) {
    let stock = collector::emit(&stocks.clone());
    let mut stdout_reader = stock.stdout_reader;
    let servername = stock.servername;

    summoner::summon(&servername, stock.window, summon_options);
    collector::collect(&stocks.clone(), 1, spawn_options);

    if nofork {
        thread::spawn(move || {
            let mut output = String::new();
            stdout_reader.read_to_string(&mut output).unwrap();
            stream.write_fmt(format_args!("{}\n", servername)).unwrap();
            stream.shutdown(Shutdown::Both).unwrap();
        });
    } else {
        stream.write_fmt(format_args!("{}\n", servername)).unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    }
}
