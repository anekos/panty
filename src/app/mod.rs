
use std::collections::VecDeque;
use std::fs::remove_file;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use police;
use collector;
use collector::RenewOptions;
use executioner;
use mage;
use gvim::SpawnOptions;


pub fn start(max_stocks: usize, socket_filepath: &str, watch_targets: Vec<String>, recursive_watch_targets: Vec<String>, renew_options: RenewOptions, spawn_options: SpawnOptions) {
    let stocks: collector::Stocks = Arc::new(Mutex::new(VecDeque::new()));

    initialize(socket_filepath);

    collector::collect(stocks.clone(), max_stocks, spawn_options.clone());
    police::patrol(stocks.clone(), watch_targets, recursive_watch_targets, renew_options);
    executioner::watch(stocks.clone(), socket_filepath.to_string());
    mage::meditate(stocks.clone(), max_stocks, socket_filepath, spawn_options.clone());
}


fn initialize(socket_filepath: &str) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket file");
    }
}
