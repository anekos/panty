
use rustc_serialize::json;
use std::io::{Write, BufReader, BufRead};
use std::net::Shutdown;
use unix_socket::UnixStream;


#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum Spell {
    Summon {files: Vec<String>, keys: Option<String>, role: Option<String>, nofork: bool},
    Renew,
    Clean
}


pub fn cast(socket_filepath: String, spell: Spell) -> String {
    let mut stream = UnixStream::connect(socket_filepath).unwrap();

    stream.write_all(json::encode(&spell).expect("Fail: json::encode").as_bytes()).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    response
}
