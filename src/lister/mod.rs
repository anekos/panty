
use std::collections::HashSet;
use std::hash::BuildHasher;
use std::thread;

use gvim;
use collector::Stocks;



#[derive(PartialEq,Eq,Hash,Clone,RustcEncodable,RustcDecodable,Debug)]
pub enum Condition {
    Visible(bool),
    Stocked(bool),
    Panty(bool),
}

pub type ConditionSet = HashSet<Condition>;


pub fn parse_condition(s: &str) -> Result<ConditionSet, String> {
    use self::Condition::*;

    let mut set: ConditionSet = HashSet::new();

    for term in s.split_terminator(',') {
        let invert = term.starts_with('!');
        let term: &str = if invert { &term[1..] } else { term };
        match term {
            "v" | "visible" => set.insert(Visible(invert)),
            "s" | "stocked" => set.insert(Stocked(invert)),
            "p" | "panty" => set.insert(Panty(invert)),
            invalid => return Err(format!("Invalid condition name: {}", invalid)),
        };
    }

    Ok(set)
}



pub fn list<S: BuildHasher + Clone + Send + 'static>(stocks: &Option<Stocks>, conditions: &HashSet<Condition, S>) -> Vec<gvim::Instance> {
    let servernames: Vec<String> = gvim::fetch_existing_servernames();

    let join_handles: Vec<_> = servernames.iter()
        .map(|servername| {
            let servername = servername.to_owned();
            let conditions = conditions.clone();
            let stocks = stocks.clone();
            thread::spawn(move || condition_match(stocks, conditions, &servername))
        })
        .collect();

    {

        let mut result = vec![];

        for handle in join_handles {
            if let Ok(found) = handle.join() {
                if let Some(instance) = found {
                    result.push(instance);
                }
            }
        }

        result
    }
}


fn condition_match<S: BuildHasher + Clone + Send>(stocks: Option<Stocks>, conditions: HashSet<Condition, S>, servername: &str) -> Option<gvim::Instance> {
    use self::Condition::*;

    let stocked_servers: HashSet<String> = if let Some(stocks) = stocks {
        let stocks = stocks.lock().unwrap();
        stocks.iter().map(|it| it.servername.clone()).collect()
    } else {
        HashSet::new()
    };

    with_display!(display => {
        if let Some(window) = gvim::fetch_window_id(&servername) {
            let mut matched = true;
            for condition in conditions {
                let m = match condition {
                    Visible(invert) => invert != is_window_visible(display, window),
                    Stocked(invert) => invert != stocked_servers.contains(&*servername),
                    Panty(invert) => invert != get_text_property(display, window, "_PANTY_SERVERNAME").is_some()
                };
                if !m {
                    matched = false;
                    break;
                }
            }

            if matched {
                if let Some(title) = get_text_property(display, window, "WM_NAME") {
                    return Some(gvim::Instance {
                        servername: servername.to_owned(),
                        title,
                        window,
                    })
                }
            }
        }

        None
    })

}
