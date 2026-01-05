# Overview
Troc is structured as a layered architecture separating protocol logic from I/O and Time.

The project is composed of:
- **troc-core**: [Sans-io](https://sans-io.readthedocs.io/) DDS protocol logic
- **troc-cdr**: currently empty, placeholder for future implementation
- **troc-messaging**: currently empty, placeholder for future implementation
- **troc-derive**: Derive macro handling Keyed structures
- **troc**: The Realization layer bridging core logic with I/O, Time, multithreading and concurrency, using Tokio

# Troc-core
This is the layer that focus on the DDS procotol logic, using [sans-io](https://sans-io.readthedocs.io/).

## What is [sans-io](https://sans-io.readthedocs.io/) ?
If you don't know about [sans-io](https://sans-io.readthedocs.io/), the term might seems weird at first, after all, how a network protocol could do it's job without network in the first place ?
The purpose of [sans-io](https://sans-io.readthedocs.io/) architectural pattern is to separate the protocol logic from the infrastructure logic. By infrastructure, i mean socket, multithreading, concurrency, time, message passing etc...
This lead to a protocol component that is highly testable and reliable.

> **Time is abstracted**: troc-core uses i64 as a replacement for a Timestamp
> Every time related input can then be fed with deterministic values

## Why using it for Troc ?
Troc is a DDS implementation and thus has a stateful and complex protocol logic. Using [sans-io](https://sans-io.readthedocs.io/) for troc-core leverage all the benefits of this pattern.

Troc-core can be fully tested easily without touching network or even time. All inputs of the public API are completely decorrelated from the infrastructure. It's a monothreaded logic and thus lock free.

## What are the tradeoffs ?
Nothing comes free, this apply also to [sans-io](https://sans-io.readthedocs.io/), of course.
The downsides of using [sans-io](https://sans-io.readthedocs.io/) is that the project must be separated into at least 2 components, the core layer ([sans-io](https://sans-io.readthedocs.io/)) and the realization layer (infrastructure).

Concretely, it means more boilterplate code is needed to stitch infrastructure layer to core layer.

## Architecture
Troc-core is composed of 4 main modules:
- domain types: map RTPS and DDS domain types
- reader: map RTPS Reader / DDS DataReader
- writer: map RTPS Writer / DDS DataWriter
- discovery: map RTPS discovery

## Test strategy
Since troc-core is fully testable, don't know about network or even time, the following test strategy apply naturally:
- unit testing
- property testing
- mutation testing

# Troc
Troc is the realization layer, the one who bridge troc-core with infrastructure.
It's highly asynchronous, multihreaded, concurrent, uses channel for message passing, timer. It rely heavily on Tokio for those jobs.
This layer is also where the serialization and deserialization of the protocol messages and payload happens.

## Architecture
While the purpose of the realization layer is to provide all the infrastructure needed to truly accomplish the goal of DDS, it's not yet completely clear how this layer will be organized.

There are aspects i'm sure of though:
- **actor pattern**: to speed up developement i use Kameo
- **async runtime**: Tokio

Things i would like to have:
- ability to mock endpoints (sockets, shm): for chaos testing purpose

## Test strategy
Since this layer is mostly about I/O and then dealing with lot of undeterminism and error sources, the testing strategy is as this:
- unit testing
- integration testing
- fuzzing

# Performances considerations
Troc goal is not to overperf the other DDS implementation. That being said, it must be usable and have close performances.

Achieving 50us latency is pretty good and that's the main performance target.

## Benchmarks
To achieve the performance targets, the first step is to measure. For this is i will rely on Criterion benchmark crate.

The main benchmarks will sit in the troc crate.

## Profile
### CPU
Criterion can integrate pprof to, using the same benchmark, profile and produce a flamegraph.
It's not as good as using perf for example, but it give a good overview.

When digging deeper is needed, i use perf to generate performance profiling files and hotspot to visualize them.

### Memory
For memory profiling, i use bytehound.
