use futures::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    try_join, join,
};
use interprocess::local_socket::tokio::{
    LocalSocketListener, LocalSocketStream,
};
use std::io;
use tokio::sync::oneshot::Sender;
use std::thread;
use std::time;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::runtime::Handle;

async fn handle_conn(conn: LocalSocketStream) -> io::Result<()> {
    // Split the connection into two halves to process
    // received and sent data concurrently.
    let (reader, mut writer) = conn.into_split();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone_1 = running.clone();
    let running_clone_2 = running.clone();

    let mut reader = BufReader::new(reader);

   
    // Loop until EOF
    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);

    let read_thread_handle = Handle::current().spawn(async move {
        let mut total_read = 0;
        loop {          
            // Describe the read operation as reading into our big buffer.
            let read = reader.read_line(&mut buffer);

            let read_result = read.await;

            if read_result.is_err() {
                println!("Error reading result in server. Exiting.");
                break;
            }
            total_read += 1;

            if buffer.contains('\0') {
                running_clone_2.store(false, Ordering::Relaxed);
                println!("Reached end on server side. Total read: {}", total_read);
                break;
            } else {
                println!("CLIENT -> SERVER: {}", buffer.as_str());
                buffer.clear();
            }
        }
        drop(reader);
    });

    let write_thread_handle = Handle::current().spawn(async move {
        let mut total_written = 0;
        loop {
            if !running_clone_1.load(Ordering::Relaxed) {
                println!("Reached end. Writing acknowledgement and exiting. Total written: {}", total_written);
                let write = writer.write_all(b"\0\n");
                let write_result = try_join!(write);
                if write_result.is_err() {
                    eprintln!("Error writing final ackknowledgement");
                }
                break;
            }
            println!("Writing from server to client");
            let to_write = format!("Hello from server #{}.\n", total_written);
            let write = writer.write_all(to_write.as_bytes());
            // The other process is waiting for this, so wait.
            let write_result = try_join!(write);
            if write_result.is_err() {
                eprintln!("Error writing");
            }
            total_written += 1;

            thread::sleep(time::Duration::from_millis(500));
        }
    });
    
    let (read_join_result, write_join_result) = join!(read_thread_handle, write_thread_handle);
    if read_join_result.is_err() {
        eprintln!("Failed to join to read future")
    }
    if write_join_result.is_err() {
        eprintln!("Failed to join to write future")
    }

    // Produce our output!
    Ok(())
}


pub async fn main(notify: Sender<()>) -> anyhow::Result<()> {
    let name = "/tmp/polychat.sock";

    let listener = LocalSocketListener::bind(name)?;

    println!("Running server at {}", name);

    let _ = notify.send(());

    //loop {
    let conn = match listener.accept().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not handle incoming connection: {}", e);
            return anyhow::Result::Err(anyhow::Error::msg("Could not handle incoming connection"));
        }
    };
    
    
    if let Err(e) = handle_conn(conn).await {
        eprintln!("Error Handling conn: {}", e);
    }

    anyhow::Result::Ok(())
    //}
}
