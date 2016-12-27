
use std::collections::{VecDeque, HashSet};
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::process::ChildStdout;
use x11::xlib::Window;

use gvim;
use gvim::SpawnOptions;


pub struct Stock {
    pub window: Window,
    pub servername: String,
    pub stdout_reader: BufReader<ChildStdout>
}


pub type Stocks = Arc<Mutex<VecDeque<Stock>>>;



pub fn collect(stocks: Stocks, n: usize, spawn_options: SpawnOptions) {
    let names = gvim::new_servernames(n);
    for servername in names {
        let stocks = stocks.clone();
        let spawn_options = spawn_options.clone();
        thread::spawn(move || {
            let (window, stdout) = gvim::spawn_secretly(&servername, &spawn_options);
            {
                let mut stocks = stocks.lock().unwrap();
                stocks.push_back(Stock {window: window, servername: servername, stdout_reader: stdout});
            }
        });
    }
}


pub fn emit(stocks: Stocks) -> Stock {
    loop {
        {
            let mut m_stocks = stocks.lock().unwrap();
            if let Some(stock) = m_stocks.pop_front() {
                return stock
            }
        }
    }
}


pub fn renew(stocks: Stocks, max_stocks: usize, spawn_options: SpawnOptions) {
    let killees: Vec<Window> = {
        let mut stocks = stocks.lock().unwrap();
        tap!(stocks.iter().map(|it| it.window).collect() => stocks.clear())
    };

    let spawn_options = spawn_options.clone();

    thread::spawn(move || {
        with_display!(display => {
            for killee in killees {
                trace!("kill: window = {}", killee);
                kill_window(display, killee);
            }
        });
        collect(stocks, max_stocks, spawn_options);
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
