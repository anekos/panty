
extern crate x11;
extern crate unix_socket;
extern crate core;
#[macro_use]
extern crate log;
extern crate argparse;
extern crate serde;
extern crate serde_json;
extern crate inotify;
extern crate ctrlc;
extern crate walkdir;
extern crate tempfile;
extern crate rand;
extern crate libc;



#[macro_use]
pub mod utils;
#[macro_use]
pub mod x;
pub mod namer;
pub mod gvim;
pub mod spell;
pub mod lister;
pub mod broadcaster;
pub mod mage;
pub mod collector;
pub mod summoner;
pub mod police;
pub mod executioner;
pub mod sender;
pub mod app;
