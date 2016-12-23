

use gvim;



pub fn clean() {
    let instances = gvim::find_instances_without_panty(false);
    with_display!(display => {
        let role = Some(gvim::STOCKED_WINDOW_ROLE.to_string());
        for instance in instances {
            if role == get_window_role(display, instance.window) {
                println!("kill_window: window = {}, servername = {}", instance.window, instance.servername);
                kill_window(display, instance.window);
            }
        }
    })
}
