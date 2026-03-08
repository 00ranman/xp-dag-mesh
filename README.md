# XP DAG Mesh

> **Ecosystem Note:** This is the standalone Rust implementation of the DAG Layer 1 protocol. It provides the consensus and transaction layer for the [extropy-engine](https://github.com/00ranman/extropy-engine) monorepo ecosystem. XP minting uses the same `XP = R × F × ΔS × (w · E) × log(1/Tₛ)` formula as `packages/xp-mint`. Validator mesh coordinates with [SignalFlow](https://github.com/00ranman/signalflow) for real-time orchestration. See [ECOSYSTEM_MAP.md](https://github.com/00ranman/extropy-engine/blob/main/ECOSYSTEM_MAP.md) for the full repository mapping.

Physics-anchored DAG Layer 1 protocol with entropy-weighted consensus for the Extropy Engine ecosystem.

## Overview

XP DAG Mesh implements a revolutionary blockchain protocol that replaces traditional proof-of-work/stake with physics-based validation. Transactions are ordered through a Directed Acyclic Graph (DAG) structure where entropy reduction becomes the basis for consensus and value creation.

## Key Features

- **DAG Architecture**: No blocks, parallel transaction processing
- **Physics-Based Consensus**: Entropy reduction drives validation
- **Thermodynamic Compliance**: Real physics laws prevent gaming
- **Validator Mesh**: Domain-specialized entropy-weighted consensus
- **Causal Loop Detection**: Cross-platform synergy rewards
- **Temporal Integration**: XP timekeeping for precise ordering

## Core Components

### DAG Core (`dag_core/`)
- Transaction types: Transfer, EntropyReductionClaim, PhysicsXpMint, TemporalLoop
- DAG structure with parallel processing
- Transaction validation with entropy constraints
- Topological ordering and traversal algorithms

### Validator Mesh (`validator_mesh/`)
- Entropy-weighted consensus mechanism
- Domain specialization (Cognitive, Creative, Physical, etc.)
- Reputation-based validator selection
- Thermodynamic compliance verification

### P2P Sync (`p2p_sync/`)
- Gossip protocol for transaction propagation
- DAG synchronization across nodes
- Network topology optimization
- Bandwidth-efficient sync algorithms

### Common (`common/`)
- Shared types and utilities
- Cryptographic primitives
- Entropy measurement systems
- Physics calculation engine

## Transaction Types

```rust
// Entropy reduction claim with physics validation
TransactionType::EntropyReductionClaim {
    claimant: Address,
    reduction: EntropyReduction,
}

// Physics-based XP minting
TransactionType::PhysicsXpMint {
    recipient: Address,
    calculation_input: XpCalculationInput,
}

// Temporal loop closure
TransactionType::TemporalLoop {
    initiator: Address,
    temporal_loop: TemporalLoop,
}
```

## Physics Formula

XP calculation uses the complete physics formula:
```
XP = R × F × ΔS × (w · E) × log(1/Tₛ)
```

Where:
- R = Reputation factor
- F = Feedback closure strength  
- ΔS = Entropy reduction (J/K)
- (w · E) = Domain-weighted essentiality
- Tₛ = Temporal sustainability

## Integration

Works with:
- **XP Timekeeping**: Base-10 temporal architecture
- **LevelUp Academy**: Skill-based entropy reduction
- **SignalFlow**: Validation mesh coordination
- **HomeFlow**: Daily optimization loops
- **Merchant Network**: Value exchange protocols

## Usage

```rust
use xp_dag_mesh::{Dag, Transaction, TransactionType, PhysicsXpCalculator};

// Create DAG
let mut dag = Dag::new();

// Add entropy reduction transaction
let entropy_tx = Transaction::new(
    TransactionType::EntropyReductionClaim {
        claimant: my_address,
        reduction: entropy_measurement,
    },
    parents,
    nonce,
    metadata,
);

dag.add_transaction(entropy_tx)?;

// Calculate physics-based XP
let calculator = PhysicsXpCalculator::default();
let xp_result = calculator.calculate_xp(&calculation_input)?;
```

## License

MIT License - See LICENSE file for details
