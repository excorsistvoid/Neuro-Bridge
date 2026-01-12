use serde::{Deserialize, Serialize};

/// Represents commands that can be sent from the client to the server.
/// These commands define the actions the client requests the server to perform.
#[derive(Serialize, Deserialize, Debug)]
pub enum BridgeCommand {
    /// A simple command to check if the server is alive and responsive.
    Ping,
    /// Command to request information about the GPU visible to the server.
    GetGpuInfo,
    // Future commands could be added here, e.g.,
    // LoadModel { path: String },
    // Inference { data: Vec<u8> },
}

/// Represents responses that the server sends back to the client.
/// These responses convey the results of the commands executed by the server.
#[derive(Serialize, Deserialize, Debug)]
pub enum BridgeResponse {
    /// Response to a `Ping` command, indicating the server is alive.
    Pong,
    /// Response to a `GetGpuInfo` command, containing details about the detected GPU.
    GpuInfo { 
        /// The name of the GPU device.
        device_name: String, 
        /// The version of the GPU driver.
        driver_version: String 
    },
    /// General error response, carrying an error message from the server.
    Error(String),
    /// Acknowledgment for commands that don't return specific data.
    Ack,
}
