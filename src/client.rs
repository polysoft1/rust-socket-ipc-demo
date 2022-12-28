use futures::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    try_join,
};
use interprocess::local_socket::{tokio::LocalSocketStream, NameTypeSupport};

pub async fn main() -> anyhow::Result<()> {
    // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
    // in Rust, `match` is your concise, readable and expressive decision making construct.
    let name = "/tmp/polychat.sock";

    // Await this here since we can't do a whole lot without a connection.
    let conn = LocalSocketStream::connect(name).await?;

    // This consumes our connection and splits it into two halves,
    // so that we could concurrently act on both.
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);

    // Allocate a sizeable buffer for reading.
    // This size should be enough and should be easy to find for the allocator.
    let mut buffer = String::with_capacity(128);

    // Describe the write operation as writing our whole string.
    let write = writer.write_all(b"Hello from client!\n");
    // Describe the read operation as reading until a newline into our buffer.
    let read = reader.read_line(&mut buffer);

    // Concurrently perform both operations.
    try_join!(write, read)?;


    // Close the connection a bit earlier than you'd think we would. Nice practice!
    drop((reader, writer));

    // Describe the write operation as writing our whole string.
    println!("Server answered: {}", buffer.trim());

    // Display the results when we're done!

    Ok(())
}
