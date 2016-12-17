
use ctrlc;
use std::fs::remove_file;
use std::process::exit;


use collector;



pub fn watch(stocks: collector::Stocks, socket_filepath: String) {
    ctrlc::set_handler(move || {
        trace!("Executing stocks");
        with_display!(display => {
            let stocks = stocks.lock().unwrap();
            for stock in stocks.iter() {
                kill_window(display, stock.window);
            }
        });
        trace!("Executed all stocks");

        trace!("Removing socket file");
        remove_file(&socket_filepath).expect("Faild: remove socket file");
        trace!("Removed socket file");

        exit(0);
    });
}
