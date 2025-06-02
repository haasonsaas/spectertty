# SpecterTTY

**AI-Native Terminal Automation Platform**

Transform any interactive CLI session into structured, token-efficient JSON events purpose-built for AI agents.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

---

## ✨ Features

- **🎯 AI-Optimized Output**: Structured JSON frames with semantic event types for deterministic AI parsing
- **⚡ High Performance**: Sub-10ms spawn times, high-throughput PTY handling
- **🗜️ Token Efficiency**: ANSI stripping, progress bar optimization, output batching for reduced LLM costs
- **📹 Session Recording**: Built-in asciinema v2 recording for session replay and analysis
- **🔧 Comprehensive CLI**: Full-featured command-line interface with extensive configuration options

---

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/haasonsaas/spectertty.git
cd spectertty
cargo build --release
```

### Basic Usage

```bash
# Run command with JSON output
./target/release/spectertty --json echo "Hello World"

# With token optimization
./target/release/spectertty --token-mode compact -- npm install

# With session recording
./target/release/spectertty --json --record session.cast -- ls -la

# Multiple commands
./target/release/spectertty --json -- bash -c "echo 'Step 1'; sleep 1; echo 'Step 2'"
```

---

## 📊 JSON Frame Schema

SpecterTTY outputs structured JSON frames for each terminal event:

```json
{
  "ts": 1748846947.429619,
  "type": "stdout",
  "data": "Hello from SpecterTTY!\n"
}
```

### Frame Types

| Type | Description |
|------|-------------|
| `stdout` | Standard output from the command |
| `stderr` | Standard error from the command |
| `stdin` | Input sent to the command |
| `exit` | Command exit with status code |
| `idle` | No activity for specified duration |
| `line_update` | Progress bar or dynamic content updates |
| `resize` | Terminal window size changes |

### Complete Frame Schema

```typescript
interface Frame {
  ts: number;           // Timestamp (seconds since epoch)
  type: FrameType;      // Event type
  data?: string;        // UTF-8 content or base64 if binary=true
  binary?: boolean;     // True if data is base64 encoded
  cols?: number;        // Terminal columns (resize events)
  rows?: number;        // Terminal rows (resize events)
  code?: number;        // Exit code (exit events)
  signal?: string;      // Signal name (signal events)
  dur_ms?: number;      // Duration in milliseconds (idle events)
  reason?: string;      // Reason for event (overflow/kill events)
}
```

---

## 🛠️ CLI Reference

```bash
spectertty [OPTIONS] <COMMAND> [ARGS]...
```

### Key Options

| Flag | Description | Default |
|------|-------------|---------|
| `--json` | Output JSON frames to stdout | `false` |
| `--token-mode <MODE>` | Token processing: `raw`, `compact`, `parsed` | `raw` |
| `--record <FILE>` | Record session to asciinema file | None |
| `--cols <N>` | Terminal columns | `120` |
| `--rows <N>` | Terminal rows | `40` |
| `--idle <MS>` | Idle timeout in milliseconds | `200` |
| `--prompt-regex <PATTERN>` | Prompt detection pattern (repeatable) | None |
| `--verbose` | Enable verbose logging | `false` |

### Token Processing Modes

- **`raw`**: Output frames as-is with no processing
- **`compact`**: Strip ANSI codes, batch output, optimize for token efficiency
- **`parsed`**: Advanced processing with semantic understanding (future)

---

## 🎯 Use Cases

### AI Agent Integration

```bash
# AI agent running a deployment
spectertty --json --token-mode compact -- kubectl apply -f deployment.yaml

# AI agent debugging an application  
spectertty --json --record debug-session.cast -- docker logs app-container

# AI agent running tests
spectertty --json --token-mode compact -- npm test
```

### Session Analysis

```bash
# Record interactive session for later analysis
spectertty --record troubleshooting.cast --json bash

# Compare outputs across runs
spectertty --json -- pytest tests/ > run1.json
spectertty --json -- pytest tests/ > run2.json
```

### CI/CD Integration

```bash
# Structured output for build pipelines
spectertty --json --token-mode compact -- make build

# Record deployment sessions
spectertty --record "deploy-$(date +%Y%m%d).cast" --json -- ./deploy.sh
```

---

## 🏗️ Architecture

SpecterTTY uses a multi-threaded architecture for optimal performance:

1. **PTY Management**: Real-time pseudo-terminal handling with `portable-pty`
2. **Frame Processing**: Configurable output processing for token optimization
3. **Recording**: Concurrent asciinema v2 recording
4. **Signal Handling**: Graceful shutdown and process management

---

## 🔮 Roadmap

- [x] **Core PTY automation with JSON frames**
- [x] **Token-efficient processing modes** 
- [x] **asciinema recording**
- [ ] **Sandboxing integration** (capsule-run)
- [ ] **Session durability** (state persistence)
- [ ] **Network transport** (Unix sockets, TCP)
- [ ] **Python/TypeScript SDKs**
- [ ] **AI framework adapters** (LangChain, AutoGen, CrewAI)

---

## 🤝 Contributing

SpecterTTY is built for the AI agent ecosystem. Contributions welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Repository**: https://github.com/haasonsaas/spectertty
- **Issues**: https://github.com/haasonsaas/spectertty/issues
- **Discussions**: https://github.com/haasonsaas/spectertty/discussions

---

*Built with ❤️ for the AI agent revolution*