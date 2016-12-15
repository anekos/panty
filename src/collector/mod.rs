
use std::collections::LinkedList;
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



pub fn collect(stocks: Stocks, n: usize) {
    let names = gvim::new_servernames(n);
    for servername in names {
        let stocks = stocks.clone();
        thread::spawn(move || {
            let window = gvim::spawn_in_secret(&servername);
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
