use std::{time, thread};

// TODO Move everything to a lib module, and add doc
pub mod sum_queue {
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
}

// TODO Replace `main` by tests
fn main() {
    use sum_queue::SumQueue;
    let mut queue: SumQueue<i32> = SumQueue::with_capacity(2, 20);
    queue.push(1);
    queue.push(5);
    queue.push(2);
    assert_eq!(queue.peek(), Some(1));
    println!("heap data: {:?}", queue.iter().collect::<Vec<_>>());  // [1, 5, 2]
    print!("heap data, again... :");
    for num in queue.iter() {
        print!(" {}", num)
    }
    println!();
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.iter().collect::<Vec<_>>(), vec![5, 2]);
    println!("heap data with the last removed: {:?}", queue.iter().collect::<Vec<_>>());

    thread::sleep(time::Duration::from_secs(1));

    println!("Same elements: {:?}", queue.iter().collect::<Vec<_>>());
    queue.push(50);
    println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>());

    thread::sleep(time::Duration::from_secs(2));

    println!("Just 50: {:?}", queue.iter().collect::<Vec<_>>());

    thread::sleep(time::Duration::from_secs(2));
    println!("No elements: {:?}", queue.iter().collect::<Vec<_>>());
}
