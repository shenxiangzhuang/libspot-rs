//! Circular buffer implementation (Ubend)
//! 
//! This module implements a circular buffer that matches the C implementation exactly.
//! The Ubend structure is a kind of circular vector that starts empty, fills up to capacity,
//! and then overwrites older data with newer data.

use crate::error::{SpotError, SpotResult};

/// Circular buffer implementation that matches the C Ubend structure
#[derive(Debug, Clone)]
pub struct Ubend {
    /// Current position inside the container
    cursor: usize,
    /// Maximum storage capacity
    capacity: usize,
    /// Last erased value (i.e., replaced by a new one)
    last_erased_data: f64,
    /// Container fill status (true = filled, false = not filled)
    filled: bool,
    /// Data container
    data: Vec<f64>,
}

impl Ubend {
    /// Initialize a new Ubend with the given capacity
    pub fn new(capacity: usize) -> SpotResult<Self> {
        if capacity == 0 {
            return Err(SpotError::MemoryAllocationFailed);
        }
        
        Ok(Self {
            cursor: 0,
            filled: false,
            capacity,
            last_erased_data: f64::NAN,
            data: vec![0.0; capacity],
        })
    }

    /// Get the current size of the container
    /// Returns capacity if filled, otherwise returns cursor position
    pub fn size(&self) -> usize {
        if self.filled {
            self.capacity
        } else {
            self.cursor
        }
    }

    /// Push a new value into the container
    /// Returns the value that was erased (if any), otherwise NaN
    pub fn push(&mut self, x: f64) -> f64 {
        // If the container has already been filled, we must keep in memory
        // the data we will erase
        if self.filled {
            self.last_erased_data = self.data[self.cursor];
        }

        // Assign value at cursor
        self.data[self.cursor] = x;

        // Increment cursor
        if self.cursor == self.capacity - 1 {
            self.cursor = 0;
            self.filled = true;
        } else {
            self.cursor += 1;
        }

        self.last_erased_data
    }

    /// Get iterator over the data in insertion order
    pub fn iter(&self) -> UbendIterator<'_> {
        UbendIterator {
            ubend: self,
            index: 0,
        }
    }

    /// Get the data at a specific index in insertion order
    pub fn get(&self, index: usize) -> Option<f64> {
        let size = self.size();
        if index >= size {
            return None;
        }
        
        if !self.filled {
            // Simple case: data is contiguous from 0 to cursor-1
            Some(self.data[index])
        } else {
            // Complex case: data wraps around
            let real_index = (self.cursor + index) % self.capacity;
            Some(self.data[real_index])
        }
    }

    /// Access to raw data (for compatibility with C implementation)
    pub fn raw_data(&self) -> &[f64] {
        &self.data
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if the buffer is filled
    pub fn is_filled(&self) -> bool {
        self.filled
    }

    /// Get current cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get last erased data
    pub fn last_erased_data(&self) -> f64 {
        self.last_erased_data
    }

    /// Get all data in insertion order as a vector
    pub fn data(&self) -> Vec<f64> {
        self.iter().collect()
    }
}

/// Iterator over Ubend data in insertion order
pub struct UbendIterator<'a> {
    ubend: &'a Ubend,
    index: usize,
}

impl<'a> Iterator for UbendIterator<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.ubend.get(self.index);
        self.index += 1;
        result
    }
}

impl<'a> ExactSizeIterator for UbendIterator<'a> {
    fn len(&self) -> usize {
        self.ubend.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::math::is_nan;

    #[test]
    fn test_ubend_creation() {
        let ubend = Ubend::new(5).unwrap();
        assert_eq!(ubend.capacity(), 5);
        assert_eq!(ubend.size(), 0);
        assert!(!ubend.is_filled());
        assert_eq!(ubend.cursor(), 0);
        assert!(is_nan(ubend.last_erased_data()));
    }

    #[test]
    fn test_ubend_zero_capacity() {
        let result = Ubend::new(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::MemoryAllocationFailed);
    }

    #[test]
    fn test_ubend_push_before_full() {
        let mut ubend = Ubend::new(3).unwrap();
        
        // Push first element
        let erased = ubend.push(1.0);
        assert!(is_nan(erased));
        assert_eq!(ubend.size(), 1);
        assert!(!ubend.is_filled());
        assert_eq!(ubend.cursor(), 1);

        // Push second element
        let erased = ubend.push(2.0);
        assert!(is_nan(erased));
        assert_eq!(ubend.size(), 2);
        assert!(!ubend.is_filled());
        assert_eq!(ubend.cursor(), 2);

        // Push third element
        let erased = ubend.push(3.0);
        assert!(is_nan(erased));
        assert_eq!(ubend.size(), 3);
        assert!(ubend.is_filled());
        assert_eq!(ubend.cursor(), 0);
    }

    #[test]
    fn test_ubend_push_after_full() {
        let mut ubend = Ubend::new(3).unwrap();
        
        // Fill the buffer
        ubend.push(1.0);
        ubend.push(2.0);
        ubend.push(3.0);
        
        // Now it should start overwriting
        let erased = ubend.push(4.0);
        assert_relative_eq!(erased, 1.0);
        assert_eq!(ubend.size(), 3);
        assert!(ubend.is_filled());
        assert_eq!(ubend.cursor(), 1);

        let erased = ubend.push(5.0);
        assert_relative_eq!(erased, 2.0);
        assert_eq!(ubend.size(), 3);
        assert!(ubend.is_filled());
        assert_eq!(ubend.cursor(), 2);
    }

    #[test]
    fn test_ubend_get() {
        let mut ubend = Ubend::new(3).unwrap();
        
        // Test empty buffer
        assert!(ubend.get(0).is_none());
        
        // Add some data
        ubend.push(10.0);
        ubend.push(20.0);
        
        assert_relative_eq!(ubend.get(0).unwrap(), 10.0);
        assert_relative_eq!(ubend.get(1).unwrap(), 20.0);
        assert!(ubend.get(2).is_none());
        
        // Fill buffer and test wraparound
        ubend.push(30.0);
        ubend.push(40.0); // This should overwrite 10.0
        
        assert_relative_eq!(ubend.get(0).unwrap(), 20.0);
        assert_relative_eq!(ubend.get(1).unwrap(), 30.0);
        assert_relative_eq!(ubend.get(2).unwrap(), 40.0);
    }

    #[test]
    fn test_ubend_iterator() {
        let mut ubend = Ubend::new(3).unwrap();
        
        ubend.push(1.0);
        ubend.push(2.0);
        ubend.push(3.0);
        
        let values: Vec<f64> = ubend.iter().collect();
        assert_eq!(values, vec![1.0, 2.0, 3.0]);
        
        // Test after wraparound
        ubend.push(4.0);
        let values: Vec<f64> = ubend.iter().collect();
        assert_eq!(values, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_ubend_exact_size_iterator() {
        let mut ubend = Ubend::new(3).unwrap();
        
        assert_eq!(ubend.iter().len(), 0);
        
        ubend.push(1.0);
        assert_eq!(ubend.iter().len(), 1);
        
        ubend.push(2.0);
        ubend.push(3.0);
        assert_eq!(ubend.iter().len(), 3);
        
        ubend.push(4.0);
        assert_eq!(ubend.iter().len(), 3);
    }
}