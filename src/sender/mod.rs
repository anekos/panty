
use std::io::stdin;

use gvim;



pub fn send_files(working_directory: &str, files: &[&str], tab: bool, use_panty: bool) -> Option<String> {
    let instances = if use_panty {
        gvim::find_instances(true)
    } else {
        gvim::find_instances_without_panty(true)
    };

    let servername =
        match instances.len() {
            0 => None,
            1 => Some(instances[0].servername.clone()),
            _ => number_prompt(&instances).ok()
        };

    if let Some(servername) = servername {
        gvim::send_files(&servername, working_directory, files, tab, false);
        Some(servername)
    } else {
        None
    }
}

fn number_prompt(candidates: &[gvim::Instance]) -> Result<String, String> {
    for (index, candidate) in candidates.iter().enumerate() {
        println!("[{}] {}", index, candidate.title);
    }

    let mut buffer = String::new();
    if stdin().read_line(&mut buffer).is_ok() {
        buffer.trim().parse::<usize>()
            .map_err(|it| it.to_string())
            .and_then(|number| {
                if number < candidates.len() {
                    Ok(candidates[number].servername.clone())
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
