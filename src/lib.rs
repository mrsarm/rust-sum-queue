//! `SumQueue` it's a queue struct that keeps a fixed number of
//! items by time, not capacity, similar to a cache, but with a simpler
//! and faster implementation. It also allows to get summarized stats
//! of the values on it at any time.
//! 
//! ## Examples
//! 
//! ```
//! use sum_queue::SumQueue;
//! use std::time::Duration;
//! use std::thread;
//! 
//! // creates a queue where elements expire after 2 seconds
//! let mut queue: SumQueue<i32> = SumQueue::new(Duration::from_secs(2));
//! queue.push(1);
//! queue.push(10);
//! queue.push(3);
//! 
//! // Check the peek without removing the element
//! assert_eq!(queue.peek(), Some(&1));
//! // elements are removed in the same order were pushed
//! assert_eq!(queue.pop(), Some(1));
//! assert_eq!(queue.pop(), Some(10));
//! assert_eq!(queue.pop(), Some(3));
//! assert_eq!(queue.pop(), None);
//!
//! // Lets puts elements again
//! queue.push(1);
//! queue.push(5);
//! queue.push(2);
//! // Elements can be iterated as many times as you want
//! println!("heap data: {:?}", queue.iter().collect::<Vec<_>>());  // [1, 5, 2]
//! 
//! // Check stats
//! let stats = queue.stats();
//! println!("Stats - min value in queue: {}", stats.min.unwrap());         // 1
//! println!("Stats - max value in queue: {}", stats.max.unwrap());         // 5
//! println!("Stats - sum all values in queue: {}", stats.sum.unwrap());    // 8
//! println!("Stats - length of queue: {}", stats.len);                     // 3
//! 
//! assert_eq!(queue.pop(), Some(1));
//! assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&5, &2]);
//! 
//! // After a second the elements are still the same
//! thread::sleep(Duration::from_secs(1));
//! println!("Same elements: {:?}", queue.iter().collect::<Vec<_>>());      // [5, 2]
//! 
//! queue.push(50); // Add an element 1 second younger than the rest of elements
//! println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>()); // [5, 2, 50]
//! 
//! // Now let sleep 2 secs so the first elements expire
//! thread::sleep(Duration::from_secs(2));
//! println!("Just 50: {:?}", queue.iter().collect::<Vec<_>>());            // [50]
//! 
//! // 2 seconds later the last element also expires
//! thread::sleep(Duration::from_secs(2));
//! println!("No elements: {:?}", queue.iter().collect::<Vec<_>>());        // []
//! ```
//!
//! ## Implementation
//!
//! Underneath uses a [`BinaryHeap`] struct to keep the values,
//! and implements the same methods: `push()`, `pop()`, `peek()` ...
//! although worth to note that the implementations of the `SumQueue` type take mutable
//! ownership of the `self` reference (eg. `peek(&mut self) -> Option<&T>`). That is
//! because the cleaning of the expired elements of the queue occurs each time
//! a method is called to read or write a value, including the `len()` method.
//!
//! So as long you manage only one instance of `SumQueue`, there is no
//! risk of excessive memory allocation, because while you push elements with the `push()`
//! method, or call any other method to read the queue you are taking care of removing
//! and deallocating the expired elements, but if you are using multiple instances, and
//! pushing too many items to some queues and not accessing others further, the memory usage
//! may growth with elements expired not been deallocated because you are not accessing
//! those queues to push, pop or get the stats of them. In that case you can at least
//! try to call often to the `len()` method to force the unused queues to remove and
//! deallocate the expired elements.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::binary_heap;
use std::ops::Add;
use std::time::{Instant, Duration};

/// Internal element used by `SumQueue` to hold the values.
struct QueueElement<T> {
    time: Instant,
    value: T
}

/// Stats of the queue.
///
/// It provides the following statistics: **min** and **max** value
/// in the queue, the **sum** of all the values and the **length**
/// of all elements hold in the queue.
///
/// The values are computed taking into account only
/// the existent elements in the queue, and not past
/// elements removed because expiration or because
/// they were removed.
///
/// You can get the stats object calling to
/// the [`SumQueue::stats()`] method of the queue:
///
/// ```
/// use std::time::Duration;
/// use sum_queue::SumQueue;
/// let mut queue = SumQueue::new(Duration::from_millis(800));
/// queue.push(-1);
/// queue.push(5);
/// queue.push(2);
/// let stats = queue.stats();
/// assert_eq!(stats.min, Some(-1));
/// assert_eq!(stats.max, Some(5));
/// assert_eq!(stats.sum, Some(6));
/// assert_eq!(stats.len, 3);
/// ```
///
/// But you can also get the stats
/// while pushing elements, which it's more
/// efficient than push and then get the stats:
///
/// ```
/// use std::time::Duration;
/// use sum_queue::SumQueue;
/// let mut queue = SumQueue::new(Duration::from_secs(1000));
/// queue.push(-1);
/// queue.push(5);
/// let stats = queue.push_and_stats(2);
/// assert_eq!(stats.min, Some(-1));
/// assert_eq!(stats.max, Some(5));
/// assert_eq!(stats.sum, Some(6));
/// assert_eq!(stats.len, 3);
/// ```
pub struct QueueStats<T: Ord + Add<Output = T>> {
    /// min value of the queue
    pub min: Option<T>,
    /// max value of the queue
    pub max: Option<T>,
    /// sum of all the values in the queue
    pub sum: Option<T>,
    /// size of the queue, same than [`SumQueue::len()`]
    pub len: usize
}

impl<T> PartialEq for QueueElement<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl<T> Eq for QueueElement<T> {}

impl<T> Ord for QueueElement<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        //! Reverse order to set lower number higher
        other.time.cmp(&self.time)
    }
}

impl<T> PartialOrd for QueueElement<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn now() -> Instant {
    Instant::now()
}

/// Main struct that holds the queue of elements.
///
/// There are different ways to create the queue:
///
/// ```
/// use std::time::Duration;
/// use sum_queue::SumQueue;
///
/// let mut queue: SumQueue<i32>;
///
/// // Create a queue with elements that expires after 60 seconds
/// queue = SumQueue::new(Duration::from_secs(60));
/// // Create with 500 milliseconds expiration and an initial capacity of 20 elements
/// queue = SumQueue::with_capacity(Duration::from_millis(500), 20);
/// ```
pub struct SumQueue<T> {
    /// the heap with the data
    heap: BinaryHeap<QueueElement<T>>,
    /// max time the elements will
    /// live in the queue.
    max_age: Duration,
}

impl<T> SumQueue<T> {
    /// Creates an empty `SumQueue`, where the elements inside
    /// will live `max_age_duration` at maximum.
    pub fn new(max_age_duration: Duration) -> SumQueue<T> {
        SumQueue {
            heap: BinaryHeap::<QueueElement<T>>::new(),
            max_age: max_age_duration,
        }
    }

    /// Creates an empty `SumQueue` with a specific initial capacity.
    /// This preallocates enough memory for `capacity` elements,
    /// so that the [`BinaryHeap`] inside the `SumQueue` does not have
    /// to be reallocated until it contains at least that many values.
    /// The elements inside the queue will live `max_age_duration` time at maximum.
    pub fn with_capacity(max_age_duration: Duration, capacity: usize) -> SumQueue<T> {
        SumQueue {
            heap: BinaryHeap::<QueueElement<T>>::with_capacity(capacity),
            max_age: max_age_duration,
        }
    }

    /// Pushes an item onto the heap of the queue.
    ///
    /// See [`BinaryHeap::push`] to known more about the time complexity.
    ///
    /// It returns the size of the queue, and before the element is pushed to the heap,
    /// it also drops all expired elements in the queue.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue = SumQueue::new(Duration::from_secs(60));
    /// queue.push(1);
    /// queue.push(5);
    /// assert_eq!(queue.push(2), 3);
    /// assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2]);
    /// ```
    pub fn push(&mut self, item: T) -> usize {
        let now = now();
        self.clear_oldest(now);
        self.heap.push(QueueElement {
            time: now,
            value: item
        });
        self.heap.len()
    }

    fn clear_oldest(&mut self, now: Instant) {
        while let Some(el) = self.heap.peek() {
            let peek_age = now - el.time;
            if peek_age > self.max_age {
                self.heap.pop();
            } else {
                break;
            }
        }
    }

    /// Drops all items.
    pub fn clear(&mut self) {
        self.heap.clear();
    }

    /// Returns the length of the heap.
    ///
    /// It takes a mutable reference of `self` because
    /// before return the size it also cleans all the
    /// expired elements of the queue, so only
    /// no expired elements are count.
    pub fn len(&mut self) -> usize {
        self.clear_oldest(now());
        self.heap.len()
    }

    /// Checks if the heap is empty. Expired elements are not taken
    /// into account because are droped by `is_empty()` before
    /// return the result.
    ///
    /// ```
    /// use std::time::Duration;
    /// use std::thread;
    /// use sum_queue::SumQueue;
    /// let mut queue = SumQueue::new(Duration::from_millis(600));
    ///
    /// assert!(queue.is_empty());
    ///
    /// queue.push(123);
    /// queue.push(555);
    ///
    /// assert!(!queue.is_empty());
    ///
    /// thread::sleep(Duration::from_secs(1));
    ///
    /// assert!(queue.is_empty());
    /// ```
    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the heap can hold without reallocating.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue: SumQueue<char> = SumQueue::with_capacity(Duration::from_secs(60), 5);
    /// assert_eq!(queue.capacity(), 5);
    /// assert_eq!(queue.len(), 0);
    /// ```
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    /// Returns the max time the elements will live in the queue.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue: SumQueue<char> = SumQueue::new(Duration::from_secs(60));
    /// assert_eq!(queue.max_age().as_secs(), 60);
    /// ```
    pub fn max_age(&self) -> Duration {
        self.max_age
    }

    /// Returns the first item in the heap, or `None` if it is empty.
    ///
    /// Before the element is returned, it also drops all expired
    /// elements from the queue.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue = SumQueue::new(Duration::from_secs(60));
    /// assert_eq!(queue.peek(), None);
    /// queue.push("Hello");
    /// queue.push("World");
    /// queue.push("!");
    /// assert_eq!(queue.peek(), Some(&"Hello"));
    /// ```
    pub fn peek(&mut self) -> Option<&T> {
        self.clear_oldest(now());
        self.heap.peek().map( |q_element| &q_element.value)
    }

    /// Removes the first item from the heap and returns it, or `None` if it
    /// is empty.
    ///
    /// Before the element is dropped from the queue and returned,
    /// it also drops all expired elements.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue = SumQueue::with_capacity(Duration::from_secs(60), 5);
    /// assert_eq!(queue.pop(), None);
    /// queue.push('a');
    /// queue.push('x');
    /// queue.push('c');
    /// assert_eq!(queue.pop(), Some('a'));
    /// assert_eq!(queue.pop(), Some('x'));
    /// assert_eq!(queue.pop(), Some('c'));
    /// assert_eq!(queue.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.clear_oldest(now());
        self.heap.pop().map( |q_element| q_element.value)
    }

    /// Returns an iterator visiting all values in the underlying heap, in
    /// same order they were pushed.
    ///
    /// Before return the iterator, it also drops all expired elements.
    ///
    /// The iterator does not change the state of the queue, this
    /// method takes ownership of the queue because as mentioned above
    /// it clears the expired elements before return the iterator, even
    /// if the iterator is not consumed later on.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue = SumQueue::new(Duration::from_secs(60));
    /// queue.push('a');
    /// queue.push('z');
    /// queue.push('x');
    /// assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&'a', &'z', &'x']);
    /// ```
    pub fn iter(&mut self) -> Iter<'_, T> {
        self.clear_oldest(now());
        Iter { iter: self.heap.iter() }
    }
}

impl<T: Copy + Ord + Add<Output = T>> SumQueue<T> {

    fn _stats(&mut self, len: usize) -> QueueStats<T> {
        let mut min = None; let mut max = None; let mut sum = None;
        for i in self.heap.iter().map(|x| x.value) {
            if min == None || Some(i) < min {
                min = Some(i);
            }
            if max == None || Some(i) > max {
                max = Some(i);
            }
            sum = match sum {
                Some(s) => Some(s + i),
                None => Some(i)
            };
        }
        QueueStats {
            min, max, sum, len
        }
    }

    /// Get statistics of the queue. The type of the elements
    /// on it needs to implements the `Copy`, `Ord` and `Add` traits.
    ///
    /// Before the stats are returned, it also drops all expired elements.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue: SumQueue<i64> = SumQueue::new(Duration::from_secs(1000));
    /// queue.push(-10);
    /// queue.push(50);
    /// queue.push(40);
    /// queue.push(20);
    /// let stats = queue.stats();
    /// assert_eq!(stats.min, Some(-10));
    /// assert_eq!(stats.max, Some(50));
    /// assert_eq!(stats.sum, Some(100));
    /// assert_eq!(stats.len, 4);
    /// ```
    ///
    /// See also `push_and_stats`.
    pub fn stats(&mut self) -> QueueStats<T> {
        let len = self.len();
        self._stats(len)
    }

    /// Pushes an item onto the heap of the queue, and returns
    /// the stats of the queue. The type of the elements
    /// on it need to implements the `Copy`, `Ord` and `Add`
    /// traits.
    ///
    /// Before push and return the stats, it also drops all expired elements.
    ///
    /// ```
    /// use std::time::Duration;
    /// use sum_queue::SumQueue;
    /// let mut queue: SumQueue<i64> = SumQueue::new(Duration::from_secs(1000));
    /// queue.push(-10);
    /// queue.push(50);
    /// queue.push(40);
    /// let stats = queue.push_and_stats(20);
    /// assert_eq!(stats.min, Some(-10));
    /// assert_eq!(stats.max, Some(50));
    /// assert_eq!(stats.sum, Some(100));
    /// assert_eq!(stats.len, 4);
    /// ```
    ///
    /// Use `push` instead if you don't need the stats
    /// or the elements in the heap don't implement
    /// any of the required traits.
    pub fn push_and_stats(&mut self, item: T) -> QueueStats<T> {
        let len = self.push(item);
        self._stats(len)
    }
}

/// An iterator over the elements of a `SumQueue`.
///
/// This `struct` is created by [`SumQueue::iter()`]. See its
/// documentation for more.
pub struct Iter<'a, T: 'a> {
    iter: binary_heap::Iter<'a, QueueElement<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let element = self.iter.next()?;
        Some(&element.value)
    }
}

mod tests {
    pub use std::thread;
    pub use std::time::Duration;
    pub use crate::SumQueue;

    #[test]
    fn push_pop_peek() {
        let mut queue: SumQueue<i32> = SumQueue::new(Duration::from_secs(60));
        queue.push(1);
        queue.push(5);
        assert_eq!(queue.push(2), 3);  // push return queue length
        assert_eq!(queue.peek(), Some(&1));
        assert_eq!(queue.peek(), Some(&1));  // still the same
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(5));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.peek(), None);
        queue.push(1_000);
        assert_eq!(queue.peek(), Some(&1_000));
    }

    #[test]
    fn push_pop_peek_refs() {
        let mut queue: SumQueue<&i32> = SumQueue::new(Duration::from_secs(60));
        queue.push(&1);
        queue.push(&5);
        assert_eq!(queue.push(&2), 3);
        assert_eq!(queue.peek(), Some(&&1));
        assert_eq!(queue.peek(), Some(&&1));
        assert_eq!(queue.pop(), Some(&1));
        assert_eq!(queue.pop(), Some(&5));
        assert_eq!(queue.pop(), Some(&2));
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.peek(), None);
        queue.push(&1_000);
        assert_eq!(queue.peek(), Some(&&1_000));
    }

    #[test]
    fn len_clear() {
        let mut queue: SumQueue<char> =SumQueue::with_capacity(
            Duration::from_secs(60), 2); // small capacity shouldn't be a problem
        assert_eq!(queue.len(), 0);
        queue.push('a');
        queue.push('b');
        queue.push('c');
        assert_eq!(queue.len(), 3);
        queue.pop();
        assert_eq!(queue.len(), 2);
        queue.clear();
        assert_eq!(queue.len(), 0);
        queue.push('$');
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn iter() {
        let mut queue: SumQueue<&str> = SumQueue::with_capacity(
            Duration::from_secs(60), 20);
        queue.push("Hey");
        queue.push("You");
        queue.push("!");
        println!("heap data with &str references: {:?}", queue.iter().collect::<Vec<_>>());
        // data can be iterated as many time as you want
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&"Hey", &"You", &"!"]);
        print!("heap data, iterate one by one... :");
        for word in queue.iter() {  // iterate one by one don't crash
            print!(" {}", word)
        }
        println!();
    }

    #[test]
    fn expire() {
        let max_age_secs = 2;
        let mut queue: SumQueue<i32> = SumQueue::with_capacity(
            Duration::from_secs(max_age_secs), 20);
        queue.push(1);
        queue.push(5);
        queue.push(2);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2]);
        println!("Elements in queue with max age of {} secs: {:?}",
                 max_age_secs, queue.iter().collect::<Vec<_>>());

        sleep_secs(1);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2]);
        println!("No expiration yet, same elements: {:?}", queue.iter().collect::<Vec<_>>());

        println!("\nAdding element 50 ...");
        queue.push(50);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2, &50]);
        println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>());

        sleep_secs(1);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&50]);
        println!("Expired original list, only 50 in the list: {:?}",
                 queue.iter().collect::<Vec<_>>());

        sleep_secs(2);
        assert_eq!(queue.iter().collect::<Vec<_>>().len(), 0);
        println!("No elements kept: {:?}", queue.iter().collect::<Vec<_>>());
    }

    #[test]
    fn expire_less_one_sec() {
        let max_age_millis = 200;
        let mut queue: SumQueue<i32> = SumQueue::with_capacity(
            Duration::from_millis(max_age_millis), 20);
        queue.push(1);
        queue.push(5);
        queue.push(2);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2]);
        println!("Elements in queue with max age of {} millis: {:?}",
                 max_age_millis, queue.iter().collect::<Vec<_>>());

        sleep_millis(100);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2]);
        println!("No expiration yet, same elements: {:?}", queue.iter().collect::<Vec<_>>());

        println!("\nAdding element 50 ...");
        queue.push(50);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&1, &5, &2, &50]);
        println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>());

        sleep_millis(100);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&50]);
        println!("Expired original list, only 50 in the list: {:?}",
                 queue.iter().collect::<Vec<_>>());

        sleep_millis(200);
        assert_eq!(queue.iter().collect::<Vec<_>>().len(), 0);
        println!("No elements kept: {:?}", queue.iter().collect::<Vec<_>>());
    }

    #[test]
    fn stats() {
        let mut queue: SumQueue<i64> = SumQueue::new(Duration::from_secs(1000));
        let mut stats = queue.stats();
        assert_eq!(stats.min, None);
        assert_eq!(stats.max, None);
        assert_eq!(stats.sum, None);
        assert_eq!(stats.len, 0);

        queue.push(-10);
        queue.push(50);
        queue.push(20);
        queue.push(20);
        stats = queue.stats();
        assert_eq!(stats.min, Some(-10));
        assert_eq!(stats.max, Some(50));
        assert_eq!(stats.sum, Some(80));
        assert_eq!(stats.len, 4);

        queue.clear();
        stats = queue.stats();
        assert_eq!(stats.min, None);
        assert_eq!(stats.max, None);
        assert_eq!(stats.sum, None);
        assert_eq!(stats.len, 0);

        queue.push(100_000);
        stats = queue.stats();
        assert_eq!(stats.min, Some(100_000));
        assert_eq!(stats.max, Some(100_000));
        assert_eq!(stats.sum, Some(100_000));
        assert_eq!(stats.len, 1);

        queue.push(5);
        stats = queue.push_and_stats(1);
        assert_eq!(stats.min, Some(1));
        assert_eq!(stats.max, Some(100_000));
        assert_eq!(stats.sum, Some(100_006));
        assert_eq!(stats.len, 3);
    }

    #[cfg(test)]
    fn sleep_secs(dur_secs: u64) {
        println!("\nSleeping {} secs ...", dur_secs);
        thread::sleep(Duration::from_secs(dur_secs));
    }

    #[cfg(test)]
    fn sleep_millis(dur_millis: u64) {
        println!("\nSleeping {} millis ...", dur_millis);
        thread::sleep(Duration::from_millis(dur_millis));
    }
}
