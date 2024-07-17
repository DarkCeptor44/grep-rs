use std::{
    env,
    io::{stdin, Read},
};

fn get_stdin() -> String {
    let mut buffer = String::new();
    match stdin().read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(_) => "".to_string(),
    }
}

fn main() {
    let text = get_stdin();
    if text.is_empty() {
        return;
    }

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: grep <pattern>");
        return;
    }

    let pattern = &args[1].to_lowercase();
    for line in text.lines() {
        if line.to_lowercase().contains(pattern) {
            println!("{}", line);
        }
    }
}
