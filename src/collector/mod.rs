
use std::collections::{LinkedList, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use x11::xlib::Window;

use gvim;


#[derive(Clone)]
pub struct Stock {
    pub window: Window,
    pub servername: String
}


pub type Stocks = Arc<Mutex<LinkedList<Stock>>>;



pub fn collect(stocks: Stocks, n: usize, gvim_options: gvim::Options) {
    let names = gvim::new_servernames(n);
    for servername in names {
        let stocks = stocks.clone();
        let gvim_options = gvim_options.clone();
        thread::spawn(move || {
            let window = gvim::spawn_secretly(&servername, &gvim_options);
            {
                let mut stocks = stocks.lock().unwrap();
                stocks.push_back(Stock {window: window, servername: servername});
            }
        });
    }
}


pub fn emit(stocks: Stocks) -> Stock {
    loop {
        {
            let mut m_stocks = stocks.lock().unwrap();
            match m_stocks.pop_front() {
                Some(stock) => return stock,
                None => ()
            }
        }
    }
}


pub fn renew(stocks: Stocks, max_stocks: usize, gvim_options: gvim::Options) {
    let killees = {
        let mut stocks = stocks.lock().unwrap();
        tap!((*stocks).clone() => stocks.clear())
    };

    let gvim_options = gvim_options.clone();

    thread::spawn(move || {
        with_display!(display => {
            for killee in killees {
                trace!("kill: {}", killee.servername);
                kill_window(display, killee.window);
            }
        });
        collect(stocks, max_stocks, gvim_options);
    });
}


pub fn clean(stocks: Stocks) {
    let current_stocks: HashSet<Window> = {
        let stocks = stocks.lock().unwrap();
        stocks.iter().map(|it| it.window).collect()
    };

    let instances = gvim::find_instances_without_panty(false);
    with_display!(display => {
        let role = Some(gvim::STOCKED_WINDOW_ROLE.to_string());
        for instance in instances {
            if !current_stocks.contains(&instance.window) && role == get_window_role(display, instance.window) {
                println!("kill_window: window = {}, servername = {}", instance.window, instance.servername);
                kill_window(display, instance.window);
            }
        }
    })
}
