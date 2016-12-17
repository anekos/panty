
use rustc_serialize::json;
use std::io::Write;
use unix_socket::UnixStream;


#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum Spell {
    Summon {files: Vec<String>, role: Option<String>},
    Renew
}


pub fn cast(socket_filepath: String, spell: Spell) {
    let mut stream = UnixStream::connect(socket_filepath).unwrap();

    stream.write_all(json::encode(&spell).expect("Fail: json::encode").as_bytes()).unwrap();
}
