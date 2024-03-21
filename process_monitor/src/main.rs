use std::sync::{Arc, Mutex};
use std::env;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Utc};
use std::fs;

extern crate rand;
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)] // Add Clone trait here
struct Monitor {
    name: Option<String>,
    script: Option<String>,
    result: Option<String>,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Monitors {
    monitors: Vec<Monitor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Result {
    value: i32,
    processed_at: DateTime<Utc>,
}

fn process_command_line_arguments() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Arguments parsed, but expected 3 arguments");
        return None;
    }
    if args[1] != "-monitorFile" {
        println!("First argument must be -monitorFile");
        return None;
    }
    Some(args[2].clone())
}

fn update_monitor(monitor: &mut Monitor) {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(0..100);
    let current_time = Utc::now();
    let result = Result {
        value: n as i32,
        processed_at: current_time,
    };
    let result_string = serde_json::to_string(&result).unwrap();
    println!("Result {:?} has been updated to the monitor", result_string);
    monitor.result = Some(result_string);
}

fn store_monitor(monitors_data: &Monitors) {
    let current_time = Utc::now();
    let file_name = current_time.format("%Y-%m-%d_%H-%M-%S.json").to_string();
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let mut file_path = PathBuf::from(current_dir);
    file_path.push(file_name);
    let json_data = serde_json::to_string(&monitors_data).expect("Failed to serialize data to JSON");
    fs::write(&file_path, json_data).expect("Failed to write to file");
    println!("Successfully stored the monitor as {}", file_path.display());
}

fn process_monitor(monitors_data: &mut Monitors) {
    let monitors_data_arc = Arc::new(Mutex::new(monitors_data.monitors.clone())); // Clone the Vec<Monitor>
    let handle = thread::spawn(move || {
        let mut monitors_data_lock = monitors_data_arc.lock().unwrap();
        for monitor in &mut *monitors_data_lock {
            update_monitor(monitor);
        }
    });
    handle.join().unwrap();
    store_monitor(monitors_data);
}

fn main() {
    let file_path = match process_command_line_arguments() {
        Some(path) => path,
        None => return,
    };
    let mut file_content = String::new();
    let mut file = match File::open(&file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file: {}", e);
            return;
        }
    };
    match file.read_to_string(&mut file_content) {
        Ok(_) => (),
        Err(e) => {
            println!("Error reading file: {}", e);
            return;
        }
    };
    println!("Contents of monitors.json file: {}", file_content);
    let mut monitors: Monitors = match serde_json::from_str(&file_content) {
        Ok(monitors) => monitors,
        Err(e) => {
            println!("Error parsing JSON: {}", e);
            return;
        }
    };
    process_monitor(&mut monitors);
}
