use crate::*;

/// A mapping between positions in a rect and some
/// values, with a default value on positions you
/// didn't explicitly set
pub struct PosMap<T: Copy> {
    dim: Dim,
    values: Box<[T]>,
    default_value: T,
}

impl<T: Copy> PosMap<T> {
    pub fn new(dim: Dim, default_value: T) -> Self {
        let values = vec![default_value; dim.w * dim.h].into_boxed_slice();
        Self {
            dim,
            values,
            default_value,
        }
    }
    pub fn get(&self, p: Pos) -> T {
        self.values[self.dim.idx(p)]
    }
    pub fn set(&mut self, p: Pos, value: T) {
        self.values[self.dim.idx(p)] = value;
    }
    pub fn clear(&mut self) {
        self.values.fill(self.default_value);
    }
    pub fn remove(&mut self, p: Pos) -> T {
        let idx = self.dim.idx(p);
        let old = self.values[idx];
        self.values[idx] = self.default_value;
        old
    }
}

impl<T: Copy + PartialEq> PosMap<T> {
    /// Warning: this function is slow
    pub fn is_empty(&self) -> bool {
        !self.is_not_empty()
    }
    /// tells whether there are not default values
    ///
    /// Warning: this function is slow
    /// (it could be optimized with a counter but
    /// there's no need today in mazter)
    pub fn is_not_empty(&self) -> bool {
        self.values.iter().any(|&v| v != self.default_value)
    }
}
pub type PosSet = PosMap<bool>;
