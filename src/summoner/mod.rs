
use x11::xlib::Window;

use gvim;


pub struct SummonOptions {
    pub files: Vec<String>,
    pub keys: Vec<String>,
    pub expressions: Vec<String>,
    pub role: Option<String>
}


pub fn summon(servername: String, window: Window, options: SummonOptions) {

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
    })
}
