

use rustc_serialize::json;
use std::io::prelude::Write;
use unix_socket::UnixStream;
use x11::xlib::Window;

use gvim;
use x;



#[derive(RustcEncodable, RustcDecodable)]
pub struct Parameter {
    pub files: Vec<String>,
    pub role: Option<String>
}


pub fn cast(param: Parameter) {
    let mut stream = UnixStream::connect("stockings").unwrap();

    stream.write_all(json::encode(&param).expect("Fail: json::encode").as_bytes()).unwrap();
}


pub fn summon(servername: String, window: Window, param: Parameter) {
    let desktop = x::get_current_desktop() as i64;

    println!("display_gvim: window = {}, desktop = {}, servername = {}", window, desktop, servername);

    x::set_window_role(window, &"PANTY");
    x::map_window(window);
    x::set_desktop_for_window(window, desktop);

    param.role.map(|role| x::set_window_role(window, role.as_str()));
    gvim::send_files(&servername, param.files);
}
