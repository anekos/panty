
use collector;
use gvim;
use lister;



pub fn broadcast(stocks: collector::Stocks, conditions: lister::ConditionSet, keys: Vec<String>, expressions: Vec<String>) {
    let instances = lister::list(Some(stocks), conditions);
    for instance in instances {
        trace!("broadcast: {}", instance.servername);
        gvim::remote(&instance.servername, &keys, &expressions);
    }
}
