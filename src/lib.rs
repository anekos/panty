
extern crate x11;
extern crate unix_socket;
extern crate core;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate argparse;
extern crate rustc_serialize;
extern crate inotify;
extern crate ctrlc;
extern crate walkdir;



#[macro_use]
pub mod utils;
#[macro_use]
pub mod x;
pub mod gvim;
pub mod spell;
pub mod mage;
pub mod collector;
pub mod summoner;
pub mod police;
pub mod executioner;
pub mod app;
