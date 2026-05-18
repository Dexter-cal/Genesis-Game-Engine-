//! Sparse set — efficient storage for components that most entities don't have

pub struct SparseSet<T> {
    dense: Vec<T>,
    dense_ids: Vec<u64>,
    sparse: Vec<u32>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self { dense: Vec::new(), dense_ids: Vec::new(), sparse: Vec::new() }
    }

    pub fn insert(&mut self, id: u64, value: T) {
        let idx = id as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, u32::MAX);
        }
        if self.sparse[idx] == u32::MAX {
            self.sparse[idx] = self.dense.len() as u32;
            self.dense.push(value);
            self.dense_ids.push(id);
        } else {
            self.dense[self.sparse[idx] as usize] = value;
        }
    }

    pub fn get(&self, id: u64) -> Option<&T> {
        let idx = id as usize;
        if idx >= self.sparse.len() || self.sparse[idx] == u32::MAX { return None; }
        Some(&self.dense[self.sparse[idx] as usize])
    }

    pub fn contains(&self, id: u64) -> bool {
        let idx = id as usize;
        idx < self.sparse.len() && self.sparse[idx] != u32::MAX
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.dense_ids.iter().copied().zip(self.dense.iter())
    }

    pub fn len(&self) -> usize { self.dense.len() }
}
