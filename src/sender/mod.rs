
use std::io::stdin;

use gvim;



pub fn send_files(files: Vec<String>, tab: bool, use_panty: bool) {
    let instances = if use_panty {
        gvim::find_instances(true)
    } else {
        gvim::find_instances_without_panty(true)
    };

    match instances.len() {
        0 => error!("No gVim!"),
        1 => gvim::send_files(&instances[0].servername, files, tab),
        _ => {
            match number_prompt(&instances) {
                Ok(servername) => gvim::send_files(servername, files, tab),
                Err(e) => println!("{}", e)
            }
        }
    }
}

fn number_prompt<'a>(candidates: &'a Vec<gvim::Instance>) -> Result<&'a String, String> {
    for (index, candidate) in candidates.iter().enumerate() {
        println!("[{}] {}", index, candidate.title);
    }

    let mut buffer = String::new();
    if stdin().read_line(&mut buffer).is_ok() {
        buffer.trim().parse::<usize>()
            .map_err(|it| it.to_string())
            .and_then(|number| {
                if number < candidates.len() {
                    Ok(&candidates[number].servername)
                } else {
                    Err("Index out of bounds".to_string())
                }
            })
    } else {
        Err("Can't read".to_string())
    }

    // use std::io::Read;
    // const HINT_CHARS: &'static str = "asdfghklqwertyuiopzxcvbnm1234567890";
    // if candidates.len() <= HINT_CHARS.len() {
    //
    //     let mut index = 0;
    //     for candidate in candidates {
    //         println!("[{}] {}", HINT_CHARS.chars().nth(index).unwrap(), candidate);
    //         index += 1;
    //     }
    //
    //     print!("Select: ");
    //     let mut buffer = [0; 1];
    //     if stdin().read_exact(&mut buffer).is_ok() {
    //         let input_c = buffer[0] as char;
    //         let mut index = 0;
    //         for hint_c in HINT_CHARS.chars() {
    //             if hint_c == input_c {
    //                 return Some(&candidates[index])
    //             }
    //             index += 1;
    //         }
    //     }
    // }
}
