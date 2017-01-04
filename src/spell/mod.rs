
use rustc_serialize::json;
use std::io::{Read, Write, BufReader};
use std::net::Shutdown;
use unix_socket::UnixStream;

use lister;


#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum Spell {
    Summon {files: Vec<String>, keys: Vec<String>, expressions: Vec<String>, after: Option<String>, before: Option<String>, role: Option<String>, nofork: bool},
    Broadcast {conditions: lister::ConditionSet, keys: Vec<String>, expressions: Vec<String>},
    Renew,
    Clean
}


pub fn cast(socket_filepath: &str, spell: Spell) -> String {
    let mut stream = UnixStream::connect(socket_filepath).unwrap();

    stream.write_all(json::encode(&spell).expect("Fail: json::encode").as_bytes()).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_to_string(&mut response).unwrap();
    response
}
