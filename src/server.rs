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

    let write = writer.write_all(b"First message sent from server to client\n");
    // The other process is waiting for this, so wait.
    try_join!(write)?;

    // Loop until EOF
    loop {
        println!("Reading (in server code) from client");
        
        // Describe the write operation as writing our whole message.
        
        // Describe the read operation as reading into our big buffer.
        let read = reader.read_line(&mut buffer);

        try_join!(read)?; // Done with read

        if buffer.contains('\0') {
            break;
        } else {
            println!("Got from client: {}", buffer.as_str());
            buffer.clear();
        }
    }

    // Dispose of our connection right now and not a moment later because I want to!
    drop((reader, writer));

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
    
    tokio::spawn(async move {
        if let Err(e) = handle_conn(conn).await {
            eprintln!("Error Handling conn: {}", e);
        }
    });

    anyhow::Result::Ok(())
    //}
}
