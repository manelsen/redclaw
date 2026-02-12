# RedClaw 🦀 — The 2MB RAM AI Agent

**RedClaw** is an ultra-efficient, memory-safe personal AI agent written in Rust. It is specifically designed to run on the most resource-constrained hardware imaginable, such as $5 RISC-V or ARM SoCs, with a target footprint of **less than 2MB of RAM**.

Inspired by the pioneering work of **OpenClaw** and **nanobot**, and directly based on the architecture of **PicoClaw**, RedClaw represents the "bit-golfing" extreme of the AI agent world.

---

## 📊 Comparison: The Path to 2MB

| Feature | OpenClaw | nanobot | PicoClaw | RedClaw 🦀 |
| :--- | :--- | :--- | :--- | :--- |
| **Language** | Python / Node.js | Go | Go | **Rust** (Stable) |
| **RAM Target** | ~100MB+ | ~50MB | ~15MB | **< 2MB** |
| **Binary Size** | N/A (Scripts) | ~10MB | ~8MB | **~650KB** |
| **Networking** | Native HTTP | Native HTTP | Native HTTP | **Curl Offload** |
| **Memory Management** | GC-based | GC-based | GC-based | **Zero-Allocation** |
| **Target Hardware** | Desktop / Cloud | Desktop / Server | LicheeRV (256MB) | **Milk-V Duo (64MB)** |

---

## 🧠 System Logic

RedClaw follows a **ReAct (Reasoning + Acting)** pattern, but optimizes every step for embedded survival:

1.  **Exoskeleton (Rust Core)**: Provides a memory-safe, statically linked binary that avoids the overhead of a Garbage Collector or heavy runtimes.
2.  **The Brain (Provider Logic)**: Communicates with OpenRouter, Gemini, or OpenAI using a **Streaming JSON Parser**. Instead of loading full API responses into RAM, RedClaw parses them byte-by-byte.
3.  **Curl Offloading**: Networking and TLS (encryption) are the biggest RAM consumers. RedClaw offloads these to the system's `curl` process, using pipes (STDIN/STDOUT) to stream data. This keeps the agent's RSS minimal.
4.  **Memory Persistence**: Uses a simple flat-file system. Long-term context is stored in `MEMORY.md`, while daily logs are organized by date. No Vector DB required.
5.  **Safety Guards**: Includes a substring-based blacklist for shell commands and strict buffer limits (256KB) for file operations to prevent OOM (Out of Memory) crashes.
6.  **Malloc Trim**: After every interaction, RedClaw forces the OS to reclaim unused heap memory using `malloc_trim`.

---

## 🛠️ Installation & Build

### Prerequisites
- **Rust** (Latest Stable)
- **curl** (Installed on the target system)

### Quick Start
```bash
make          # Compiles and creates the ./redclaw binary
./redclaw onboard  # Interactive wizard to set up your API keys
```

---

## 📖 Usage Examples

### Interactive Mode (Terminal)
```bash
./redclaw -i
```
*Visual output includes a sleek "envelope" style with real-time RAM telemetry.*

### Telegram Bot Mode
```bash
./redclaw -t
```
*Turn your hardware into a 24/7 autonomous bot.*

### Single Instruction
```bash
./redclaw -m "Check my disk space and summarize it."
```

---

## ⚙️ Configuration

RedClaw prefers **OpenRouter** for its high compatibility and stability, but supports any OpenAI-compatible endpoint.

```json
{
  "agents": {
    "defaults": {
      "workspace": "./workspace",
      "model": "google/gemini-2.0-flash-exp:free"
    }
  },
  "providers": {
    "openrouter": {
      "api_key": "sk-or-v1-..."
    }
  }
}
```

---

## 🤝 Contributing

We are in a constant state of "Bit Golfing". Contributions should focus on:
- Reducing binary size.
- Minimizing heap allocations.
- Enhancing tool safety.

## 📜 License

RedClaw is released under the **MIT License**.

## 🙏 Credits

- **[OpenClaw](https://github.com/OpenClaw)**: For the inspiration of open AI agents.
- **[nanobot](https://github.com/HKUDS/nanobot)**: For the original lightweight agent concepts.
- **[PicoClaw](https://github.com/sipeed/picoclaw)**: For the architecture and embedded focus.

---
*Built with precision in Rust for the embedded frontier.*
