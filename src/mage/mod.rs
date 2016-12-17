
use rustc_serialize::json;
use std::io::Read;
use unix_socket::UnixListener;

use summoner;
use collector;
use spell::Spell::*;



pub fn meditate(stocks: collector::Stocks, max_stocks: usize, socket_filepath: String) {
    let listener = UnixListener::bind(socket_filepath).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf: String = "".to_string();

                match stream.read_to_string(&mut buf).unwrap() {
                    _ => {
                        match json::decode(buf.as_str()).expect("Fail: json::decode") {
                            Summon {files, role} => {
                                let stock = collector::emit(stocks.clone());
                                summoner::summon(stock.servername, stock.window, files, role);
                                collector::collect(stocks.clone(), 1);
                            },
                            Renew => {
                                collector::renew(stocks.clone(), max_stocks);
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
