
use std::collections::VecDeque;
use std::fs::remove_file;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use police;
use collector;
use executioner;
use mage;
use gvim::SpawnOptions;


pub fn start(max_stocks: usize, socket_filepath: String, watch_targets: Vec<String>, recursive_watch_targets: Vec<String>, spawn_options: SpawnOptions) {
    let stocks: collector::Stocks = Arc::new(Mutex::new(VecDeque::new()));
    initialize(&socket_filepath);
    collector::collect(stocks.clone(), max_stocks, spawn_options.clone());
    police::patrol(stocks.clone(), max_stocks, &watch_targets, &recursive_watch_targets, spawn_options.clone());
    executioner::watch(stocks.clone(), socket_filepath.clone());
    mage::meditate(stocks.clone(), max_stocks, socket_filepath, spawn_options.clone());
}


fn initialize(socket_filepath: &str) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket file");
    }
}
