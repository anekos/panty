
extern crate patrol;

use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

use collector;
use collector::RenewOptions;



pub fn patrol(stocks: collector::Stocks, targets: Vec<String>, rec_targets: Vec<String>, renew_options: RenewOptions) {
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
            collector::renew(stocks.clone(), renew_options.clone());

            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {}
        }
    });
}
