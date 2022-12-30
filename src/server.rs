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
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};


fn get_time_ms() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

async fn handle_conn(conn: LocalSocketStream) -> io::Result<()> {
    // Split the connection into two halves to process
    // received and sent data concurrently.
    let (reader, mut writer) = conn.into_split();
    //let mutex_1 = Arc::new(Mutex::new(0));
    //let mutex_2 = mutex_1.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone_1 = running.clone();
    let running_clone_2 = running.clone();

    //let write = writer.write_all(b"First message sent from server to client\n");
    // The other process is waiting for this, so wait.
    //try_join!(write)?;

    let mut reader = BufReader::new(reader);

   
    // Loop until EOF
    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);

    let read_thread_handle = async {
        let mut total_read = 0;
        loop {          
            //let guard = mutex_1.lock();              
            // Describe the read operation as reading into our big buffer.
            let read = reader.read_line(&mut buffer);

            let read_result = read.await;
            //drop(guard);

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
                println!("{} Got from client: {}", get_time_ms(), buffer.as_str());
                buffer.clear();
            }
        }
        drop(reader);
    };

    let write_thread_handle = async {
        let mut total_written = 0;
        loop {
            //let guard = mutex_2.lock();            
            if !running_clone_1.load(Ordering::Relaxed) {
                println!("Reached end. Writing acknowledgement and exiting. Total written: {}", total_written);
                let write = writer.write_all(b"\0\n");
                let write_result = try_join!(write);
                if write_result.is_err() {
                    eprintln!("Error writing final ackknowledgement");
                }
                break;
            }
            println!("Writing from server to client at {}.", get_time_ms());
            let write = writer.write_all(b"Server -> client. End not acknowledged yet.\n");
            // The other process is waiting for this, so wait.
            let write_result = try_join!(write);
            if write_result.is_err() {
                eprintln!("Error writing");
            }
            total_written += 1;
            //drop(guard);

            thread::sleep(time::Duration::from_millis(500));
        }
    };
    
    join!(write_thread_handle, read_thread_handle);

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
