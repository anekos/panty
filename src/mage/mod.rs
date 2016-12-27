
use rustc_serialize::json;
use std::io::{Read, Write};
use unix_socket::{UnixListener, UnixStream};
use std::thread;

use summoner;
use collector;
use spell::Spell::*;
use gvim::SpawnOptions;



pub fn meditate(stocks: collector::Stocks, max_stocks: usize, socket_filepath: String, spawn_options: SpawnOptions) {
    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        let spawn_options = spawn_options.clone();

        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();
                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        match json::decode(buf.as_str()).expect("Fail: json::decode") {
                            Summon {files, role, nofork} =>
                                summon(stocks.clone(), files, role, nofork, spawn_options, stream),
                            Renew =>
                                collector::renew(stocks.clone(), max_stocks, spawn_options),
                            Clean =>
                                collector::clean(stocks.clone())
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


fn summon(stocks: collector::Stocks, files: Vec<String>, role: Option<String>, nofork: bool, spawn_options: SpawnOptions, mut stream: UnixStream) {
    let stock = collector::emit(stocks.clone());
    let mut stdout_reader = stock.stdout_reader;
    let servername = stock.servername;
    summoner::summon(servername.clone(), stock.window, files, role);
    collector::collect(stocks.clone(), 1, spawn_options);
    if nofork {
        thread::spawn(move || {
            let mut output = String::new();
            stdout_reader.read_to_string(&mut output).unwrap();
            debug!("gVim output: {}", output);
            stream.write_fmt(format_args!("{}\n", servername)).unwrap();
        });
    } else {
        stream.write_fmt(format_args!("{}\n", servername)).unwrap();
    }
}
