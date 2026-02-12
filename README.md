# RedClaw 🦀 — The 2MB RAM AI Agent

**RedClaw** is an ultra-lightweight, memory-safe personal AI agent written in Rust. It is specifically designed to run on the most resource-constrained hardware imaginable, such as $5 RISC-V or ARM SoCs, with a target footprint of **less than 2MB of RAM**.

Inspired by the pioneering work of **OpenClaw** and **nanobot**, and directly based on the architecture of **PicoClaw**, RedClaw represents the "bit-golfing" extreme of the AI agent world.

---

## 📊 Comparison: The Path to Efficiency

| Feature | [OpenClaw](https://github.com/OpenClaw/OpenClaw) | [nanobot](https://github.com/HKUDS/nanobot) | [PicoClaw](https://github.com/sipeed/picoclaw) | **RedClaw** 🦀 |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Goal** | Feature Richness | **Code Simplicity** | **Embedded Hardware** | **Bit-Golfing Efficiency** |
| **Language** | Node.js | Python | Go | **Rust** |
| **Code Size** | ~430k+ LOC | **~4k LOC** | ~6k LOC | **~1.3k LOC** |
| **RAM Usage** | ~1GB - 2GB+ | ~45MB - 100MB | **< 10MB** | **~2.2MB (RSS)** |
| **Binary Size** | N/A (Scripts) | N/A (Scripts) | ~8MB | **~650KB** |
| **Networking** | Native Node SDKs | Native Python Libs | Native Go `http` | **Curl Offload** |
| **Hardware** | Cloud / Desktop | Server / Desktop | LicheeRV (256MB) | **Tiny SoC (8MB+)** |

---

## 🧠 System Logic: How we hit 2MB

RedClaw follows a **ReAct (Reasoning + Acting)** pattern, but optimizes every step for embedded survival:

1.  **Exoskeleton (Rust Core)**: By using Rust with no Garbage Collector and `panic = "abort"`, we eliminate the runtime overhead that costs megabytes in Go or Python.
2.  **Curl Offloading**: Networking and TLS (encryption) are the biggest RAM consumers. RedClaw offloads these to the system's `curl` process via pipes (STDIN/STDOUT). This moves the ~2MB handshake buffers out of our process.
3.  **Streaming JSON Parser**: Instead of loading full API responses into RAM, RedClaw uses `serde_json::from_reader` to parse data byte-by-byte as it comes from the network.
4.  **Bit Golfing Memory**: 
    *   **Malloc Trim**: Forces the OS to reclaim unused heap memory after every interaction.
    *   **History Truncation**: Intelligently maintains a rolling history of 6-10 messages, ensuring tool-call sequences are never broken (preventing Gemini Error 400).
    *   **Flat-File Memory**: No Vector DB. Context is stored in simple Markdown files (`MEMORY.md`), read only when necessary.
5.  **Safety First**: Even at 2MB, security is not optional. Includes a substring-based blacklist for shell commands and a 256KB read limit for files.

---

## 🛠️ Installation & Build

### Prerequisites
- **Rust** (Latest Stable)
- **curl** (Installed on your Linux system)

### Quick Start
```bash
make               # Builds the optimized binary at ./redclaw
./redclaw onboard  # Starts the configuration wizard
```

---

## 📖 Usage Examples

### Interactive Mode (Terminal)
```bash
./redclaw -i
```
*Features a sleek "envelope" visual style with real-time RSS telemetry.*

### Telegram Bot Mode
```bash
./redclaw -t
```
*Transform your $5 board into a 24/7 autonomous assistant.*

### Single Instruction
```bash
./redclaw -m "Read technical_specs.md and summarize the constraints."
```

---

## ⚙️ Configuration

RedClaw is provider-agnostic but highly recommends **OpenRouter** for its stability and normalized OpenAI-compatible output.

```json
{
  "agents": {
    "defaults": {
      "workspace": "./workspace",
      "model": "arcee-ai/trinity-large-preview:free"
    }
  },
  "providers": {
    "openrouter": { "api_key": "sk-or-v1-..." }
  }
}
```

---

## 🤝 Contributing

We are in a constant state of "Bit Golfing". Contributions should focus on reducing footprint and enhancing safety.

## 📜 License

RedClaw is released under the **MIT License**.

---
*Built with ❤️ in Rust for the embedded frontier.*
