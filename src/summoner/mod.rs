
use std::process::Command;

use x11::xlib::Window;

use gvim;


pub struct SummonOptions {
    pub files: Vec<String>,
    pub keys: Vec<String>,
    pub expressions: Vec<String>,
    pub role: Option<String>,
    pub after: Option<String>,
    pub before: Option<String>
}


pub fn summon(servername: String, window: Window, options: SummonOptions) {

    if let Some(command_line) = options.before {
        after(&command_line, &servername, window);
    }

    with_display!(display => {
        let desktop = get_current_desktop(display) as i64;

        trace!("summon: window = {}, desktop = {}, servername = {}", window, desktop, servername);

        restore_window(display, window);
        set_desktop_for_window(display, window, desktop);

        if let Some(role) = options.role {
            set_window_role(display, window, role.as_str())
        } else {
            set_window_role(display, window, &gvim::SUMMONED_WINDOW_ROLE);
        }

         gvim::send_files(&servername, options.files, false);
         gvim::remote(&servername, &options.keys, &options.expressions);
    });

    if let Some(command_line) = options.after {
        after(&command_line, &servername, window);
    }
}


pub fn after(command_line: &str, servername: &str, window: Window) {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command_line)
        .env("PANTY_WINDOWID", window.to_string())
        .env("PANTY_SERVERNAME", servername)
        .spawn()
        .expect(&*format!("Failed to run: {}", command_line));
    child.wait().unwrap();
}
