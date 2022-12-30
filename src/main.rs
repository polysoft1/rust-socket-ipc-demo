mod server;
mod client;

use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (snd, rcv) = oneshot::channel();
    let a = async {
        if let Err(e) = server::main(snd).await {
            eprintln!("Server exited early with error: {:#}", e);
        }
    };
    let b = async {
        if rcv.await.is_ok() {
            if let Err(e) = client::main().await {
                eprintln!("Client exited early with error: {:#}", e);
            }
        }
    };
    tokio::join!(a, b);
    match std::fs::remove_file("/tmp/polychat.sock") {
        Err(e) => eprintln!("Could not cleanup socket: {}", e),
        Ok(()) => println!("Successfully cleaned up!"),
    }
}