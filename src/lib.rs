// TODO add stats code
// TODO add doc

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::binary_heap::Iter;
use std::time::{SystemTime, UNIX_EPOCH};
use std::iter::Map;

#[derive(Debug)]
pub struct QueueElement<T: Ord + Copy> {
    time: u64,  // "Unix" Time, or seconds since EPOCH when the value was added
    value: T
}

impl<T: Ord + Copy> PartialEq for QueueElement<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl<T: Ord + Copy> Eq for QueueElement<T> {}

impl<T: Ord + Copy> Ord for QueueElement<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order to set lower number higher
        other.time.cmp(&self.time)
    }
}

impl<T: Ord + Copy> PartialOrd for QueueElement<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct SumQueue<T: Ord + Copy> {
    heap: BinaryHeap<QueueElement<T>>,
    max_age: u64    // max age in seconds
}

impl<T: Ord + Copy> SumQueue<T> {
    pub fn new(max_age: u64) -> SumQueue<T> {
        SumQueue { heap: BinaryHeap::<QueueElement<T>>::new(), max_age }
    }

    pub fn with_capacity(max_age_secs: u64, capacity: usize) -> SumQueue<T> {
        SumQueue {
            heap: BinaryHeap::<QueueElement<T>>::with_capacity(capacity),
            max_age: max_age_secs
        }
    }

    pub fn push(&mut self, item: T) {
        let now = self.now();
        self.clear_oldest(now);
        self.heap.push(QueueElement {
            time: now,
            value: item
        });
    }

    pub fn clear_oldest(&mut self, now: u64) {
        while let Some(el) = self.heap.peek() {
            let peek_age = now - el.time;
            if peek_age > self.max_age {
                self.heap.pop();
            } else {
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.heap.clear();
    }

    fn now(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("<-- Time went backwards").as_secs()
    }

    pub fn len(&mut self) -> usize {
        self.clear_oldest(self.now());
        self.heap.len()
    }

    pub fn iter(&mut self) -> Map<Iter<QueueElement<T>>, fn(&QueueElement<T>) -> T> {
        self.clear_oldest(self.now());
        self.heap.iter().map(|x| x.value)
    }

    pub fn peek(&mut self) -> Option<T> {
        self.clear_oldest(self.now());
        self.heap.peek().map( |q_element| q_element.value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.clear_oldest(self.now());
        self.heap.pop().map( |q_element| q_element.value)
    }
}

mod tests {
    pub use std::thread;
    pub use std::time::Duration;
    pub use crate::SumQueue;

    #[test]
    fn push_pop() {
        let mut queue: SumQueue<i32> = SumQueue::new(60);
        queue.push(1);
        queue.push(5);
        queue.push(2);
        assert_eq!(queue.peek(), Some(1));
        assert_eq!(queue.peek(), Some(1));  // still the same
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(5));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.peek(), None);
        queue.push(1_000);
        assert_eq!(queue.peek(), Some(1_000));
    }

    #[test]
    fn len_clear() {
        let mut queue: SumQueue<char> = SumQueue::with_capacity(60, 20);
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
        let mut queue: SumQueue<&str> = SumQueue::with_capacity(60, 20);
        queue.push("Hey");
        queue.push("You");
        queue.push("!");
        println!("heap data with &str references: {:?}", queue.iter().collect::<Vec<_>>());
        // data can be iterated as many time as you want
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec!["Hey", "You", "!"]);
        print!("heap data, iterate one by one... :");
        for word in queue.iter() {
            print!(" {}", word)
        }
        println!();
    }

    #[test]
    fn expire() {
        let max_age_secs = 2;
        let mut queue: SumQueue<i32> = SumQueue::with_capacity(max_age_secs, 20);
        queue.push(1);
        queue.push(5);
        queue.push(2);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![1, 5, 2]);
        println!("Elements in queue with max age of {} secs: {:?}",
                 max_age_secs, queue.iter().collect::<Vec<_>>());

        sleep_secs(1);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![1, 5, 2]);
        println!("No expiration yet, same elements: {:?}", queue.iter().collect::<Vec<_>>());

        println!("\nAdding element 50 ...");
        queue.push(50);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![1, 5, 2, 50]);
        println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>());

        sleep_secs(2);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![50]);
        println!("Expired original list, only 50 in the list: {:?}",
                 queue.iter().collect::<Vec<_>>());

        sleep_secs(2);
        assert_eq!(queue.iter().collect::<Vec<_>>(), vec![]);
        println!("No elements kept: {:?}", queue.iter().collect::<Vec<_>>());
    }

    #[cfg(test)]
    fn sleep_secs(dur_secs: u64) {
        println!("\nSleeping {} secs ...", dur_secs);
        thread::sleep(Duration::from_secs(dur_secs));
    }
}
