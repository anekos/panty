

use unix_socket::UnixStream;
use std::io::prelude::Write;



pub fn summon() {
    let mut stream = UnixStream::connect("stockings").unwrap();
    stream.write_all(b"gal no panty kure!").unwrap();
}
