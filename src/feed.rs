use std::{
    sync::{
        Arc, Weak as WArc,
    },
};
use crate::{
    Reader,
};

/// An event feed — the source of events in a feed-based event system.
///
/// Feeds are the source of readers, which are subscibers of the feed, capable of recieving the events from the feed.
#[derive(Debug)]
pub struct Feed<Evt>
where Evt: Send {
    readers: Vec<WArc<Reader<Evt>>>,
}
impl<Evt> Feed<Evt>
where Evt: Send {
    /// Creates a new feed without any readers.
    ///
    /// If you expect a certain number of readers, use `with_reader_capacity`.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            readers: Vec::new(),
        }
    }
    /// Creates a new feed with internal storage allocated to be able to send to the specified amount of readers without reallocation.
    ///
    /// If you know in advance how many readers you will have, use this method. Otherwise, use `new` for simplicity.
    #[inline(always)]
    pub fn with_reader_capacity(capacity: usize) -> Self {
        Self {
            readers: Vec::with_capacity(capacity),
        }
    }
    /// Adds a new reader to the feed and returns it.
    ///
    /// The resulting structure can be freely sent and shared over threads.
    #[inline]
    pub fn add_reader(&mut self) -> Arc<Reader<Evt>> {
        let reader = Arc::new(Reader::new());
        self.readers.push(Arc::downgrade(&reader));
        reader
    }
    /// Sends an event to each reader by calling the specified closure once for each event.
    ///
    /// If your type implements `Clone`, simply using `send` would be more idiomatic.
    pub fn send_with<F>(&self, mut f: F)
    where F: FnMut() -> Evt {
        for reader in &self.readers {
            if let Some(reader) = reader.upgrade() {
                reader.recieve(f());
            }
        }
    }
    /// Removes references to dropped readers in order to release memory allocated for them and speed up calls to methods which send events.
    pub fn remove_dangling_readers(&mut self) {
        #[inline]
        fn find_dead_on_end<Evt: Send>(readers: &Vec<WArc<Reader<Evt>>>, current: usize) -> usize {
            let mut result = current;
            for i in (readers.len() - 1 - current)..0 {
                if readers[i].strong_count() == 0 {
                    result += 1;
                } else {break;}
            }
            result
        }
        // Keep track of how many dead readers we have on the end of the list.
        let mut dead_on_end = find_dead_on_end(&self.readers, 0);
        for i in 0..self.readers.len() {
            // If we reached the part where all elements are dead readers, we are done.
            if i >= self.readers.len() - dead_on_end {
                break;
            }
            if self.readers[i].strong_count() == 0 {
                // We found a dead reader. Let's move it to the end to remove them all quickly.
                let location_on_end = self.readers.len() - 1 - dead_on_end;
                self.readers.swap(i, location_on_end);
                dead_on_end += 1;
            }
            // Update the count.
            dead_on_end = find_dead_on_end(&self.readers, dead_on_end);
        }
        // Once we are here, all elements past a certain point are dead readers, which means that they can be removed.
        let new_size = self.readers.len() - dead_on_end;
        // Drop the elements past that point.
        self.readers.truncate(new_size);
    }
}
impl<Evt> Feed<Evt>
where Evt: Send + Clone {
    /// Sends the specified event to each reader by cloning it.
    #[inline(always)]
    pub fn send(&self, event: Evt) {
        self.send_with(|| event.clone())
    }
}
impl<Evt> Feed<Evt>
where Evt: Send + Default {
    /// Sends the default value of the event.
    ///
    /// The `Default` implementation for the event type is called once per reader, even if it implements `Clone`.
    #[inline(always)]
    pub fn send_default(&self) {
        self.send_with(Default::default)
    }
}

impl<Evt> Clone for Feed<Evt>
where Evt: Send {
    /// Clones the feed by creating a new feed which sends events to the same readers. The two feeds exist independently, i.e. adding a new reader to one of them will not modify another.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            readers: self.readers.clone()
        }
    }
}
impl<Evt> Default for Feed<Evt>
where Evt: Send {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}