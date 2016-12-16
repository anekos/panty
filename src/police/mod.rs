

use inotify::INotify;
use inotify::ffi::*;
use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::thread;

use collector;



const EVENTS: u32 = IN_CREATE | IN_MODIFY | IN_DELETE;


pub fn patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<String>) {
    let mut files: Vec<String> = vec![];
    let mut directories: Vec<String> = vec![];

    for target in targets {
        let path = Path::new(target);
        if path.is_file() {
            files.push(target.to_string());
        } else {
            directories.push(target.to_string());
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


fn file_patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<String>) {
    let mut ino = INotify::init().unwrap();
    let mut table: HashMap<i32, HashSet<String>> = HashMap::new();

    for target in targets {
        let path = Path::new(target);
        if let Some(dir) = path.parent() {
            let wd = ino.add_watch(dir, EVENTS).unwrap();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            table.entry(wd).or_insert_with(|| HashSet::new()).insert(name);
        }
    }

    loop {
        let events = ino.wait_for_events().unwrap();

        for event in events.iter() {
            if !event.is_dir() {
                if let Some(set) = table.get(&event.wd) {
                    if set.contains(event.name.to_str().unwrap()) {
                        trace!("file_changes: name = {}, wd = {}", event.name.to_str().unwrap(), event.wd);
                        renew(stocks.clone(), max_stocks)
                    }
                }
            }
        }
    }
}


fn directory_patrol(stocks: collector::Stocks, max_stocks: usize, targets: &Vec<String>) {
    let mut ino = INotify::init().unwrap();

    for target in targets {
        ino.add_watch(Path::new(target), EVENTS).unwrap();
    }

    loop {
        let events = ino.wait_for_events().unwrap();

        for event in events.iter() {
            if !event.is_dir() {
                trace!("file_changes: {}", event.name.to_str().unwrap());
                renew(stocks.clone(), max_stocks)
            }
        }
    }
}


fn renew(stocks: collector::Stocks, max_stocks: usize) {
    let killees = {
        let mut stocks = stocks.lock().unwrap();
        tap!((*stocks).clone() => stocks.clear())
    };

    thread::spawn(move || {
        with_display!(display => {
            for killee in killees {
                trace!("kill: {}", killee.servername);
                kill_window(display, killee.window);
            }
        });
        collector::collect(stocks, max_stocks);
    });
}
