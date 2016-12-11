
pub fn spawn() -> u32 {
    use std::process::Command;
    Command::new("gvim")
        .arg("--nofork")
        .spawn()
        .expect("Failed to execute process")
        .id()
}
