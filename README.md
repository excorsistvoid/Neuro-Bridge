# Neuro-Bridge

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Android%20(Root)-green.svg)]()
[![Architecture](https://img.shields.io/badge/Arch-Client%2FServer-blue.svg)]()
[![EAISD](https://img.shields.io/badge/Org-EAISD-purple.svg)](https://github.com/EAISD)

**Neuro-Bridge** is a high-performance IPC (Inter-Process Communication) Bridge designed to break hardware isolation in Linux Chroot/Proot environments on Android.

This project is not limited to a single specific function. Neuro-Bridge acts as a **"Hardware Proxy"**, enabling processes within a Linux container (Ubuntu/Debian) to offload compute workloads to native Android Host drivers (Vulkan/Adreno/Hexagon DSP).

This opens up unlimited possibilities: from AI Inference, General Purpose GPU (GPGPU), image processing, to graphical rendering experiments, without being hindered by kernel driver incompatibilities (KGSL vs DRM).

---

## Core Philosophy

**"The Brain in the Box, The Muscle on the Metal."**

The Chroot environment (The Box) is excellent for software development but is blind to hardware. The Android Host (The Metal) has full hardware access but is limited in software. **Neuro-Bridge** unites these two.

---

## Architecture

Neuro-Bridge works by separating *Request* from *Execution*:

1.  **Server (The Host Node)**
    *   Runs native on Android (`aarch64-linux-android`).
    *   Exposes access to:
        *   **Vulkan Compute** (via `ash` / `wgpu`).
        *   **OpenCL** (if available in vendor lib).
        *   **Neural Networks API (NNAPI)**.
    *   Acts as an "Executor" that receives raw data/commands.

2.  **Client (The Chroot Node)**
    *   A library/CLI binary that runs in Linux Chroot (`aarch64-unknown-linux-gnu`).
    *   Packages instructions and data, sends them over a socket, and waits for results.
    *   Can be integrated into other Python, C++, or Rust scripts.

3.  **The Pipeline**
    *   Communication via **Unix Domain Socket** (`/dev/socket/neuro_bridge.sock`).
    *   Designed for high throughput (low latency serialization).

---

## Potential Use Cases

Due to its universal nature, Neuro-Bridge can be developed for:

*   **AI/ML Acceleration:** Running ONNX/TFLite models using Adreno GPU (similar to NCNN/MNN but via bridge).
*   **GPGPU Tasks:** Performing heavy mathematical calculations (matrix multiplication, crypto operations) on the GPU.
*   **Image Processing:** Sending raw bitmaps for processing by Android's ISP/DSP.
*   **Video Transcoding:** (Experimental) Accessing Android's hardware encoder/decoder.
*   **Custom Driver Implementation:** Creating "Virtual Drivers" on the Chroot side that offload rendering instructions to the Host.

---

## Getting Started

### Prerequisites
*   Rooted Android Device (Snapdragon series recommended for Adreno/Hexagon support).
*   Rust Toolchain (with `aarch64-linux-android` and `aarch64-unknown-linux-gnu` targets).

### Build

```bash
# Clone Repo
git clone https://github.com/EAISD/Neuro-Bridge.git

# Build Server (Host Side)
cargo build --release --bin neuro_server --target aarch64-linux-android

# Build Client (Chroot Side)
cargo build --release --bin neuro_client --target aarch64-unknown-linux-gnu
```

---

## Protocol Overview

Neuro-Bridge uses a flexible binary protocol (`bincode`). The command structure can be extended as needed by installed modules.

```rust
// Example of flexible structure
pub enum BridgeCommand {
    // Basic Diagnostic
    Ping,
    GetHardwareInfo,
    
    // Generic Compute Payload
    ExecuteCompute { 
        module_id: String, // e.g., "ai_engine" or "math_core"
        payload: Vec<u8>   // Raw data
    },
    
    // Future Expansion
    AllocateMemory { size: usize },
    WriteBuffer { id: u32, data: Vec<u8> },
}
```

---

## Disclaimer

**Neuro-Bridge** is experimental low-level software.
*   Directly accessing hardware drivers can cause kernel panics or reboots if incorrect instructions (Malformed Instructions) are sent.
*   Use with caution on production devices.

---

**EAISD** - *Experiment Artificial Intelligent Software Development*
