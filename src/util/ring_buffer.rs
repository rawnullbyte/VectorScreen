use std::collections::VecDeque;
use std::fmt;

/// A fixed-capacity ring buffer backed by `VecDeque<T>`.
///
/// When the buffer is full and a new item is pushed, the oldest item
/// is silently discarded. Iteration yields items from oldest to newest.
#[derive(Clone)]
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone + fmt::Debug> RingBuffer<T> {
    /// Create an empty ring buffer with the given capacity.
    ///
    /// # Panics
    /// Panics if `capacity == 0`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be > 0");
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Create a ring buffer pre-filled from a vector.
    ///
    /// Only the last `capacity` items are kept if the vector is longer.
    /// If the vector is shorter, the buffer is simply partially filled.
    pub fn from_vec(capacity: usize, vec: Vec<T>) -> Self {
        let mut rb = Self::new(capacity);
        let start = vec.len().saturating_sub(capacity);
        for item in vec.into_iter().skip(start) {
            rb.push(item);
        }
        rb
    }

    /// Push an item into the buffer. If at capacity, the oldest item is dropped.
    pub fn push(&mut self, item: T) {
        if self.buf.len() == self.capacity {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Get an item by logical index (0 = oldest).
    pub fn get(&self, index: usize) -> Option<&T> {
        self.buf.get(index)
    }

    pub fn oldest(&self) -> Option<&T> {
        self.buf.front()
    }

    pub fn newest(&self) -> Option<&T> {
        self.buf.back()
    }

    /// Iterator from oldest to newest.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.buf.iter()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T: Clone + fmt::Debug> fmt::Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RingBuffer")
            .field("capacity", &self.capacity)
            .field("len", &self.len())
            .field("items", &self.buf)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_buffer() {
        let rb: RingBuffer<i32> = RingBuffer::new(5);
        assert_eq!(rb.capacity(), 5);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
    }

    #[test]
    fn push_within_capacity() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        assert_eq!(rb.len(), 3);
        assert!(!rb.is_empty());
        assert_eq!(rb.oldest(), Some(&1));
        assert_eq!(rb.newest(), Some(&3));
    }

    #[test]
    fn push_overflows_drops_oldest() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4); // drops 1
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.oldest(), Some(&2));
        assert_eq!(rb.newest(), Some(&4));
    }

    #[test]
    fn overflow_multiple_times() {
        let mut rb = RingBuffer::new(2);
        for i in 0..10 {
            rb.push(i);
        }
        assert_eq!(rb.len(), 2);
        assert_eq!(rb.oldest(), Some(&8));
        assert_eq!(rb.newest(), Some(&9));
    }

    #[test]
    fn get_by_index() {
        let mut rb = RingBuffer::new(3);
        rb.push("a");
        rb.push("b");
        rb.push("c");
        assert_eq!(rb.get(0), Some(&"a"));
        assert_eq!(rb.get(1), Some(&"b"));
        assert_eq!(rb.get(2), Some(&"c"));
        assert_eq!(rb.get(3), None);
    }

    #[test]
    fn get_after_overflow() {
        let mut rb = RingBuffer::new(2);
        rb.push(10);
        rb.push(20);
        rb.push(30); // drops 10
        assert_eq!(rb.get(0), Some(&20));
        assert_eq!(rb.get(1), Some(&30));
    }

    #[test]
    fn iter_order_oldest_to_newest() {
        let mut rb = RingBuffer::new(4);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn iter_after_overflow() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4); // drops 1
        rb.push(5); // drops 2
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&3, &4, &5]);
    }

    #[test]
    fn clear_resets_buffer() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.oldest(), None);
        assert_eq!(rb.newest(), None);
    }

    #[test]
    fn clear_then_push() {
        let mut rb = RingBuffer::new(2);
        rb.push(1);
        rb.clear();
        rb.push(10);
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.oldest(), Some(&10));
        assert_eq!(rb.newest(), Some(&10));
    }

    #[test]
    fn oldest_newest_single_item() {
        let mut rb = RingBuffer::new(5);
        rb.push(42);
        assert_eq!(rb.oldest(), Some(&42));
        assert_eq!(rb.newest(), Some(&42));
    }

    #[test]
    fn from_vec_within_capacity() {
        let rb = RingBuffer::from_vec(5, vec![1, 2, 3]);
        assert_eq!(rb.len(), 3);
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn from_vec_exceeds_capacity_keeps_last() {
        let rb = RingBuffer::from_vec(3, vec![1, 2, 3, 4, 5]);
        assert_eq!(rb.len(), 3);
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&3, &4, &5]);
    }

    #[test]
    fn from_vec_empty() {
        let rb: RingBuffer<i32> = RingBuffer::from_vec(5, vec![]);
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
    }

    #[test]
    fn from_vec_exact_capacity() {
        let rb = RingBuffer::from_vec(3, vec![10, 20, 30]);
        assert_eq!(rb.len(), 3);
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&10, &20, &30]);
    }

    #[test]
    fn clone_produces_independent_copy() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        let mut cloned = rb.clone();
        cloned.push(3);
        assert_eq!(rb.len(), 2);
        assert_eq!(cloned.len(), 3);
    }

    #[test]
    fn debug_format() {
        let mut rb = RingBuffer::new(3);
        rb.push(10);
        rb.push(20);
        let dbg = format!("{:?}", rb);
        assert!(dbg.contains("RingBuffer"));
        assert!(dbg.contains("capacity"));
        assert!(dbg.contains("10"));
        assert!(dbg.contains("20"));
    }

    #[test]
    fn get_empty_buffer() {
        let rb: RingBuffer<i32> = RingBuffer::new(3);
        assert_eq!(rb.get(0), None);
    }

    #[test]
    fn capacity_one() {
        let mut rb = RingBuffer::new(1);
        rb.push(1);
        rb.push(2);
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.oldest(), Some(&2));
        assert_eq!(rb.newest(), Some(&2));
    }

    #[test]
    #[should_panic(expected = "capacity must be > 0")]
    fn zero_capacity_panics() {
        let _: RingBuffer<i32> = RingBuffer::new(0);
    }
}
