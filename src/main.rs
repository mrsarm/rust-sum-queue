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
        max_age: u64      //TODO Implement max_age usage!
    }

    impl<T: Ord + Copy> SumQueue<T> {
        pub fn new(max_age: u64) -> SumQueue<T> {
            SumQueue { heap: BinaryHeap::<QueueElement<T>>::new(), max_age }
        }
        pub fn with_capacity(max_age: u64, capacity: usize) -> SumQueue<T> {
            SumQueue { heap: BinaryHeap::<QueueElement<T>>::with_capacity(capacity), max_age }
        }
        pub fn push(&mut self, item: T) {
            //TODO Remove "older" items first
            self.heap.push(QueueElement {
                time: SystemTime::now().duration_since(UNIX_EPOCH)
                                       .expect("<-- Time went backwards").as_secs(),
                value: item
            });
        }
        pub fn iter(&self) -> Map<Iter<QueueElement<T>>, fn(&QueueElement<T>) -> T> {
            self.heap.iter().map(|x| x.value)
        }
    }
}

// TODO Replace `main` by tests
fn main() {
    use sum_queue::SumQueue;
    let mut queue: SumQueue<i32> = SumQueue::with_capacity(100, 20);
    queue.push(1);
    queue.push(5);
    queue.push(2);
    //assert_eq!(queue.heap.peek(), Some(&5));
    println!("heap data: {:?}", queue.iter().collect::<Vec<_>>());
    print!("heap data, again... :");
    for num in queue.iter() {
        print!(" {}", num)
    }
    println!()
}
