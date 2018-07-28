
use std::process::Command;

use x11::xlib::Window;

use gvim;


pub struct SummonOptions {
    pub working_directory: String,
    pub files: Vec<String>,
    pub keys: Vec<String>,
    pub expressions: Vec<String>,
    pub role: Option<String>,
    pub after: Option<String>,
    pub before: Option<String>
}


pub fn summon(servername: &str, window: Window, options: SummonOptions) {

    if let Some(command_line) = options.before {
        after(&command_line, &servername, window);
    }

    with_display!(display => {
        let desktop = get_current_desktop(display) as i64;

        trace!("summon: window = {}, desktop = {}, servername = {}", window, desktop, servername);

        let mut tried = 1;

        loop {
            restore_window(display, window);
            set_desktop_for_window(display, window, desktop);

            if let Some(ref role) = options.role {
                set_window_role(display, window, role.as_str())
            } else {
                set_window_role(display, window, &gvim::SUMMONED_WINDOW_ROLE);
            }

             if wait_for_visible(display, window, 100) {
                 break;
             } else {
                 error!("Failed: wait_for_visible: servername = {}, window = {}, tried = {}", servername, window, tried);
                 tried += 1;
             }
        }

        let files: Vec<&str> = options.files.iter().map(String::as_ref).collect();
        gvim::send_files(servername, &options.working_directory, &files, false);

        gvim::remote(servername, &options.keys, &options.expressions, false);
    });

    if let Some(command_line) = options.after {
        after(&command_line, servername, window);
    }
}


pub fn after(command_line: &str, servername: &str, window: Window) {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command_line)
        .env("PANTY_WINDOWID", window.to_string())
        .env("PANTY_SERVERNAME", servername)
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to run: {}", command_line));
    child.wait().unwrap();
}
