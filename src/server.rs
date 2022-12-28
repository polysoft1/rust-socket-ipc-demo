use futures::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    try_join,
};
use interprocess::local_socket::{
    tokio::{LocalSocketListener, LocalSocketStream},
};
use std::io;
use tokio::sync::oneshot::Sender;

async fn handle_conn(conn: LocalSocketStream) -> io::Result<()> {
    // Split the connection into two halves to process
    // received and sent data concurrently.
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);

    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);

    // Describe the write operation as writing our whole message.
    let write = writer.write_all(b"Hello from server!\n");
    // Describe the read operation as reading into our big buffer.
    let read = reader.read_line(&mut buffer);

    // Run both operations concurrently.
    try_join!(read, write)?;

    // Dispose of our connection right now and not a moment later because I want to!
    drop((reader, writer));

    // Produce our output!
    println!("Client answered: {}", buffer.trim());
    Ok(())
}


pub async fn main(notify: Sender<()>) -> anyhow::Result<()> {
    let name = "/tmp/polychat.sock";
    let listener = LocalSocketListener::bind(name)?;

    println!("Running server at {}", name);

    let _ = notify.send(());

    let mut buffer = String::with_capacity(128);

    loop {
        let conn = match listener.accept().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Could not handle incoming connection: {}", e);
                continue;
            }
        };
        
        tokio::spawn(async move {
            if let Err(e) = handle_conn(conn).await {
                eprintln!("Error Handling conn: {}", e);
            }
        });
    }
}
