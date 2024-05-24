// src/segment_tree.rs
use super::types::PriceEntry;
use std::cmp::Ordering;

// Segment tree to find the minimum and maximum in a range
pub struct SegmentTree {
    min_tree: Vec<Option<PriceEntry>>,
    max_tree: Vec<Option<PriceEntry>>,
    n: usize,
}

impl SegmentTree {
    // Initialize segment tree with given data
    pub fn new(data: Vec<PriceEntry>) -> Self {
        let n = data.len();
        let mut min_tree = vec![None; 2 * n];
        let mut max_tree = vec![None; 2 * n];
        for i in 0..n {
            min_tree[n + i] = Some(data[i].clone());
            max_tree[n + i] = Some(data[i].clone());
        }
        for i in (1..n).rev() {
            min_tree[i] = partial_min(min_tree[2 * i].clone(), min_tree[2 * i + 1].clone());
            max_tree[i] = partial_max(max_tree[2 * i].clone(), max_tree[2 * i + 1].clone());
        }
        SegmentTree { min_tree, max_tree, n }
    }

    // Update segment tree at position `pos` with value `value`
    pub fn update(&mut self, mut pos: usize, value: PriceEntry) {
        pos += self.n;
        self.min_tree[pos] = Some(value.clone());
        self.max_tree[pos] = Some(value);
        while pos > 1 {
            pos /= 2;
            self.min_tree[pos] = partial_min(self.min_tree[2 * pos].clone(), self.min_tree[2 * pos + 1].clone());
            self.max_tree[pos] = partial_max(self.max_tree[2 * pos].clone(), self.max_tree[2 * pos + 1].clone());
        }
    }

    // Query minimum value in range [l, r)
    pub fn range_min_query(&self, mut l: usize, mut r: usize) -> Option<PriceEntry> {
        l += self.n;
        r += self.n;
        let mut min_val = None;
        while l < r {
            if l % 2 == 1 {
                min_val = partial_min(min_val, self.min_tree[l].clone());
                l += 1;
            }
            if r % 2 == 1 {
                r -= 1;
                min_val = partial_min(min_val, self.min_tree[r].clone());
            }
            l /= 2;
            r /= 2;
        }
        min_val
    }

    // Query maximum value in range [l, r)
    pub fn range_max_query(&self, mut l: usize, mut r: usize) -> Option<PriceEntry> {
        l += self.n;
        r += self.n;
        let mut max_val = None;
        while l < r {
            if l % 2 == 1 {
                max_val = partial_max(max_val, self.max_tree[l].clone());
                l += 1;
            }
            if r % 2 == 1 {
                r -= 1;
                max_val = partial_max(max_val, self.max_tree[r].clone());
            }
            l /= 2;
            r /= 2;
        }
        max_val
    }
}

// Helper functions for partial min and max
fn partial_min(a: Option<PriceEntry>, b: Option<PriceEntry>) -> Option<PriceEntry> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (Some(x), Some(y)) => match x.price.partial_cmp(&y.price) {
            Some(Ordering::Less) | Some(Ordering::Equal) => Some(x),
            _ => Some(y),
        },
    }
}

fn partial_max(a: Option<PriceEntry>, b: Option<PriceEntry>) -> Option<PriceEntry> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (Some(x), Some(y)) => match x.price.partial_cmp(&y.price) {
            Some(Ordering::Greater) | Some(Ordering::Equal) => Some(x),
            _ => Some(y),
        },
    }
}
