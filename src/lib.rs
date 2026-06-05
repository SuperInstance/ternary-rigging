#![forbid(unsafe_code)]

//! Interactive value manipulation and ripple propagation for balanced ternary systems.
//!
//! Provides a "grab and shake" interaction layer: connected ternary values (Rigs)
//! linked by Ropes inside a Rigging. When you shake one value, ripples propagate
//! through connections, amplified or reduced by BlockAndTackle, direction-changed
//! by Pulleys. Every propagation is recorded as a RippleTrace.

use std::collections::HashMap;

/// A single balanced ternary value: -1, 0, or +1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    pub fn value(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
}

/// One adjustable ternary value in the rigging.
#[derive(Clone, Debug, PartialEq)]
pub struct Rig {
    pub id: usize,
    pub value: Trit,
    pub label: String,
}

impl Rig {
    pub fn new(id: usize, value: Trit, label: &str) -> Self {
        Rig { id, value, label: label.to_string() }
    }

    pub fn set(&mut self, value: Trit) {
        self.value = value;
    }
}

/// A connection between two rigs that transmits ripples.
#[derive(Clone, Debug)]
pub struct Rope {
    pub from: usize,
    pub to: usize,
    pub weight: i8, // multiplier: -1, 0, or +1
}

impl Rope {
    pub fn new(from: usize, to: usize, weight: i8) -> Self {
        Rope { from, to, weight }
    }

    pub fn transmit(&self, incoming: Trit) -> Trit {
        let product = incoming.value() * self.weight;
        Trit::from_i8(product.clamp(-1, 1)).unwrap_or(Trit::Zero)
    }
}

/// A direction-change point in propagation (reverses or nullifies).
#[derive(Clone, Debug)]
pub struct Pulley {
    pub rope_index: usize,
    pub invert: bool,
}

impl Pulley {
    pub fn new(rope_index: usize, invert: bool) -> Self {
        Pulley { rope_index, invert }
    }

    pub fn apply(&self, trit: Trit) -> Trit {
        if self.invert {
            match trit {
                Trit::Neg => Trit::Pos,
                Trit::Pos => Trit::Neg,
                Trit::Zero => Trit::Zero,
            }
        } else {
            trit
        }
    }
}

/// Amplify or reduce propagation force between rigs.
#[derive(Clone, Debug)]
pub struct BlockAndTackle {
    pub rope_index: usize,
    /// Amplification factor: -3..=3. Applied as value * factor, clamped to ternary.
    pub factor: i8,
}

impl BlockAndTackle {
    pub fn new(rope_index: usize, factor: i8) -> Self {
        BlockAndTackle { rope_index, factor }
    }

    pub fn apply(&self, trit: Trit) -> Trit {
        let v = trit.value() as i8 * self.factor;
        match v.clamp(-1, 1) {
            -1 => Trit::Neg,
            0 => Trit::Zero,
            1 => Trit::Pos,
            _ => Trit::Zero,
        }
    }
}

/// A recorded step in ripple propagation.
#[derive(Clone, Debug, PartialEq)]
pub struct RippleTrace {
    pub from_id: usize,
    pub to_id: usize,
    pub value_transmitted: Trit,
    pub step: usize,
}

/// An oscillation applied to a rig, producing ripples through the rigging.
#[derive(Clone, Debug)]
pub struct RiggingShake {
    pub origin_id: usize,
    pub pattern: Vec<Trit>,
}

impl RiggingShake {
    pub fn new(origin_id: usize, pattern: Vec<Trit>) -> Self {
        RiggingShake { origin_id, pattern }
    }
}

/// The main rigging structure: a set of connected rigs.
#[derive(Clone, Debug)]
pub struct Rigging {
    rigs: HashMap<usize, Rig>,
    ropes: Vec<Rope>,
    pulleys: Vec<Pulley>,
    tackles: Vec<BlockAndTackle>,
}

impl Rigging {
    pub fn new() -> Self {
        Rigging {
            rigs: HashMap::new(),
            ropes: Vec::new(),
            pulleys: Vec::new(),
            tackles: Vec::new(),
        }
    }

    pub fn add_rig(&mut self, rig: Rig) {
        self.rigs.insert(rig.id, rig);
    }

    pub fn add_rope(&mut self, rope: Rope) -> usize {
        let idx = self.ropes.len();
        self.ropes.push(rope);
        idx
    }

    pub fn add_pulley(&mut self, pulley: Pulley) {
        self.pulleys.push(pulley);
    }

    pub fn add_tackle(&mut self, tackle: BlockAndTackle) {
        self.tackles.push(tackle);
    }

    pub fn get_rig(&self, id: usize) -> Option<&Rig> {
        self.rigs.get(&id)
    }

    pub fn get_rig_mut(&mut self, id: usize) -> Option<&mut Rig> {
        self.rigs.get_mut(&id)
    }

    pub fn rig_count(&self) -> usize {
        self.rigs.len()
    }

    pub fn rope_count(&self) -> usize {
        self.ropes.len()
    }

    /// Set a rig's value and propagate ripples through connected rigs.
    pub fn set_and_propagate(&mut self, rig_id: usize, value: Trit) -> Vec<RippleTrace> {
        if let Some(rig) = self.rigs.get_mut(&rig_id) {
            rig.set(value);
        } else {
            return Vec::new();
        }

        let mut traces = Vec::new();
        let mut visited = vec![false; 256];
        if rig_id < 256 {
            visited[rig_id] = true;
        }
        self.propagate_recursive(rig_id, value, 1, &mut visited, &mut traces);
        traces
    }

    fn propagate_recursive(
        &mut self,
        from_id: usize,
        value: Trit,
        step: usize,
        visited: &mut [bool],
        traces: &mut Vec<RippleTrace>,
    ) {
        // Collect work items to avoid borrow issues
        let work: Vec<(usize, usize, Trit)> = self.ropes
            .iter()
            .enumerate()
            .filter(|(_, rope)| rope.from == from_id)
            .filter(|(_, rope)| rope.to >= 256 || !visited[rope.to])
            .map(|(rope_idx, rope)| (rope_idx, rope.to, rope.transmit(value)))
            .collect();

        for (rope_idx, target, base_value) in work {
            let mut transmitted = base_value;

            for pulley in &self.pulleys {
                if pulley.rope_index == rope_idx {
                    transmitted = pulley.apply(transmitted);
                }
            }

            for tackle in &self.tackles {
                if tackle.rope_index == rope_idx {
                    transmitted = tackle.apply(transmitted);
                }
            }

            traces.push(RippleTrace {
                from_id,
                to_id: target,
                value_transmitted: transmitted,
                step,
            });

            if let Some(rig) = self.rigs.get_mut(&target) {
                rig.set(transmitted);
            }

            if target < 256 {
                visited[target] = true;
            }
            self.propagate_recursive(target, transmitted, step + 1, visited, traces);
        }
    }

    /// Apply a shake (oscillation pattern) to a rig, tracing each ripple.
    pub fn shake(&mut self, shake: &RiggingShake) -> Vec<Vec<RippleTrace>> {
        let mut all_traces = Vec::new();
        for trit in &shake.pattern {
            let traces = self.set_and_propagate(shake.origin_id, *trit);
            all_traces.push(traces);
        }
        all_traces
    }

    /// Collect all rigs that are neighbors (directly connected) to the given rig.
    pub fn neighbors(&self, rig_id: usize) -> Vec<usize> {
        self.ropes
            .iter()
            .filter(|r| r.from == rig_id)
            .map(|r| r.to)
            .collect()
    }
}

impl Default for Rigging {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trit_values() {
        assert_eq!(Trit::Neg.value(), -1);
        assert_eq!(Trit::Zero.value(), 0);
        assert_eq!(Trit::Pos.value(), 1);
    }

    #[test]
    fn trit_from_i8() {
        assert_eq!(Trit::from_i8(-1), Some(Trit::Neg));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::Pos));
        assert_eq!(Trit::from_i8(5), None);
        assert_eq!(Trit::from_i8(-3), None);
    }

    #[test]
    fn rig_new_and_set() {
        let mut rig = Rig::new(1, Trit::Zero, "test");
        assert_eq!(rig.value, Trit::Zero);
        rig.set(Trit::Pos);
        assert_eq!(rig.value, Trit::Pos);
    }

    #[test]
    fn rope_transmit_positive() {
        let rope = Rope::new(0, 1, 1);
        assert_eq!(rope.transmit(Trit::Pos), Trit::Pos);
        assert_eq!(rope.transmit(Trit::Neg), Trit::Neg);
        assert_eq!(rope.transmit(Trit::Zero), Trit::Zero);
    }

    #[test]
    fn rope_transmit_inverted() {
        let rope = Rope::new(0, 1, -1);
        assert_eq!(rope.transmit(Trit::Pos), Trit::Neg);
        assert_eq!(rope.transmit(Trit::Neg), Trit::Pos);
    }

    #[test]
    fn rope_transmit_zero_weight() {
        let rope = Rope::new(0, 1, 0);
        assert_eq!(rope.transmit(Trit::Pos), Trit::Zero);
        assert_eq!(rope.transmit(Trit::Neg), Trit::Zero);
    }

    #[test]
    fn pulley_invert() {
        let pulley = Pulley::new(0, true);
        assert_eq!(pulley.apply(Trit::Pos), Trit::Neg);
        assert_eq!(pulley.apply(Trit::Neg), Trit::Pos);
        assert_eq!(pulley.apply(Trit::Zero), Trit::Zero);
    }

    #[test]
    fn pulley_passthrough() {
        let pulley = Pulley::new(0, false);
        assert_eq!(pulley.apply(Trit::Pos), Trit::Pos);
    }

    #[test]
    fn block_and_tackle_amplify() {
        let tackle = BlockAndTackle::new(0, 2);
        // value 1 * 2 = 2, clamped to 1
        assert_eq!(tackle.apply(Trit::Pos), Trit::Pos);
        // value -1 * 2 = -2, clamped to -1
        assert_eq!(tackle.apply(Trit::Neg), Trit::Neg);
    }

    #[test]
    fn block_and_tackle_reduce() {
        // factor 0 always produces Zero
        let tackle = BlockAndTackle::new(0, 0);
        assert_eq!(tackle.apply(Trit::Pos), Trit::Zero);
        assert_eq!(tackle.apply(Trit::Neg), Trit::Zero);
    }

    #[test]
    fn block_and_tackle_invert() {
        let tackle = BlockAndTackle::new(0, -1);
        assert_eq!(tackle.apply(Trit::Pos), Trit::Neg);
        assert_eq!(tackle.apply(Trit::Neg), Trit::Pos);
    }

    #[test]
    fn rigging_add_and_get() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        assert_eq!(rigging.rig_count(), 2);
        assert_eq!(rigging.get_rig(0).unwrap().value, Trit::Zero);
    }

    #[test]
    fn rigging_propagate_simple() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        rigging.add_rope(Rope::new(0, 1, 1));

        let traces = rigging.set_and_propagate(0, Trit::Pos);
        assert_eq!(rigging.get_rig(0).unwrap().value, Trit::Pos);
        assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Pos);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].from_id, 0);
        assert_eq!(traces[0].to_id, 1);
    }

    #[test]
    fn rigging_propagate_chain() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        rigging.add_rig(Rig::new(2, Trit::Zero, "c"));
        rigging.add_rope(Rope::new(0, 1, 1));
        rigging.add_rope(Rope::new(1, 2, 1));

        let traces = rigging.set_and_propagate(0, Trit::Neg);
        assert_eq!(rigging.get_rig(2).unwrap().value, Trit::Neg);
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn rigging_propagate_with_pulley() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        let rope_idx = rigging.add_rope(Rope::new(0, 1, 1));
        rigging.add_pulley(Pulley::new(rope_idx, true));

        let traces = rigging.set_and_propagate(0, Trit::Pos);
        assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Neg);
    }

    #[test]
    fn rigging_propagate_with_tackle() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        let rope_idx = rigging.add_rope(Rope::new(0, 1, 1));
        rigging.add_tackle(BlockAndTackle::new(rope_idx, -1));

        let traces = rigging.set_and_propagate(0, Trit::Pos);
        assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Neg);
    }

    #[test]
    fn rigging_shake_oscillation() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        rigging.add_rope(Rope::new(0, 1, 1));

        let shake = RiggingShake::new(0, vec![Trit::Pos, Trit::Neg, Trit::Zero]);
        let all_traces = rigging.shake(&shake);
        assert_eq!(all_traces.len(), 3);
        // After last shake with Zero, rig 1 should be Zero
        assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Zero);
    }

    #[test]
    fn rigging_neighbors() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        rigging.add_rig(Rig::new(2, Trit::Zero, "c"));
        rigging.add_rope(Rope::new(0, 1, 1));
        rigging.add_rope(Rope::new(0, 2, 1));

        let neighbors = rigging.neighbors(0);
        assert_eq!(neighbors, vec![1, 2]);
    }

    #[test]
    fn rigging_no_ropes_no_traces() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        let traces = rigging.set_and_propagate(0, Trit::Pos);
        assert!(traces.is_empty());
    }

    #[test]
    fn rigging_nonexistent_rig() {
        let mut rigging = Rigging::new();
        let traces = rigging.set_and_propagate(99, Trit::Pos);
        assert!(traces.is_empty());
    }

    #[test]
    fn ripple_trace_fields() {
        let trace = RippleTrace {
            from_id: 0,
            to_id: 1,
            value_transmitted: Trit::Neg,
            step: 2,
        };
        assert_eq!(trace.from_id, 0);
        assert_eq!(trace.to_id, 1);
        assert_eq!(trace.value_transmitted, Trit::Neg);
        assert_eq!(trace.step, 2);
    }

    #[test]
    fn rigging_default() {
        let rigging = Rigging::default();
        assert_eq!(rigging.rig_count(), 0);
        assert_eq!(rigging.rope_count(), 0);
    }

    #[test]
    fn rigging_propagate_no_cycles() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.add_rig(Rig::new(1, Trit::Zero, "b"));
        rigging.add_rope(Rope::new(0, 1, 1));
        rigging.add_rope(Rope::new(1, 0, 1)); // cycle

        let traces = rigging.set_and_propagate(0, Trit::Pos);
        // Should not infinite loop; rig 1 gets Pos, then stops (0 already visited)
        assert_eq!(rigging.get_rig(1).unwrap().value, Trit::Pos);
        assert_eq!(traces.len(), 1);
    }

    #[test]
    fn rigging_get_mut() {
        let mut rigging = Rigging::new();
        rigging.add_rig(Rig::new(0, Trit::Zero, "a"));
        rigging.get_rig_mut(0).unwrap().set(Trit::Neg);
        assert_eq!(rigging.get_rig(0).unwrap().value, Trit::Neg);
    }
}
