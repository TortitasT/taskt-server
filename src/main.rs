mod task;

use core::time;
use std::{
    io::{Read, Result, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use task::Task;

#[derive(Clone)]
struct State {
    tasks: Vec<Task>,
}

fn main() -> Result<()> {
    let receiver_listener =
        TcpListener::bind("127.0.0.1:7878").expect("Failed and bind with the sender");

    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();

    let state = Arc::new(Mutex::new(State {
        tasks: vec![
            Task {
                text: "Welcome to taskt".to_string(),
                completed: false,
            },
            Task {
                text: "You are connected to a server :)".to_string(),
                completed: false,
            },
        ],
    }));

    for stream in receiver_listener.incoming() {
        let stream = stream.expect("Failed to establish connection");

        let state = Arc::clone(&state);

        let handle = thread::spawn(move || {
            handle_sender(stream, state).unwrap_or_else(|error| eprintln!("{:?}", error))
        });

        thread_vec.push(handle);
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }

    Ok(())
}

fn handle_sender(mut stream: TcpStream, state: Arc<Mutex<State>>) -> Result<()> {
    let mut locked_state = state.lock().unwrap();

    let mut buf = [0; 512];

    let bytes_read = stream.read(&mut buf)?;
    if bytes_read == 0 {
        return Ok(());
    }

    println!("Received: {}", String::from_utf8_lossy(&buf[..bytes_read]));

    if &buf[..bytes_read] == b"read\n" {
        println!("Requested read from {}", stream.peer_addr()?);

        let tasks_json = serde_json::to_string(&locked_state.tasks).unwrap();

        stream.write(tasks_json.as_bytes())?;

        return Ok(());
    }

    if buf[..bytes_read].starts_with(b"write\n") {
        println!("Requested write from {}", stream.peer_addr()?);

        let all_string = String::from_utf8_lossy(&buf[..bytes_read]);

        let tasks_string = all_string.split('\n').nth(1).unwrap();

        let tasks: Vec<Task> = serde_json::from_str(&tasks_string).unwrap();

        *locked_state = State { tasks };

        println!("Received tasks: {}", tasks_string);

        return Ok(());
    }

    thread::sleep(time::Duration::from_secs(1));

    Ok(())
}
