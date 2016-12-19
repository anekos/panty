

use inotify::INotify;
use inotify::ffi::*;
use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use walkdir::WalkDir;

use collector;



const EVENTS: u32 = IN_CREATE | IN_MODIFY | IN_DELETE;


pub fn patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<String>, rec_targets: &Vec<String>) {
    let mut files: Vec<PathBuf> = vec![];
    let mut directories: Vec<PathBuf> = vec![];

    for target in targets {
        let path = Path::new(target).to_path_buf();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            directories.push(path);
        } else {
            error!("Invalid watch target: {}", path.to_str().unwrap());
        }
    }

    for target in rec_targets {
        for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                directories.push(entry.path().to_path_buf());
            }
        }
    }

    {
        let stocks = stocks.clone();
        thread::spawn(move || file_patrol(stocks, max_stocks, &files));
    }
    {
        let stocks = stocks.clone();
        thread::spawn(move || directory_patrol(stocks, max_stocks, &directories));
    }
}


fn file_patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<PathBuf>) {
    let mut ino = INotify::init().unwrap();
    let mut table: HashMap<i32, HashSet<String>> = HashMap::new();
    let mut watched: HashMap<PathBuf, i32> = HashMap::new();

    for target in targets {
        if let Some(dir) = target.parent() {
            let wd = watched.entry(dir.to_path_buf()).or_insert_with(|| {
                trace!("watch: {}", dir.to_str().unwrap());
                ino.add_watch(dir, EVENTS).unwrap()
            });
            let name = target.file_name().unwrap().to_str().unwrap().to_string();
            table.entry(*wd).or_insert_with(|| HashSet::new()).insert(name);
        }
    }

    loop {
        let events = ino.wait_for_events().unwrap();

        for event in events.iter() {
            if !event.is_dir() {
                if let Some(set) = table.get(&event.wd) {
                    if set.contains(event.name.to_str().unwrap()) {
                        trace!("file_changes: name = {}, wd = {}", event.name.to_str().unwrap(), event.wd);
                        collector::renew(stocks.clone(), max_stocks)
                    }
                }
            }
        }
    }
}


fn directory_patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<PathBuf>) {
    let mut ino = INotify::init().unwrap();

    for target in targets {
        trace!("watch: {}", target.to_str().unwrap());
        ino.add_watch(target, EVENTS).unwrap();
    }

    loop {
        let events = ino.wait_for_events().unwrap();

        for event in events.iter() {
            if !event.is_dir() {
                trace!("file_changes: {}", event.name.to_str().unwrap());
                collector::renew(stocks.clone(), max_stocks)
            }
        }
    }
}
