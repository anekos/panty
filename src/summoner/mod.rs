

use unix_socket::UnixStream;
use std::io::prelude::Write;



pub fn summon(files: Vec<String>) {
    let mut stream = UnixStream::connect("stockings").unwrap();
    let mut content = String::new();
    for file in files {
        content.push_str(file.as_str());
        content.push_str("\n");
    }
    stream.write_all(content.as_bytes()).unwrap();
}
