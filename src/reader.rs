use std::{
    collections::VecDeque,
    iter::{
        Iterator, IntoIterator, DoubleEndedIterator, ExactSizeIterator, FusedIterator,
    },
    fmt::{self, Formatter},
};
use parking_lot::{
    Mutex,
    MutexGuard,
};

/// Recieves events from event feeds and queues them until they are processed.
pub struct Reader<Evt>
where Evt: Send {
    queue: Mutex<VecDeque<Evt>>,
}
impl<Evt> Reader<Evt>
where Evt: Send {
    /// Creates an iterator which reads and removes the events from the queue.
    ///
    /// The queue's mutex remains locked for the entire lifetime of the returned iterator, which means that all calls to the feed's `send_with`, `send` and others will block. If you do not want that behavior, drop the iterator after a number of iterations and create a new one, which should cause a fair mutex unlock if it ran for long enough, allowing the feed to send new events.
    #[inline(always)]
    pub fn read(&self) -> ReaderIter<'_, Evt> {
        ReaderIter {
            queue: self.queue.lock()
        }
    }
    /// Reads the entire queue by using the specified closure to process the events. Useful for simple event handling, i.e. if the closure doesn't return anything depending on how it processes the events. If it does, using `read` directly is necesarry.
    ///
    /// See `read` for the mutex-related implications of using this.
    #[inline(always)]
    pub fn read_with<F>(&self, f: F)
    where F: FnMut(Evt) {
        self.read().for_each(f);
    }
    /// Creates a reader.
    ///
    /// This method is not meant to be exposed to library users. The correct method which you should use for creating readers is `Feed`'s `add_reader`.
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }
    /// Recieves the specified event by putting it into the queue.
    ///
    /// This method is not meant to be exposed to library users. The only place where it should be called is `Feed`'s `send_with` and its variations.
    #[inline]
    pub(crate) fn recieve(&self, event: Evt) {
        let mut queue = self.queue.lock();
        queue.push_back(event);
    }
}
impl<Evt> fmt::Debug for Reader<Evt>
where Evt: fmt::Debug + Send {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Reader")
            .field("queue", &*self.queue.lock())
        .finish()
    }
}

/// An iterator used for processing events in an event reader's queue.
///
/// Use the `read` method to acquire this.
pub struct ReaderIter<'r, Evt>
where Evt: Send {
    queue: MutexGuard<'r, VecDeque<Evt>>,
}
impl<'r, Evt> Iterator for ReaderIter<'r, Evt>
where Evt: Send {
    type Item = Evt;
    
    #[inline(always)]
    fn next(&mut self) -> Option<Evt> {
        self.queue.pop_front()
    }
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.len(),
            Some(self.len())
        )
    }
    #[inline(always)]
    fn count(self) -> usize {
        self.len()
    }
}
impl<'r, Evt> DoubleEndedIterator for ReaderIter<'r, Evt>
where Evt: Send {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Evt> {
        self.queue.pop_back()
    }
}
impl<'r, Evt> ExactSizeIterator for ReaderIter<'r, Evt>
where Evt: Send {
    #[inline(always)]
    fn len(&self) -> usize {
        self.queue.len()
    }
}
impl<Evt> FusedIterator for ReaderIter<'_, Evt>
where Evt: Send {}