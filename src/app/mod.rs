
use std::collections::LinkedList;
use std::fs::remove_file;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use police;
use collector;
use executioner;
use mage;


pub fn start(max_stocks: usize, socket_filepath: String, watch_targets: Vec<String>, recursive_watch_targets: Vec<String>) {
    let stocks: collector::Stocks = Arc::new(Mutex::new(LinkedList::new()));
    initialize(&socket_filepath);
    collector::collect(stocks.clone(), max_stocks);
    police::patrol(stocks.clone(), max_stocks, &watch_targets, &recursive_watch_targets);
    executioner::watch(stocks.clone(), socket_filepath.clone());
    mage::meditate(stocks.clone(), max_stocks, socket_filepath);
}


fn initialize(socket_filepath: &String) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket file");
    }
}
