# Troc

[![CI](https://github.com/curiousmindflow/troc/actions/workflows/ci.yml/badge.svg)](https://github.com/curiousmindflow/troc/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://curiousmindflow.github.io/troc/coverage-badge.json)](https://curiousmindflow.github.io/troc/coverage/tarpaulin-report.html)

Troc is a Data Distribution Service implementation in Rust, using [sans-io](https://sans-io.readthedocs.io/) architecture pattern for maximum testability, flexibility, reliability and performance.

## About DDS
Data Distribution Service (DDS) is a middleware protocol for real-time, data-centric publish-subscribe communication. It's widely used in:
- Robotics (ROS 2 uses DDS underneath)
- Defense systems
- Aeronautics
- Autonomous vehicles  
- Industrial IoT
- Telecommunications

DDS provides discovery, QoS policies, and reliable delivery without centralized brokers. It's a completely distributed applicative protocol.

## Why Troc ?
Existing Rust DDS implementations (RustDDS, DustDDS) couple protocol logic with I/O infrastructure. Troc separates these concerns using the sans-io pattern:

**Sans-io** means the protocol core operates on messages and state machines without directly performing I/O operations. Instead, it returns "effects" that the realization layer executes.

**Benefits for DDS:**
- **Testability** - Complex RTPS state machines tested with deterministic inputs, no real network needed
- **Mutation testing** - Validate test suite quality catches protocol bugs
- **Flexibility** - Same core can target Tokio, async-std, embedded systems, eventually bare metal
- **Debuggability** - Protocol logic isolated from concurrency and timing issues
- **Performance** - Lock-free core, zero I/O overhead in protocol decisions

**Target:** Sub 50μs latency through careful design and zero-copy operations where possible.

## Project status
⚠️ **Experimental / Work in Progress**

**Current state:**
  - ✅ Sans-io core with basic RTPS support
  - 🚧 Tokio-based realization layer
  - 📋 Production examples
  - 📋 More complete QoS support


**This is an early-stage exploration, not production-ready.**

See [ROADMAP.md](ROADMAP.md) for planned milestones and experimental features like:
- Gossip-based discovery as an alternative to multicast SPDP
- io_uring integration for performance
- Embedded/no_std support

## Technology Stack
Complete implementation in Rust.

- **Core:** Pure Rust, sans-io, lock-free state machines
- **Realization:** Tokio async runtime, Kameo actor framework
- **Testing:** Unit tests (rstest), property-based testing (proptest), mutation testing (cargo-mutants), fuzzing (cargo-fuzz)

## Architecture
See [ARCHITECTURE.md](ARCHITECTURE.md) for:
- Detailed explanation of the sans-io pattern
- Layer separation (core vs realization)
- Test strategies for each layer
- Performance considerations

## Changelog
See [CHANGELOG.md](CHANGELOG.md).

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md).

## License
Apache 2.0 / MIT
