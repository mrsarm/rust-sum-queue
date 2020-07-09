sum-queue
=========

`SumQueue` it's a **Rust** :crab: queue type that keeps a fixed number of
items by time, not capacity, similar to a cache, but with a simpler
and faster implementation. It also allows to get summarized stats
of the values on it:

```rust
use sum_queue::SumQueue;
use std::{time, thread};

// creates a queue where elements expires after 2 seconds
let queue: SumQueue<i32> = SumQueue::new(2);
queue.push(1);
queue.push(5);
queue.push(2);

// Check the peek without removing the element
assert_eq!(queue.peek(), Some(&1));

// Elements can be iterated as many times you want
println!("heap data: {:?}", queue.iter().collect::<Vec<_>>());  // [1, 5, 2]

// Check stats
let stats = queue.stats();
println!("Stats - min value in queue: {}", stats.min.unwrap());  // 1
println!("Stats - max value in queue: {}", stats.max.unwrap());  // 5
println!("Stats - sum value in queue: {}", stats.sum.unwrap());  // 8
println!("Stats - length of queue: {}", stats.len);              // 3

assert_eq!(queue.pop(), Some(1));
assert_eq!(queue.iter().collect::<Vec<_>>(), vec![&5, &2]);

// After a second the elements are still the same
thread::sleep(time::Duration::from_secs(1));
println!("Same elements: {:?}", queue.iter().collect::<Vec<_>>());      // [5, 2]

queue.push(50); // Add an element 1 second younger than the rest of elements
println!("Same elements + 50: {:?}", queue.iter().collect::<Vec<_>>()); // [5, 2, 50]

// Now let sleep 2 secs so the first elements expire
thread::sleep(time::Duration::from_secs(2));
println!("Just 50: {:?}", queue.iter().collect::<Vec<_>>());            // [50]

// 2 seconds later the latest element also expires
thread::sleep(time::Duration::from_secs(2));
println!("No elements: {:?}", queue.iter().collect::<Vec<_>>());        // []
```

Underneath it uses a [BinaryHeap](https://doc.rust-lang.org/std/collections/binary_heap/struct.BinaryHeap.html)
struct to keep the values.


About
-----

Source: https://github.com/mrsarm/rust-sum-queue

Authors: (2020) Mariano Ruiz <mrsarm@gmail.cm>

License: LGPL-3

