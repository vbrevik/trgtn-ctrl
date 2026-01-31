# MSHNCTRL Targeting

Targeting and BDA (Battle Damage Assessment) module for military operations.

## Crates

| Crate | Description |
|-------|-------------|
| `targeting` | F3EAD cycle, JTB voting, CDE, target nominations |
| `bda` | Battle Damage Assessment reports, annotations, peer review |
| `roe` | Rules of Engagement management |
| `targeting-server` | Unified server composing targeting features |

## Features

### Targeting Cell
- **F3EAD Cycle**: Find, Fix, Finish, Exploit, Analyze, Disseminate
- **Joint Targeting Board (JTB)**: Target nomination and voting
- **Collateral Damage Estimation (CDE)**: Risk assessment
- **Strike Platforms**: Weapon-target pairing

### BDA Workbench
- **Phase 0**: Physical damage assessment
- **Phase 1**: Functional damage assessment  
- **Phase 4**: Target system assessment
- **Annotations**: Image markup and notes
- **Peer Review**: Collaborative assessment validation

### ROE
- **Rules Management**: Create and manage ROE sets
- **Compliance Checking**: Validate operations against rules

## Quick Start

```bash
cargo build --workspace
cargo run -p targeting-server
```

## Dependencies

Requires `mshnctrl-core` for authentication and authorization.
