use futures::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    try_join, join
};
use interprocess::local_socket::{tokio::LocalSocketStream};
use std::thread;
use std::time;
use tokio::runtime::Handle;

pub async fn main() -> anyhow::Result<()> {
    // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
    // in Rust, `match` is your concise, readable and expressive decision making construct.
    let name = "/tmp/polychat.sock";

    // Await this here since we can't do a whole lot without a connection.
    let conn = LocalSocketStream::connect(name).await?;

    //let mutex_1 = Arc::new(Mutex::new(0));
    //let mutex_2 = mutex_1.clone();

    // This consumes our connection and splits it into two halves,
    // so that we could concurrently act on both.
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);

    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.

    // Loop until EOF
    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);

    let read_async_handle = Handle::current().spawn(async move {
        let mut total_read = 0;
        loop {                        
            // Describe the read operation as reading into our big buffer.
            let read = reader.read_line(&mut buffer);
            total_read += 1;

            let read_result = read.await;

            if read_result.is_err() {
                println!("Error reading result in client. Exiting. Total read: {}", total_read);
                break;
            }

            if buffer.contains('\0') {
                println!("End acknowldged received on client side. Exiting. Total read: {}", total_read);
                break;
            } else {
                println!("SERVER -> CLIENT: {}", buffer.as_str());
                buffer.clear();
            }
        }
        drop(reader);
    });
    

    let write_async_handle = Handle::current().spawn(async move {
        for i in 0..5 {
            println!("Sending data #{} to server.", i);

            // Describe the write operation as writing our whole string.
            let to_write = format!("Hello from client #{}.\n", i);
            let write = writer.write_all(to_write.as_bytes());
            // Describe the read operation as reading until a newline into our buffer.
            //

            // Concurrently perform both operations.
            let write_join_result = try_join!(write);
            match write_join_result {
                Ok(_) => {
                    println!("Successfully wrote to server");
                },
                Err(e) => {
                    eprintln!("Error writing #{} message to server: {}", i, e);
                },
            }
            thread::sleep(time::Duration::from_millis(1000));
        }

        let write = writer.write_all(b"\0\n");
        let write_join_result = try_join!(write);
        match write_join_result {
            Ok(_) => {
                println!("Successfully wrote null (end) to server");
            },
            Err(e) => {
                eprintln!("Error writing null end message to server: {}", e);
            },
        }
    });

    let (read_join_result, write_join_result) = join!(read_async_handle, write_async_handle);
    if read_join_result.is_err() {
        eprintln!("Failed to join to read future")
    }
    if write_join_result.is_err() {
        eprintln!("Failed to join to write future")
    }

    // Display the results when we're done!

    Ok(())
}
