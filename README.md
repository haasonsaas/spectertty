# **SpecterTTY** — AI‑Native Terminal Automation Platform

*Product Specification · Draft 1.0 (refined after competitive analysis)*

---

## 1  Purpose & Positioning

SpecterTTY (working title; final branding TBD) transforms any interactive CLI session into a **structured, token‑efficient JSON event stream** purpose‑built for AI agents. It unifies real‑time PTY automation, lossless recording, and lightweight sandboxing in a single Rust binary. Competitive analysis confirms *no public tool delivers this combination*—creating a first‑mover opportunity in the AI‑agent infrastructure stack.

---

## 2  Market Gaps Addressed

| Gap                                           | How SpecterTTY Closes It                                                                                                           |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Raw, unstructured PTY output                  | Emits length‑prefixed JSON frames with semantic event types, enabling deterministic AI parsing.                                    |
| High token cost for LLMs                      | Optional **token‑optimized summariser** (e.g., line‑update delta, stripped ANSI) slashes output tokens without losing intent.      |
| Disjoint security + automation stacks         | Integrated *capsule‑run* sandbox delivers resource ceilings, ro‑bind mounts, and net toggles—no external VM or container required. |
| Lack of AI‑first developer experience         | Thin client SDKs (Python, TS, Rust) expose awaitable `Session` objects; first‑class adapters for LangChain, AutoGen, CrewAI.       |
| Weak observability in interactive automations | Built‑in asciinema recorder, golden‑transcript diff, and future breakpoint/stepper mode.                                           |

---

## 3  High‑Level Architecture

```mermaid
graph TD
A[Agent] -- JSON Frames --> G[ SpecterTTY ]
G -- execve + PTY --> C[capsule‑run (optional)]
C --> T[Target CLI]
G --> R{Recorder / Durability}
G <---> S(Socket / Stdio Transport)
```

*All transport is newline‑delimited or length‑prefixed JSON; binary payloads base64‑encoded with `binary: true` flag.*

---

## 4  Functional Requirements

### 4.1  Real‑Time PTY Automation

* Sub‑10 ms median spawn time on Linux x86‑64 (Ryzen 9 baseline).
* ≥ 45 MB/s sustained pass‑through with <60 % single‑core utilisation.
* Structured frame schema (see §5) with **idle**, **prompt**, **line\_update**, and **overflow** events.

### 4.2  Token‑Efficiency Layer

* `--token‑mode compact` — strips ANSI, collapses CR progress bars into `line_update` frames, batches stdout into ≤512‑byte chunks.
* Target ≥50 % token reduction vs raw output on representative workloads (`npm install`, `pytest -vv`, network gear configs).

### 4.3  Sandboxing via Capsule Mode

* Memory / CPU / wall‑time ceilings; JSON notifications (`capsule_kill`) on breach.
* Tmpfs root with RO system binds; per‑session writable overlay if `--persist`.
* Seccomp deny list: `ptrace`, `mount`, raw sockets (net off), `clone3` with `CLONE_NEWUSER`.

### 4.4  Session Durability & Recovery

* `--state-dir` enables PTY FD resurrection after daemon crash (similar to `tmux` attach).
* Frame log persisted via *sled* ring buffer; agents can `seek offset` on reconnect.
* Graceful handoff mechanism to migrate running sessions between nodes (v2 roadmap).

### 4.5  Observability & Debugging

* Always‑on asciinema v2 recorder (`--record FILE.cast`).
* `verify` sub‑command: diff against golden transcript; non‑zero exit on drift (CI gate).
* Future *debug* sub‑command: regex breakpoints + live stepper.

---

## 5  Frame Schema (v1)

```jsonc
{
  "ts": 1717260631.021,     // float, μs precision
  "type": "stdout" | "stdin" | "stderr" | "cursor" | "resize" |
           "resize_ack" | "prompt" | "idle" | "line_update" |
           "overflow" | "signal" | "exit" | "stopped" |
           "continued" | "capsule_kill" | "ping" | "pong",
  "data": "…",              // UTF‑8 or base64 if binary=true
  "binary": false,           // true if base64 encoded
  "cols": 120, "rows": 40, // for resize* frames
  "code": 0, "signal": "TSTP",      // exit / signal frames
  "regex": "^.+[#>] $",              // prompt matcher hit
  "dur_ms": 200,                       // idle duration
  "reason": "memory"                  // overflow / kill rationale
}
```

*The schema MUST remain backward‑compatible; new optional props only.*

---

## 6  CLI Interface

See **appendix A** for full flag table. Key additions based on analysis:

* `--token-mode {raw|compact|parsed}`
* `--prompt-regex PATTERN` (multiple allowed)
* `--sandbox-profile basic|strict|custom.toml`

---

## 7  Non‑Functional Requirements

| Category                  | Target                                                                   |                     |
| ------------------------- | ------------------------------------------------------------------------ | ------------------- |
| **Startup latency**       | ≤10 ms cold, ≤5 ms warm cache                                            |                     |
| **Throughput**            | ≥45 MB/s raw                                                             | >40 MB/s compressed |
| **Simultaneous sessions** | 1 000 idle PTYs <100 MiB RSS                                             |                     |
| **Security**              | Zero CVE high‑severity at GA; fuzz PTY parser with 10^7 cases            |                     |
| **Reliability**           | 99.95 % 30‑day uptime for long‑running daemon                            |                     |
| **Platform**              | Linux (musl static), macOS (arm64/x86‑64), experimental Windows (winpty) |                     |

---

## 8  SDK & Framework Adapters

| Framework     | Adapter                         | Status      |
| ------------- | ------------------------------- | ----------- |
| **LangChain** | `SpecterTTYTool` (Python)       | MVP in v0.2 |
| **AutoGen**   | `SpecterTTYChannel`             | backlog     |
| **CrewAI**    | Wrapper via `TaskLibrary`       | backlog     |
| **CI/CD**     | GitHub Action + GitLab Template | part of GA  |

Each adapter MUST expose:

* async `run(cmd, **opts)` returning stream/async‑iterator of frames
* optional `lease()` / `release()` for session pooling

---

## 9  Open‑Source & Licensing Plan

* License: **Apache‑2.0** with commons‑clause restriction on hosted resale (TBC).
* Public repo by **September 30 2025**; private alpha branches starting July.
* Core crates published to crates.io; Docker images on ghcr.io.

---

## 10  Milestones (High‑Level)

| ID | Deliverable                                 | Target Date    |
| -- | ------------------------------------------- | -------------- |
| M1 | Rust PTY core + JSON framing, raw mode      | **2025‑06‑30** |
| M2 | Capsule integration + overflow guard        | 2025‑07‑25     |
| M3 | Token‑efficient summariser + prompt matcher | 2025‑08‑15     |
| M4 | SDKs (Python, TS) + LangChain adapter       | 2025‑09‑05     |
| M5 | OSS launch, website, docs                   | 2025‑09‑30     |

---

## Appendix A  Full CLI Flag Reference

*(abbreviated – see `docs/cli.md` in repo for auto‑generated source‑of‑truth)*

```
spectertty [GLOBAL] -- <cmd>
GLOBAL:
  --json                       Output frames to stdout (default)
  --socket PATH                Unix socket transport
  --bind HOST:PORT             TCP transport
  --cols / --rows N            Initial window size
  --idle DUR                   Idle duration before idle frame (default 200ms)
  --token-mode MODE            raw | compact | parsed
  --prompt-regex PATTERN       Register prompt matcher (repeatable)
  --buffer SIZE                Max in‑mem queue before back‑pressure (8M)
  --overflow-timeout DUR       Grace before SIGKILL on overflow (5s)
  --record FILE.cast           asciinema v2 output
  --capsule                    Run target via capsule‑run
  --sandbox-profile PROFILE    basic | strict | custom.toml
  --state-dir DIR              Enable session resurrection
  --compress {zstd|none}       Compress frame payloads
  --help / --version
```

---

### End of Spec