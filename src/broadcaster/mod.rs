
use collector;
use gvim;



pub fn broadcast(stocks: collector::Stocks, keys: Vec<String>, expressions: Vec<String>) {
    let m_stocks = stocks.lock().unwrap();
    let servernames = m_stocks.iter().map(|it| it.servername.clone());
    for servername in servernames {
        gvim::remote(&servername, &keys, &expressions);
    }
}
