# ternary-rigging: Interactive value manipulation and ripple propagation for {-1, 0, +1} systems

## Why This Exists

When you grab a value in a "living spreadsheet" and shake it, the change shouldn't just sit there — it should ripple through connected values, amplified by some connections, dampened by others, reversed by some. Casey needed an interaction layer where ternary values propagate changes through a graph of connections, like pulling a rope through a system of pulleys. No existing ternary crate modeled this kind of interactive, traceable propagation.

## Core Concepts

**Balanced ternary**: A number system with three values: -1 (Neg), 0 (Zero), +1 (Pos). More expressive than binary for three-way decisions.

**Rig**: A single adjustable ternary value with an ID and label. The thing you "grab."

**Rope**: A connection between two Rigs that transmits changes. Has a weight (-1, 0, or +1) that multiplies the transmitted value.

**Pulley**: A direction-change point on a Rope. When `invert` is true, it reverses Pos ↔ Neg during propagation.

**BlockAndTackle**: An amplifier/reducer on a Rope. Multiplies the transmitted value by a factor, then clamps to ternary range.

**RiggingShake**: Applies an oscillation pattern (sequence of Trits) to a Rig, tracing ripples at each step.

**RippleTrace**: A recorded step showing which Rig changed, what value was transmitted, and at which propagation step.

## Quick Start

```toml
[dependencies]
ternary-rigging = "0.1"
```

```rust
use ternary_rigging::*;

let mut rigging = Rigging::new();
rigging.add_rig(Rig::new(0, Trit::Zero, "source"));
rigging.add_rig(Rig::new(1, Trit::Zero, "target"));
let rope_idx = rigging.add_rope(Rope::new(0, 1, 1));
rigging.add_pulley(Pulley::new(rope_idx, true)); // invert

let traces = rigging.set_and_propagate(0, Trit::Pos);
// Rig 1 now holds Neg (Pos inverted by pulley)
assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Neg);
```

## API Overview

| Type | Description |
|------|-------------|
| `Trit` | A balanced ternary value: Neg, Zero, or Pos |
| `Rig` | One adjustable ternary value with ID and label |
| `Rope` | Weighted connection transmitting changes between Rigs |
| `Pulley` | Direction-changer (inverter) applied to a Rope |
| `BlockAndTackle` | Amplifier/reducer applied to a Rope |
| `Rigging` | The graph of connected Rigs with propagation logic |
| `RiggingShake` | An oscillation pattern applied to a Rig |
| `RippleTrace` | A recorded step in a propagation chain |

## How It Works

When you call `set_and_propagate`, the Rigging performs a depth-first traversal of the graph starting from the changed Rig. At each connected Rope, the incoming Trit is multiplied by the Rope's weight, then passed through any Pulleys (direction change) and BlockAndTackles (amplitude change) attached to that Rope. The resulting Trit is set on the target Rig, and propagation continues recursively.

Cycle detection uses a visited-bit array (256 slots). Once a Rig has been visited in the current propagation, it won't be visited again, preventing infinite loops in cyclic graphs.

The `shake` method calls `set_and_propagate` for each Trit in the oscillation pattern, returning the full trace history for all steps.

## Known Limitations

- Cycle detection uses a fixed 256-slot visited array. Rig IDs ≥ 256 are never marked as visited, meaning cyclic graphs with high IDs could cause stack overflow from infinite recursion.
- Propagation is depth-first, which means the order of rope connections matters for the final state of shared targets.
- No weighted blending: if two Ropes lead to the same target, the last one to propagate wins (no accumulation).
- All state is in-memory only. No persistence or serialization.

## Use Cases

- **Living spreadsheet**: Grab a ternary cell, shake it, and watch changes ripple through dependent cells.
- **Signal propagation simulation**: Model how a ternary signal travels through a network of amplifiers and inverters.
- **Game state propagation**: When one game element changes, automatically propagate the effect through connected elements.
- **Constraint satisfaction**: Set values and trace how constraints propagate through a network.

## Ecosystem Context

Part of the SuperInstance ternary ecosystem. Could feed into `ternary-spreadsheet` as its interactive layer, or use `ternary-graph` for the underlying connection topology. Relates to `ternary-dynamics` for more sophisticated propagation models.

## License

MIT

## See Also
- **ternary-shipyard** — related fleet coordination
- **ternary-dockyard** — related fleet coordination
- **ternary-harbor** — related fleet coordination
- **ternary-sail** — related fleet coordination
- **ternary-cargo** — related fleet coordination
- **ternary-room** — related fleet coordination

