
use ctrlc;
use std::process::exit;


use collector;



pub fn watch(stocks: collector::Stocks) {
    ctrlc::set_handler(move || {
        trace!("Executing stocks");
        with_display!(display => {
            let stocks = stocks.lock().unwrap();
            for stock in stocks.iter() {
                kill_window(display, stock.window);
            }
        });
        trace!("Executed all stocks");
        exit(0);
    });
}
