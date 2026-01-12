use anyhow::Result;
use ash::{vk, Entry};
use shared::{BridgeCommand, BridgeResponse};
use std::ffi::CStr;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SOCKET_PATH: &str = "/dev/socket/neuro_bridge.sock";

/// Checks for GPU information using the Vulkan API.
///
/// This function is marked `unsafe` because it directly interacts with the Vulkan C API
/// via the `ash` crate. Incorrect usage of Vulkan functions can lead to crashes,
/// memory corruption, or undefined behavior.
///
/// It loads the Vulkan library, creates an instance, enumerates physical devices,
/// retrieves properties of the first detected GPU, and then destroys the Vulkan instance.
fn check_gpu() -> Result<(String, String)> {
    unsafe {
        // Attempt to load the Vulkan library dynamically. On Android, this typically
        // resolves to /system/lib64/libvulkan.so.
        let entry = Entry::load()?;
        
        // Define application information for the Vulkan instance.
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CStr::from_bytes_with_nul(b"NeuroBridge\0")?)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        // Create a Vulkan instance. This is the first step to using Vulkan.
        let create_info = vk::InstanceCreateInfo::builder().application_info(&app_info);
        let instance = entry.create_instance(&create_info, None)?;

        // Enumerate physical devices (GPUs) available to the system.
        let pdevices = instance.enumerate_physical_devices()?;
        if let Some(pdevice) = pdevices.first() {
            // Get properties of the first physical device found, including its name and driver version.
            let props = instance.get_physical_device_properties(*pdevice);
            
            // Convert the C-style device name to a Rust String.
            let device_name = CStr::from_ptr(props.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();
            
            // Convert the driver version to a String.
            let driver_ver = props.driver_version.to_string();
            
            // IMPORTANT: Destroy the Vulkan instance to release resources.
            // Failure to do so can lead to resource leaks.
            instance.destroy_instance(None);
            
            Ok((device_name, driver_ver))
        } else {
            // If no physical device is found, report it.
            Ok(("No GPU Found".to_string(), "0".to_string()))
        }
    }
}

/// Handles an incoming client connection over a Unix domain socket.
///
/// This function continuously reads commands from the client, processes them,
/// and sends back responses. It implements a simple length-prefixed messaging
/// protocol for robust communication.
async fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut len_buf = [0u8; 4];
    
    loop {
        // First, attempt to read the 4-byte length prefix of the incoming message.
        // If the read fails (e.g., client disconnects), return Ok(()) to terminate
        // this handler gracefully.
        if stream.read_exact(&mut len_buf).await.is_err() {
            return Ok(()); // Connection closed by client
        }
        // Convert the 4 bytes into a u32 representing the message length (big-endian).
        let len = u32::from_be_bytes(len_buf) as usize;

        // Allocate a buffer of the exact size specified by the length prefix and
        // read the message body into it.
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        // Deserialize the received bytes into a BridgeCommand enum.
        let command: BridgeCommand = bincode::deserialize(&buf)?;
        println!("[Server] Received: {:?}", command);

        // Process the received command and generate an appropriate response.
        let response = match command {
            BridgeCommand::Ping => BridgeResponse::Pong,
            BridgeCommand::GetGpuInfo => {
                match check_gpu() {
                    Ok((name, ver)) => BridgeResponse::GpuInfo { device_name: name, driver_version: ver },
                    // If GPU check fails, send an error response to the client.
                    Err(e) => BridgeResponse::Error(e.to_string()),
                }
            }
        };

        // Serialize the response back into bytes.
        let resp_bytes = bincode::serialize(&response)?;
        // Prepare the 4-byte length prefix for the response.
        let resp_len = (resp_bytes.len() as u32).to_be_bytes();
        
        // Send the length prefix, followed by the serialized response.
        stream.write_all(&resp_len).await?;
        stream.write_all(&resp_bytes).await?;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("[Neuro-Bridge] Server Starting on Android Host...");

    // IMPORTANT: Clean up any old socket file that might exist from a previous
    // run. If the socket file exists and is not removed, `bind` will fail.
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }

    // Bind the Unix domain socket. This creates the socket file and starts
    // listening for incoming connections.
    let listener = UnixListener::bind(SOCKET_PATH)?;
    
    // CRITICAL SECURITY/PERMISSIONS STEP:
    // Set file permissions for the Unix domain socket to allow all users (0o777)
    // to read and write. This is essential for the client running in the chroot
    // environment to be able to connect to the server on the Android host.
    // Without this, connection attempts from chroot will likely fail due to
    // permission denied errors. This grants broad access, so ensure only
    // trusted clients can connect or implement further authentication if needed.
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o777))?;

    println!("[Neuro-Bridge] Listening on {}", SOCKET_PATH);

    // Main server loop: accept incoming connections indefinitely.
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // For each new connection, spawn a new asynchronous task to
                // handle it concurrently. This allows the server to manage
                // multiple clients simultaneously without blocking.
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("Client handler error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}