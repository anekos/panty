

use inotify::INotify;
use inotify::ffi::*;
use std::path::Path;
use std::thread;

use collector;


pub fn patrol(stocks: collector::Stocks, max_stocks: usize, targets: Vec<String>) {
    thread::spawn(move || {
        let mut ino = INotify::init().unwrap();

        for target in targets {
            info!("Patrol: {}", target);
            ino.add_watch(Path::new(&target), IN_CREATE | IN_MODIFY | IN_DELETE).unwrap();
        }

        loop {
            let events = ino.wait_for_events().unwrap();

            for event in events.iter() {
                if !event.is_dir() {
                    trace!("event: is_dir");
                    renew(stocks.clone(), max_stocks)
                }
            }
        }
    });
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
