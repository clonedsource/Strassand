// Author: Vincent Dellmar
// Created: 11-07-2026 (Happy Birthday to me!)
// Email: vide1819@colorado.edu
// Description:
/*
    This is a module for a helper gui for matrlang and other stuff.
*/
#![allow(unused_imports)]
mod matrlang;
use super::matrix;
use std::{collections::HashMap, env, io::{self, Write, prelude::*, stdin}, time::Duration};
use colored::{ColoredString, Colorize};

fn input() -> (bool, String) {
    // Prints error reason.
    let fail = |reason: &str| {
        println!("{} {}", "(ERROR)".truecolor(255,0,0), reason.truecolor(255, 100, 100));
        return (false, String::new());
    };
    // Parses buffer and fails if buffer is empty.
    let mut parse = |immut_buf: &str| {
        let choice = immut_buf.trim().to_string();
        if choice.is_empty() {
            return fail("No choice was made!");
        }
        return (true, choice);
    };
    // Finds state based off buffer.
    {
        let mut buf: String = String::new();
        match io::stdin().read_line(&mut buf) {
        Ok(_) => parse(&buf),
        Err(e) => fail(&e.to_string()),
        }
    }
}
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H"); // clear + move cursor to top-left
    io::stdout().flush().unwrap();
}
fn prgm_title() {
    println!("{}{}{}{}{}",
        "---==".truecolor(100,100,100),
        "==##{[".truecolor(200,200,200),
        "MATRIX PROJECT GUI".truecolor(255, 255, 255),
        "]}##==".truecolor(200,200,200),
        "==---".truecolor(100,100,100)
    );
}
fn prgm_desc() {
    println!("{} {}\n{}{}{}{}{}",
    "-".truecolor(255, 170, 170),
    "This program is designed to allow execution of .matrlang code and see outputs.".truecolor(255, 255, 255),
    "---==".truecolor(100,100,100),
    "==####".truecolor(200,200,200),
    "##################".truecolor(255,255,255),
    "####==".truecolor(200,200,200),
    "==---".truecolor(100,100,100)
    )
}
fn prgm_fns() {
    println!("               # {} #", "Options".truecolor(255,255,255));
    let prgm_fn_list: Vec<&str> = vec![
        "execute : Execute", "view : View Log(s)", "dir : List Directory", "exit : Exit"
    ];
    for (i, str) in prgm_fn_list.iter().enumerate() {
        println!("{}{} {}",
            (i+1).to_string().truecolor(255,255,0),
            ".".truecolor(255, 255, 0),
            str.truecolor(255,255,255)
        )
    }
}

fn prgm_fns_handle() {
    // Maps strings to function pointers.
    let mut fns_hash_map: HashMap<String, fn()> = HashMap::new();
    fns_hash_map.insert("execute".to_string(), execute_fn);
    fns_hash_map.insert("view".to_string(), view_log_fn);
    fns_hash_map.insert("dir".to_string(), list_directory_fn);
    // Buffer parsed successfully -> Handle ran.
    let mut handle = |choice: &str| {
        // Handle exit separately.
        if choice.to_lowercase().eq("exit") {return false;}

        let contained: bool = fns_hash_map.contains_key(&choice.to_lowercase());
        if !contained {
            println!("{}", "Invalid Choice!".truecolor(255, 100, 100));
            std::thread::sleep(Duration::from_secs(2));
            clear_screen();
            return true
        };
        fns_hash_map.get(&choice.to_lowercase()).unwrap()();
        return true;
    };

    // Terminal Loop.
    loop {
        // Display options.
        prgm_fns();

        // Finds state based off buffer.
        let (state, choice) = input();
        
        // If state isn't satisfied just continue.
        if !state {
            // Clear your throat.
            clear_screen();
            continue;
        }

        // Clears screen.
        clear_screen();

        // Handle the choice, handle returning false is only done by the exit condition.
        if !(handle(&choice)) {break;}
        std::thread::sleep(Duration::from_secs(4));
        clear_screen();
    }
    println!("Exiting!");
    
    
}

pub fn begin() {
    // Important for matrlang
    matrlang::create_folders();

    clear_screen();
    prgm_title();
    prgm_desc();
    prgm_fns_handle();
}

// Functions that belong to the hash map.
fn execute_fn() {
    // Lists Program Directory
    let path = env::current_dir();
    if path.is_err() {
        let E = path.unwrap_err();
        println!("{} {}","(ERROR)".truecolor(255,0,0), E.to_string().truecolor(255, 100, 100));
        return;
    }
    let mut path_buf = path.unwrap().join("LocalData\\Programs");
    println!("{}", format!("Directory ({})",path_buf.display()).truecolor(255,255,255));

    //List Files in Directory
    let paths = std::fs::read_dir(path_buf.clone()).unwrap();
    for path in paths {
        println!("- {}", format!("{:?}", path.unwrap().path().file_name().unwrap()).truecolor(255,255,255));
    }
    println!("\n{}","Choose File:".truecolor(255, 255, 255));
    let (state, choice) = input();
    if !state {return;}
    // Redoes the PathBuf to match file type.
    path_buf = path_buf.join(choice);
    path_buf.add_extension("matr");

    // Checks if file exists.
    if !std::fs::exists(path_buf.clone()).unwrap() {
        println!("{} {}","(ERROR)".truecolor(255,0,0), "File does not exist!".truecolor(255, 100, 100));
    }

    matrlang::run(path_buf);

}

fn view_log_fn() {
    // Lists Program Directory
    let path = env::current_dir();
    if path.is_err() {
        let E = path.unwrap_err();
        println!("{} {}","(ERROR)".truecolor(255,0,0), E.to_string().truecolor(255, 100, 100));
        return;
    }
    let mut path_buf = path.unwrap().join("LocalData\\Logs");
    println!("{}", format!("Directory ({})",path_buf.display()).truecolor(255,255,255));

    //List Files in Directory
    let paths = std::fs::read_dir(path_buf.clone()).unwrap();
    for path in paths {
        println!("- {}", format!("{:?}", path.unwrap().path().file_name().unwrap()).truecolor(255,255,255));
    }
    println!("\n{}","Choose File:".truecolor(255, 255, 255));
    let (state, choice) = input();
    if !state {return;}
    // Redoes the PathBuf to match file type.
    path_buf = path_buf.join(choice);
    path_buf.add_extension("log");

    // Checks if file exists.
    if !std::fs::exists(path_buf.clone()).unwrap() {
        println!("{} {}","(ERROR)".truecolor(255,0,0), "File does not exist!".truecolor(255, 100, 100));
    }
    matrlang::view_log(path_buf);
}
fn list_directory_fn() {
    let path = env::current_dir();
    if path.is_err() {
        let E = path.unwrap_err();
        println!("{} {}","(ERROR)".truecolor(255,0,0), E.to_string().truecolor(255, 100, 100));
        return;
    }
    let mut path_buf = path.unwrap();
    path_buf = path_buf.join("LocalData");
    println!("{}", format!("Directory ({})",path_buf.display()).truecolor(255,255,255));

    
    std::thread::sleep(Duration::from_secs(3));
}