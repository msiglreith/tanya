use std::collections::LinkedList;
use std::ops::Range;

use super::EntityId;

// TODO: mostly from the dx12 free list allocator of gfx-rs
//       modify it to fit the needs for this use-case better.

#[derive(Debug)]
pub struct Allocator {
    size: EntityId,
    free_list: LinkedList<Range<EntityId>>,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            size: 0,
            free_list: LinkedList::new(),
        }
    }

    pub fn with_capacity(size: EntityId) -> Self {
        // Node spanning the whole range.
        let node = Range {
            start: 0,
            end: size,
        };
        let mut free_list = LinkedList::new();
        free_list.push_front(node);
        Allocator { size, free_list }
    }

    pub fn append(&mut self, num: EntityId) {
        self.deallocate(self.size..self.size + num);
        self.size += num;
    }

    pub fn allocate(&mut self, mut size: EntityId) -> Option<Range<EntityId>> {
        // TODO: refactor

        if size == 0 {
            return Some(Range { start: 0, end: 0 });
        }

        // Find first node  ..
        let mut split_index = None;
        for (index, _) in self.free_list.iter().enumerate() {
            if true {
                // Found a candidate.
                split_index = Some(index);
                break;
            }
        }

        split_index.map(|index| {
            let mut tail = self.free_list.split_off(index);

            // The first list element of `tail` will be split into two nodes.
            let mut node = tail.pop_front().unwrap();
            size = size.min(node.end - node.start);
            let allocated = Range {
                start: node.start,
                end: node.start + size,
            };
            node.start += size;

            // Our new list will look like this considering our 2nd node part
            // is not empty:
            // Before: [old list] -- [allocated|node] -- [tail]
            // After:  [old list] -- [node] -- [tail] || [allocated]
            if node.start < node.end {
                self.free_list.push_back(node);
            }
            self.free_list.append(&mut tail);

            allocated
        })
    }

    pub fn deallocate(&mut self, mut range: Range<EntityId>) {
        // early out for invalid or empty ranges
        if range.end <= range.start {
            return;
        }

        // Find node where we want to insert the range.
        // We aim to merge consecutive nodes into larger ranges, so we maintain
        // a sorted list.
        let mut insert_index = self.free_list.len(); // append at the end
        for (index, node) in self.free_list.iter().enumerate() {
            if node.start > range.start {
                // Found a better place!
                insert_index = index;
                break;
            }
        }

        // New list: [head] -- [node] -- [tail]
        let mut tail = self.free_list.split_off(insert_index);

        // Try merge with prior node from [head]
        let pre_node = self.free_list.pop_back();
        pre_node.map(|pre_node| {
            if pre_node.end == range.start {
                // Merge both nodes
                range.start = pre_node.start;
            } else {
                // Re-insert the previous node
                self.free_list.push_back(pre_node);
            }
        });

        // Try merge with next node from [tail]
        let next_node = tail.pop_front();
        next_node.map(|next_node| {
            if range.end == next_node.start {
                // Merge both nodes
                range.end = next_node.end;
            } else {
                // Re-insert the next node
                tail.push_front(next_node);
            }
        });

        self.free_list.push_back(range);
        self.free_list.append(&mut tail);
    }
}

#[cfg(test)]
mod tests {
    use super::Allocator;

    #[test]
    fn test_allocate() {
        let mut allocator = Allocator::with_capacity(8);
        assert_eq!(Some(0..4), allocator.allocate(4));
        assert_eq!(Some(4..6), allocator.allocate(2));
        assert_eq!(Some(6..8), allocator.allocate(2));
        assert_eq!(None, allocator.allocate(1));
    }

    #[test]
    fn test_merge() {
        let mut allocator = Allocator::with_capacity(8);
        let front = allocator.allocate(4).unwrap();
        let middle = allocator.allocate(2).unwrap();
        let back = allocator.allocate(2).unwrap();

        allocator.deallocate(front);
        allocator.deallocate(back);

        assert_eq!(Some(0..4), allocator.allocate(5));
        allocator.deallocate(middle);
        assert_eq!(Some(4..8), allocator.allocate(9));
    }
}
