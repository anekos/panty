
extern crate patrol;

use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

use collector;
use gvim::SpawnOptions;



pub fn patrol(stocks: collector::Stocks, max_stocks: usize, targets: Vec<String>, rec_targets: Vec<String>, spawn_options: SpawnOptions) {
    let mut targets = targets;

    for target in rec_targets {
        for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                targets.push(entry.path().to_str().unwrap().to_string())
            }
        }
    }

    let rx = patrol::spawn(targets.iter().map(|it| patrol::Target::new(it)).collect());

    thread::spawn(move || {
        loop {
            rx.recv().unwrap();

            trace!("Renew stocked instances");
            collector::renew(stocks.clone(), max_stocks, spawn_options.clone());

            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {}
        }
    });
}
