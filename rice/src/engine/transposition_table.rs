use crate::score::Score;

use std::mem;

static MEMORY_FACTOR: usize = 1024;
static TABLE_SIZE: usize =  1 * MEMORY_FACTOR * MEMORY_FACTOR * MEMORY_FACTOR; // 1GB in bytes

#[derive(Copy, Clone)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone)]
pub struct TranspositionEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: Score,
    pub bound: Bound,
}

pub struct TranspositionTable(Vec<Option<TranspositionEntry>>);

impl TranspositionTable {
    pub fn new() -> Self {
        let entry_size = mem::size_of::<TranspositionEntry>();
        let num_entries = TABLE_SIZE / entry_size;
        Self(vec![None; num_entries])
    }

    pub fn lookup(&self, hash: u64, depth: u8) -> Option<TranspositionEntry> {
        let index_num = (hash % self.0.len() as u64) as usize;
        if let Some(entry) = self.0[index_num] {
            if entry.depth < depth || hash != entry.hash {
                return None;
            }
            return Some(entry);
        }
        None
    }

    pub fn store(&mut self, hash: u64, score: Score, bound: Bound, depth: u8) {
        let entry = TranspositionEntry {
            hash,
            depth,
            score,
            bound,
        };
        let index = (hash % self.0.len() as u64) as usize;
        self.0[index] = Some(entry);
    }
}
