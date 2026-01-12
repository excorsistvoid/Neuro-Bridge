use anyhow::Result;
use clap::{Parser, Subcommand};
use shared::{BridgeCommand, BridgeResponse};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SOCKET_PATH: &str = "/dev/socket/neuro_bridge.sock";

#[derive(Parser)]
#[command(name = "neuro")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ping,
    Gpu,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Attempt to connect to the Unix domain socket. This is a critical step
    // as it establishes the communication channel with the Neuro-Bridge server
    // running on the Android host. If the server is not running or the socket
    // path is incorrect, this connection will fail.
    let mut stream = UnixStream::connect(SOCKET_PATH).await
        .map_err(|_| anyhow::anyhow!("Failed to connect to Neuro-Bridge. Is the server running on the Android Host?"))?;

    // Prepare the command to be sent to the server based on CLI arguments.
    let cmd = match cli.command {
        Commands::Ping => BridgeCommand::Ping,
        Commands::Gpu => BridgeCommand::GetGpuInfo,
    };

    // Serialize the command into a byte array using bincode.
    // A 4-byte length prefix (big-endian) is sent before the actual payload
    // to allow the server to know how many bytes to read for the command.
    let cmd_bytes = bincode::serialize(&cmd)?;
    let cmd_len = (cmd_bytes.len() as u32).to_be_bytes();
    stream.write_all(&cmd_len).await?; // Send the length prefix
    stream.write_all(&cmd_bytes).await?; // Send the serialized command

    // Read the 4-byte length prefix of the incoming response.
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read the actual response payload based on the received length.
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    // Deserialize the response bytes back into a BridgeResponse enum.
    let response: BridgeResponse = bincode::deserialize(&buf)?;

    // Display the result of the command.
    match response {
        BridgeResponse::Pong => println!("Pong! Server is alive."),
        BridgeResponse::GpuInfo { device_name, driver_version } => {
            println!("GPU Detected via Bridge!");
            println!("   Device: {}", device_name);
            println!("   Driver: {}", driver_version);
        },
        BridgeResponse::Error(e) => eprintln!("Server Error: {}", e),
        _ => println!("Received: {:?}", response),
    }

    Ok(())
}
