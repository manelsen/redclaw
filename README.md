# RedClaw 🦀

**RedClaw** is an ultra-lightweight personal AI agent written in Rust, specifically optimized for embedded systems and hardware with extremely limited resources (target: **< 2MB RAM**).

Inspired by [OpenClaw](https://github.com/OpenClaw) and directly based on the architecture of [PicoClaw](https://github.com/sipeed/picoclaw), RedClaw brings the power of LLMs to the smallest Linux-based boards (RISC-V/ARM) with a focus on memory safety, zero-cost abstractions, and operational security.

## 🚀 Features

- **Extreme Portability**: Statically linked binary support via `musl` for ARM64 (Android/Termux) and RISC-V.
- **Resource Optimized**: 
  - Binary size: ~1.9MB (optimized release).
  - RAM usage: Strict memory limits (256KB file read buffers) to fit in 2MB total system RAM.
- **Multi-Provider Support**: Native compatibility with OpenAI, Google Gemini (via compatibility layer), OpenRouter, and Zhipu.
- **Persistent Flat-File Memory**: Uses simple Markdown files for long-term memory and daily logs.
- **Safety Guards**: 
  - **Command Guard**: Built-in blacklist for dangerous shell commands (e.g., `rm -rf`).
  - **RAM Guard**: Automatic truncation of large files and command outputs.
- **Tool System**: Built-in capabilities for file manipulation, shell execution, and web search/fetch.

## 🛠️ Installation

### Prerequisites
- **Rust** (Latest Stable)
- **Target Toolchains** (Optional for cross-compilation):
  ```bash
  rustup target add aarch64-unknown-linux-musl # For ARM64/Android
  rustup target add x86_64-unknown-linux-musl  # For Static x86
  ```

### Build
To build the optimized production binary:
```bash
cargo build --release
```
The binary will be available at `target/release/redclaw`.

## ⚙️ Configuration

Copy the example configuration and add your API keys:
```bash
cp config.example.json config.json
```

Example `config.json` for Gemini:
```json
{
  "agents": {
    "defaults": {
      "workspace": "./workspace",
      "model": "gemini-1.5-flash"
    }
  },
  "providers": {
    "gemini": {
      "api_key": "YOUR_GEMINI_API_KEY"
    }
  }
}
```

## 📖 Usage

### Interactive Mode
Engage in a continuous session with the agent:
```bash
./redclaw -i
```

### Single Command
Send a direct instruction:
```bash
./redclaw -m "Read technical_specs.md and summarize the memory constraints."
```

### Telegram Bot Mode
Run the agent as a Telegram bot (requires token in config.json):
```bash
./redclaw -t
```

## 🤝 Contribution

Contributions are welcome! Since RedClaw focuses on low-resource optimization:
1. Ensure new features do not significantly increase the binary size.
2. Prefer `std` and zero-dependency implementations where possible.
3. Maintain the memory-safe philosophy (no `unsafe` blocks without strict justification).

### Development
1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'Add amazing feature'`).
4. Push to the branch (`git push origin feature/amazing-feature`).
5. Open a Pull Request.

## 📜 License

This project is licensed under the MIT License - see the `LICENSE` file for details.

## 🙏 Credits & Inspiration

- **[PicoClaw](https://github.com/sipeed/picoclaw)**: The original Go implementation that defined the lightweight agent loop and tool registry.
- **[OpenClaw](https://github.com/OpenClaw)**: For the inspiration of open, community-driven AI agents for all.

---
*Built with ❤️ in Rust for the embedded world.*
