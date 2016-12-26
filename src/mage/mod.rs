
use rustc_serialize::json;
use std::io::{Read, Write};
use unix_socket::UnixListener;
use std::thread;

use summoner;
use collector;
use spell::Spell::*;
use gvim;



pub fn meditate(stocks: collector::Stocks, max_stocks: usize, socket_filepath: String, gvim_options: gvim::Options) {
    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        let gvim_options = gvim_options.clone();

        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();

                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        match json::decode(buf.as_str()).expect("Fail: json::decode") {
                            Summon {files, role, nofork} => {
                                let stock = collector::emit(stocks.clone());
                                let window = stock.window;
                                summoner::summon(stock.servername, stock.window, files, role);
                                collector::collect(stocks.clone(), 1, gvim_options);
                                if nofork {
                                    thread::spawn(move || {
                                        with_display!(display => wait_for_kill(display, window));
                                        stream.write_fmt(format_args!("OK\n")).unwrap();
                                    });
                                } else {
                                    stream.write_fmt(format_args!("OK\n")).unwrap();
                                }
                            },
                            Renew => {
                                collector::renew(stocks.clone(), max_stocks, gvim_options);
                            },
                            Clean => {
                                collector::clean(stocks.clone());
                            }

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
