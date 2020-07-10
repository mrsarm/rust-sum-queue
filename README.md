sum-queue
=========

`SumQueue` it's a **Rust** :crab: queue type that keeps a fixed number of
items by time, not capacity, similar to a cache, but with a simpler
and faster implementation. It also allows to get summarized stats
of the values on it at any time.

## Examples

```rust
use sum_queue::SumQueue;
use std::{time, thread};

// creates a queue where elements expires after 2 seconds
let mut queue: SumQueue<i32> = SumQueue::new(2);
queue.push(1);
queue.push(10);
queue.push(3);

// Check the peek without removing the element
assert_eq!(queue.peek(), Some(&1));
// elements are removed in the same order were pushed
assert_eq!(queue.pop(), Some(1));
assert_eq!(queue.pop(), Some(10));
assert_eq!(queue.pop(), Some(3));
assert_eq!(queue.pop(), None);

// Lets puts elements again
queue.push(1);
queue.push(5);
queue.push(2);
// Elements can be iterated as many times you want
println!("heap data: {:?}", queue.iter().collect::<Vec<_>>());  // [1, 5, 2]

// Check stats
let stats = queue.stats();
println!("Stats - min value in queue: {}", stats.min.unwrap());         // 1
println!("Stats - max value in queue: {}", stats.max.unwrap());         // 5
println!("Stats - sum all values in queue: {}", stats.sum.unwrap());    // 8
println!("Stats - length of queue: {}", stats.len);                     // 3

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

// 2 seconds later the last element also expires
thread::sleep(time::Duration::from_secs(2));
println!("No elements: {:?}", queue.iter().collect::<Vec<_>>());        // []
```

## Implementation

Underneath uses a [BinaryHeap](https://doc.rust-lang.org/std/collections/binary_heap/struct.BinaryHeap.html)
struct to keep the values, and implements the same methods: `push()`, `pop()`, `peek()` ...
although worth to note that the implementations of the `SumQueue` type take mutable
ownership of the `self` reference (eg. `peek(&mut self) -> Option<&T>`). That is
because the cleaning of the expired elements of the queue occurs each time
a method is called to read or write a value, including the `len()` method.

So as long you manage only one instance of `SumQueue`, there is no
risk of excessive memory allocation, because while you push elements with the `push()`
method, or call any other method to read the queue you are taking care of removing
and deallocating the expired elements, but if you are using multiple instances, and
pushing too many items to some queues and not accessing others further, the memory usage
may growth with elements expired not been deallocated because you are not accessing
those queues to push, pop or get the stats of them. In that case you can at least
try to call often to the `len()` method to force the unused queues to remove and
deallocate the expired elements.


## About

**Source**: https://github.com/mrsarm/rust-sum-queue

**Authors**: (2020) Mariano Ruiz <mrsarm@gmail.com>

**Documentation**: https://docs.rs/sum-queue/

**License**: LGPL-3
