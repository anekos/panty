
use rustc_serialize::json;
use std::io::prelude::Write;
use unix_socket::UnixStream;
use x11::xlib::*;

use gvim;



#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct Parameter {
    pub files: Vec<String>,
    pub role: Option<String>
}


pub fn cast(socket_filepath: String, param: Parameter) {
    let mut stream = UnixStream::connect(socket_filepath).unwrap();

    stream.write_all(json::encode(&param).expect("Fail: json::encode").as_bytes()).unwrap();
}



pub fn summon(servername: String, window: Window, param: Parameter) {
    with_display!(display => {
        let desktop = get_current_desktop(display) as i64;

        trace!("summon: window = {}, desktop = {}, servername = {}", window, desktop, servername);

        set_window_role(display, window, &"PANTY");
        restore_window(display, window);
        set_desktop_for_window(display, window, desktop);

         param.role.map(|role| set_window_role(display, window, role.as_str()));
         gvim::send_files(&servername, param.files);
    })
}
