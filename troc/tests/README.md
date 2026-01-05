# DDS/RTPS Implementation Test Plan

This document outlines a comprehensive test suite for validating a DDS/RTPS implementation, focusing on discovery and QOS features.

## 0. **USAGE**

```bash
cargo test --package troc_core --test mod -- discovery::basic --show-output
```

## 1. Discovery

### 1.1 Basic
Tests focusing on basic discovery mechanisms with default QOS between two participants.

#### `test_two_participants_discovery()`
- Tests basic discovery between two participants
- Verifies basic matching with default QOS
- Validates simple communication path

#### `test_participant_join_leave()`
- Tests participant leaving the system
- Validates cleanup of discovery data
- Verifies system stability after participant removal

### 1.2 Complex

#### `test_many_participants_discovery()`
- Tests discovery with multiple endpoints (3+)
- Verifies complete discovery mesh through communication
- Validates data consistency across multiple participants

#### `test_domain_isolation()`
- Tests isolation between different DDS domains
- Validates no cross-domain discovery
- Verifies domain boundary enforcement

### 1.3 Matching
Tests focusing on QOS matching between two participants.

#### `test_reliable_delivery()`
- Tests reliable communication matching
- Validates ordered delivery
- Verifies retransmission behavior

#### `test_reliable_resource_limits()`
- Tests resource limits with reliable QOS
- Validates blocking behavior
- Verifies queue management

#### `test_best_effort_behavior()`
- Tests best effort communication matching
- Validates loss handling
- Verifies performance characteristics

#### `test_reliable_with_keep_last()`
- Tests reliability with limited history
- Validates queue management
- Verifies behavior under stress

#### `test_reliable_with_keep_all()`
- Tests reliability with unlimited history
- Validates resource management
- Verifies slow reader scenarios

#### `test_transient_local_reliable()`
- Tests durability with reliability
- Validates historical data delivery
- Verifies resource usage

#### `test_deadline_with_reliability()`
- Tests deadline with reliable delivery
- Validates timing accuracy
- Verifies resource conflicts

## 2. Communication
### 2.1 Default test suit

#### One-to-One Communication

#### `test_single_reader_single_writer()`

- Tests 1:1 communication pattern
- Validates simple scenario communication
- Verifies data consistency 

#### One-to-Many Communication

#### `test_many_readers_single_writer()`
- Tests 1:3 communication pattern (1 writer to 3 readers)
- Validates fan-out communication
- Verifies data consistency across readers

#### `test_many_readers_message_ordering()`
- Tests message ordering with multiple readers
- Validates sequence delivery
- Verifies ordering preservation

#### Many-to-One Communication

#### `test_many_writers_single_reader()`
- Tests 3:1 communication pattern
- Validates multiple writer discovery
- Verifies message identification

#### Many-to-Many Communication

#### `test_mesh_topology()`
- Tests 3:3 communication pattern
- Validates full mesh communication
- Verifies cross-participant messaging

#### `test_several_endpoints_per_participants()`
- Tests multiple endpoints in same participant
- Validates endpoint discovery
- Verifies endpoint isolation

### 2.2 QOS Scenarios

Each test case is executed under the following **6 valid QOS configurations** defined [Go to 2.1 Default Test Suite](#21-default-test-suit)
:

| #    | Reliability | History      | Durability      |
| ---- | ----------- | :----------- | --------------- |
| 0    | BEST_EFFORT | KEEP_LAST(1) | VOLATILE        |
| 1    | BEST_EFFORT | KEEP_ALL     | VOLATILE        |
| 2    | RELIABLE    | KEEP_LAST(1) | VOLATILE        |
| 3    | RELIABLE    | KEEP_LAST(1) | TRANSIENT_LOCAL |
| 4    | RELIABLE    | KEEP_ALL     | VOLATILE        |
| 5    | RELIABLE    | KEEP_ALL     | TRANSIENT_LOCAL |

### 2.3 Communication with QOS

#### `test_late_joiner()`
- Tests late-joining participant behavior
- Validates historical data handling
- Verifies discovery consistency

#### `test_endpoint_deadline_finite()`

- Tests deadline mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_endpoint_deadline_infinite()`

- Tests deadline mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_endpoint_liveliness_manual()`
- Tests liveliness mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_endpoint_liveliness_by_topic()`

- Tests liveliness mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_endpoint_lifespan_finite()`

- Tests lifespan mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_endpoint_lifespan_infinite()`

- Tests lifespan mechanism
- Validates endpoint state tracking
- Verifies liveliness changes

#### `test_topic_lifecycle()`
- Tests topic creation and deletion
- Validates topic discovery
- Verifies cleanup

#### `test_incompatible_combinations()`
- Tests invalid QOS combinations
- Validates error handling
- Verifies configuration validation

## 3. Basic QOS

#### `test_deadline_enforcement()`
- Tests basic deadline behavior
- Validates deadline missed callbacks
- Verifies timing accuracy

#### `test_automatic_liveliness()`
- Tests automatic liveliness assertion
- Validates lease duration handling
- Verifies liveliness changes

#### `test_manual_liveliness()`
- Tests manual liveliness assertion
- Validates assertion period
- Verifies manual control

#### `test_volatile_durability()`
- Tests volatile durability behavior
- Validates no-persistence
- Verifies late-joiner behavior

#### `test_transient_local_durability()`
- Tests transient local durability
- Validates historical data
- Verifies persistence

#### `test_keep_last_behavior()`
- Tests limited history
- Validates sample replacement
- Verifies resource usage

#### `test_keep_all_behavior()`
- Tests unlimited history
- Validates resource limits
- Verifies memory management

## 4. Stress / Charge

#### `test_bulk_participant_join()`
- Tests concurrent participant joining
- Validates discovery under load
- Verifies system stability

#### `test_bulk_endpoint_creation()`
- Tests concurrent endpoint creation
- Validates endpoint discovery under load
- Verifies no missed discoveries

#### `test_participant_removal_cascade()`
- Tests cascade of participant removals
- Validates cleanup propagation
- Verifies system recovery

#### `test_cascading_discovery()`
- Tests sequential discovery
- Validates discovery propagation
- Verifies timing consistency

#### `test_multiple_topics_discovery()`
- Tests multiple topic handling
- Validates topic isolation
- Verifies cross-topic behavior

#### `test_qos_combinations_under_load()`
- Tests QOS combinations under stress
- Validates system stability
- Verifies resource management