//! Explicit labeled transition systems and their goal distances.

use alloc::collections::BinaryHeap;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

/// A transition `src --label--> dst` of a [`LabeledTransitionSystem`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transition {
    /// The source state, in `0..num_states`.
    pub src: usize,
    /// The label, in `0..num_labels`; for abstractions of a task, the
    /// operator id.
    pub label: usize,
    /// The destination state, in `0..num_states`.
    pub dst: usize,
}

/// An explicit labeled transition system. Self-loops are kept: they matter
/// for synchronized products (merge-and-shrink), and are ignored where they
/// are irrelevant (distances, saturated costs).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabeledTransitionSystem {
    /// The number of states; states are `0..num_states`.
    pub num_states: usize,
    /// The number of labels; labels are `0..num_labels`.
    pub num_labels: usize,
    /// The transitions, in no particular order unless [`normalize`d](Self::normalize).
    pub transitions: Vec<Transition>,
    /// Whether each state is a goal state; `num_states` entries.
    pub goal: Vec<bool>,
    /// The initial state, if there is one.
    pub initial: Option<usize>,
}

impl LabeledTransitionSystem {
    /// Cheapest cost to reach a goal state from each state, under `costs`
    /// (indexed by label). `None` = no goal reachable. Sums saturate at
    /// `u64::MAX`.
    ///
    /// # Panics
    ///
    /// If `costs` does not have one entry per label, or a transition or goal
    /// flag refers to a state outside `0..num_states`.
    #[must_use]
    pub fn goal_distances(&self, costs: &[u64]) -> Vec<Option<u64>> {
        dijkstra(
            &self.adjacency(costs, true),
            self.goal.iter().enumerate().filter(|g| *g.1).map(|g| g.0),
        )
    }

    /// Cheapest cost from the initial state to each state; all `None`
    /// without an initial state.
    ///
    /// # Panics
    ///
    /// If `costs` does not have one entry per label, or a transition or the
    /// initial state refers to a state outside `0..num_states`.
    #[must_use]
    pub fn init_distances(&self, costs: &[u64]) -> Vec<Option<u64>> {
        dijkstra(&self.adjacency(costs, false), self.initial)
    }

    /// Weighted edges per state, without self-loops; `reverse` follows
    /// transitions backwards.
    fn adjacency(&self, costs: &[u64], reverse: bool) -> Vec<Vec<(usize, u64)>> {
        assert_eq!(costs.len(), self.num_labels, "one cost per label");
        let mut adj = vec![Vec::new(); self.num_states];
        for t in self.transitions.iter().filter(|t| t.src != t.dst) {
            let (from, to) = if reverse {
                (t.dst, t.src)
            } else {
                (t.src, t.dst)
            };
            adj[from].push((to, costs[t.label]));
        }

        adj
    }

    /// Sorts transitions and removes duplicates.
    pub fn normalize(&mut self) {
        self.transitions.sort_unstable();
        self.transitions.dedup();
    }
}

/// Multi-source Dijkstra: the cheapest distance from the nearest of
/// `sources` to each node of the weighted adjacency list `adj`.
fn dijkstra(
    adj: &[Vec<(usize, u64)>],
    sources: impl IntoIterator<Item = usize>,
) -> Vec<Option<u64>> {
    let mut dist = vec![None; adj.len()];
    let mut heap = BinaryHeap::new();
    for s in sources {
        dist[s] = Some(0);
        heap.push(Reverse((0u64, s)));
    }

    while let Some(Reverse((d, s))) = heap.pop() {
        if dist[s].is_some_and(|best| d > best) {
            continue;
        }
        for &(t, c) in &adj[s] {
            // Saturate, so an "infinite" cost of u64::MAX cannot wrap to a short path.
            let candidate = d.saturating_add(c);
            if dist[t].is_none_or(|old| candidate < old) {
                dist[t] = Some(candidate);
                heap.push(Reverse((candidate, t)));
            }
        }
    }

    dist
}
