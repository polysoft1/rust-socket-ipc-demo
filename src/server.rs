use std::{
    io::{self, BufReader, BufRead, Write},
    sync::mpsc::Sender, process::exit
};

use anyhow::Context;
use interprocess::local_socket::{LocalSocketStream, LocalSocketListener};

fn handle_conn_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Failed incoming connection: {}", e);
            None
        }
    }
}

pub fn main(notify: Sender<()>) -> anyhow::Result<()> {
    let name = "/tmp/polychat.sock";
    let listener = match LocalSocketListener::bind(name) {
        Err(e) => {
            eprintln!("Could not start server: {}", e);
            exit(1);
        }
        Ok(x) => x,
    };

    println!("Running server at {}", name);

    let _ = notify.send(());

    let mut buffer = String::with_capacity(128);

    for conn in listener.incoming().filter_map(handle_conn_error) {
        let mut conn = BufReader::new(conn);
        println!("Incoming connection");

        conn.read_line(&mut buffer).context("Failed to receive socket")?;

        conn.get_mut().write_all(b"Hello from the servah!\n")?;
        print!("Client answered: {}", buffer);

        if buffer == "stop\n" {
            break;
        }

        buffer.clear();
    }

    Ok(())
}
