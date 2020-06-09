//! Event systems implemented using event feeds.
//!
//! # The problem
//! The common pattern for implementing events in Rust prorgams is by passing a boxed closure (`Box<dyn FnMut(EventData)>`) to the source of the event using some sort of callback registration interface. This is commonly referred to as the **observer pattern**. The main advantage of this approach is that by attaching this kind of callback, the dependents of a library can extend that library and intergrate into something bigger while retaining the zero cost abstraction, since instead of polling the state in a busy loop, the library calls the callbacks itself, removing the overhead introduced by abstraction, which is one of the main goals of Rust as a language.
//!
//! This approach, however, introduces several new problems:
//! - **The callback cannot be rescheduled** or postponed, since the library which calls that callback cannot know anything about outer code and thus can only move the responsibility of managing whatever happens next on the callback itself, which complicates the callback and introduces new synchronization overhead if the program chooses to use threading, as it should if it desires to perform a complex and/or time-critical task, which brings us to the next implication:
//! - **The callback needs to be `Send` and `Sync`** if the library object which calls the callback also needs to stay `Send` and `Sync`. Both of these are a requirement for `RwLock`, while `Mutex` requires only `Send`. Still, if neither are upheld — which is often the case with objects locked to the main thread by operating system limitations (yes, I do mean GDI/WGL here) — the library object cannot even be put inside a `Mutex`.
//! - **There is also the overhead of dynamic dispatch**, which grows as the amount of callbacks grows. And since closures are typically short and specific, a lot of them should be expected in the long run, i.e. the callback abstraction is not entirely zero-cost as promised by its definition and goals.
//! - Speaking about overhead, **the callbacks are not ran instantly** either. The thread which gets unlucky enough to call the method which calls the callbacks has to carry the burden of executing every single one of them, which clogs the thread, preventing it from performing whatever it wanted to perform after the unfortunate call. If the job needs to continue immediately after the call, it should be offloaded, which brings even more complexity and overhead, considering how it'd be nescesarry to modify the code before the call when it gathers a critical mass of callbacks.
//!
//! The goal of this crate is to provide a solution to all of these problems using something called **event feeds**.
//!
//! # Event feeds
//! An event feed is the source of events which can *send* them to **event readers** — structures which are **subscribed to a specific feed**, which means that whenever the feed has a new event, it **sends** *the event* to the readers for them to process that event later. Notice how the structure has taken a turn towards lazy architecture. It definitely smells like iterators here, and that's exactly what the approach relies upon. The event readers iteratively perform the job of event loops processing the events which the feed sent them. Iteratively here means that the reader returns an iterator of the events, which, as it runs, cleans the inner queue of the reader and processes the events. Apply the `.for_each()` adapter to the iterator, set up a closure which `match`es on the events (or, if the event type is not an enumeration, processes it as applicable to the type), and you got yourself a version of the callback architecture with the aforementioned problems completely gone.
//!
//! Let's see how the event feed solves the aforementioned problems:
//! - Since the events are processed separately from being produced, they can be processed anytime, and that includes postponed processing. This allows for intricate scheduling using completely unrelated libraries, all being totally painless since nothing happens immediately.
//! - The sending part (the library object which produces events, i.e. the part which stores callbacks in the problematic case) does not store anything except for references to the readers, meaning that it always is `Send` and `Sync`. The only trait bound in this case is `Send` for the event type — the implementation takes care of the rest.
//! - No dynamic dispatch is mandatory — the readers process the events using just one closure, which can combine multiple handlers for different types of events, or perform multiple actions for one type of event.
//! - Posting an event into a feed still has `O(n)` complexity, but in this case `n` is the number of readers rather than the number of callbacks, and one reader does the job of multiple or all callbacks.
//!
//! # Usage
//! Basic usage:
//! ```
//! // Use the prelude module to quickly access the types in a renamed version which avoids name collisions.
//! use event_feed::prelude::*;
//!
//! // Create the feed. Initially it has no readers.
//! let mut feed = EventFeed::new();
//! // Create the reader to read events from the feed.
//! let reader = feed.add_reader();
//! // Send an event through the feed.
//! feed.send("Hello event feed!");
//!
//! // We can now read the event we sent.
//! assert_eq!(
//!     reader.read().next(),
//!     Some("Hello event feed!"),
//! );
//! // There are no more events in the feed.
//! assert_eq!(
//!     reader.read().next(),
//!     None,
//! );
//! // Send another event.
//! feed.send("This event will be displayed through a handler closure!");
//! feed.send("There are multiple events to display!");
//!
//! // Read multiple events using a closure.
//! {
//!     let mut event_number = 0;
//!     reader.read_with(|event| {
//!         println!(
//!             "Event number {num} says: {evt}",
//!             num = event_number,
//!             evt = event,
//!         );
//!         event_number += 1;
//!     });
//! }
//! ```
//!
//! # License
//! This crate is distributed under the Unlicense, including contributions not made by the original author.

mod feed;
pub use feed::*;

mod reader;
pub use reader::*;

/// A prelude module which reexports a minimal set of types you need to use event feeds which are renamed specifically to be glob-imported without any name conflicts (`use event_feed::prelude::*`).
pub mod prelude {
    pub use crate::{
        Feed as EventFeed,
        Reader as EventReader,
    };
}