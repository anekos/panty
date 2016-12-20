
use std::collections::LinkedList;
use std::fs::remove_file;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use police;
use collector;
use executioner;
use mage;
use gvim;


pub fn start(max_stocks: usize, socket_filepath: String, watch_targets: Vec<String>, recursive_watch_targets: Vec<String>, gvim_options: gvim::Options) {
    let stocks: collector::Stocks = Arc::new(Mutex::new(LinkedList::new()));
    initialize(&socket_filepath);
    collector::collect(stocks.clone(), max_stocks, gvim_options.clone());
    police::patrol(stocks.clone(), max_stocks, &watch_targets, &recursive_watch_targets, gvim_options.clone());
    executioner::watch(stocks.clone(), socket_filepath.clone());
    mage::meditate(stocks.clone(), max_stocks, socket_filepath, gvim_options.clone());
}


fn initialize(socket_filepath: &String) {
    if Path::new(&socket_filepath).exists() {
        remove_file(&socket_filepath).expect("Faild: remove socket file");
    }
}
