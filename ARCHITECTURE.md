# Overview
Troc is structured as a layered architecture separating protocol logic from I/O and Time.

The project is composed of:
- **troc-core**: [Sans-io](https://sans-io.readthedocs.io/) DDS protocol logic
- **troc-cdr**: currently empty, placeholder for future implementation
- **troc-messaging**: currently empty, placeholder for future implementation
- **troc-derive**: Derive macro handling Keyed structures
- **troc**: The Realization layer bridging core logic with I/O, Time, multithreading and concurrency, using Tokio

# Troc-core
This is the layer that focuses on the DDS protocol logic, using sans-io.

## What is sans-io ?
If you don't know about sans-io, the term might seem weird at first, after all, how a network protocol could do its job without network in the first place ?
The purpose of sans-io architectural pattern is to separate the protocol logic from the infrastructure logic. By infrastructure, i mean socket, multithreading, concurrency, time, message passing etc...
This lead to a protocol component that is highly testable and reliable.

> **Time is abstracted**: troc-core uses i64 as a replacement for a Timestamp
> Every time related input can then be fed with deterministic values

## Why using it for Troc ?
Troc is a DDS implementation and thus has a stateful and complex protocol logic. Using sans-io for troc-core leverages all the benefits of this pattern.

Troc-core can be fully tested easily without touching network or even time. All inputs of the public API are completely decorrelated from the infrastructure. It's a single-threaded logic and thus lock free.

## What are the tradeoffs ?
Nothing comes free, this apply also to sans-io, of course.
The downsides of using sans-io is that the project must be separated into at least 2 components, the core layer (sans-io) and the realization layer (infrastructure).

Concretely, it means more boilerplate code is needed to stitch infrastructure layer to core layer.

## Architecture
Troc-core is composed of 4 main modules:
- domain types: map RTPS and DDS domain types
- reader: map RTPS Reader / DDS DataReader
- writer: map RTPS Writer / DDS DataWriter
- discovery: map RTPS discovery

## Test strategy
Since troc-core is fully testable, doesn't know about network or even time, the following test strategy apply naturally:
- unit testing
- property testing
- mutation testing

# Troc
Troc is the realization layer, the one that bridges troc-core with infrastructure.
It's highly asynchronous, multithreaded, concurrent, uses channel for message passing, timer. It relies heavily on Tokio for those jobs.
This layer is also where the serialization and deserialization of the protocol messages and payload happens.

## Architecture
While the purpose of the realization layer is to provide all the infrastructure needed to truly accomplish the goal of DDS, it's not yet completely clear how this layer will be organized.

There are aspects i'm sure of though:
- **actor pattern**: to speed up development i use Kameo
- **async runtime**: Tokio

Things i would like to have:
- ability to mock endpoints (sockets, shm): for chaos testing purpose

## Test strategy
Since this layer is mostly about I/O and then dealing with lots of nondeterminism and error sources, the testing strategy is as this:
- unit testing
- integration testing
- fuzzing
- chaos testing

### Fuzzing
The realization layer is the primary attack surface for malformed network input. Fuzzing validates robustness against unexpected data.

#### Deserialization Fuzzing
**Goal:** Ensure the deserializer never panics, crashes, or exhibits memory unsafety on arbitrary input.

**Approach:** Using cargo-fuzz to generate randomized byte sequences.

**What this catches:**
- Buffer overflows from malicious length fields
- Integer overflows in size calculations  
- Panics from invalid UTF-8 in string fields
- Out-of-bounds access from malformed submessage headers
- Excessive allocations or infinite loops

**Coverage target:** All deserialization code paths exercised with randomized inputs.

#### Structured Message Fuzzing
Validating the core's handling of *valid* but arbitrary messages.

**What this catches:**
- Unexpected state transitions
- Resource leaks in core logic
- Incorrect idempotence implementation

### Chaos Testing

**Goal:** Validate graceful degradation under realistic failure conditions.

While fuzzing tests robustness to *malformed data*, chaos testing validates behavior under *infrastructure failures*.

#### Network Chaos

Inject realistic network problems:

**Chaos scenarios:**
- **Packet loss:** Random drops (0-50% loss rate)
- **Delays:** Variable latency (0-500ms)
- **Reordering:** Buffer and shuffle packets
- **Duplication:** Send packets multiple times
- **Connection reset:** Simulate socket failures

**Validation:**
- System recovers without manual intervention
- No deadlocks or hangs
- Retransmissions work correctly
- Discovery re-establishes after network partitions

#### Actor Chaos

Inject actor failures to test supervision and recovery:

**Chaos scenarios:**
- Random actor crashes
- Actor restart with state loss
- Message queue overflows
- Slow message processing

**Validation:**
- Supervisor restarts failed actors
- System continues operating with degraded performance
- No resource leaks (sockets, memory, file descriptors)
- State is recovered or gracefully rebuilt

# Performances considerations
Troc goal is not to outperform the other DDS implementation. That being said, it must be usable and have close performances.

Achieving 50us latency is pretty good and that's the main performance target.

## Benchmarks
To achieve the performance targets, the first step is to measure. For this i will rely on Criterion benchmark crate.

The main benchmarks will sit in the troc crate.

## Profile
### CPU
Criterion can integrate pprof to, using the same benchmark, profile and produce a flamegraph.
It's not as good as using perf for example, but it gives a good overview.

When digging deeper is needed, i use perf to generate performance profiling files and hotspot to visualize them.

### Memory
For memory profiling, i use bytehound.
