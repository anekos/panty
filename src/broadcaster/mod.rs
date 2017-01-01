
use std::thread;
use std::io::Read;
use std::sync::{Arc, Mutex};

use collector;
use gvim;
use lister;



pub fn broadcast(stocks: collector::Stocks, conditions: lister::ConditionSet, keys: Vec<String>, expressions: Vec<String>) -> String {
    let instances = lister::list(Some(stocks), conditions);

    let buffer = Arc::new(Mutex::new(String::new()));

    let handles: Vec<_> =
        instances.iter().map(|instance| {
            trace!("broadcast: {}", instance.servername);

            let (keys, expressions, instance, buffer) = (keys.clone(), expressions.clone(), instance.clone(), buffer.clone());

            thread::spawn(move || {
                gvim::remote(&instance.servername, &keys, &expressions).map(|mut reader| {
                    let mut buffer = buffer.lock().unwrap();
                    let mut output = String::new();
                    reader.read_to_string(&mut output).unwrap();
                    (*buffer).push_str(&*output);
                })
            })
        }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    {
       let buffer = buffer.lock().unwrap();
       (*buffer).to_owned()
    }
}