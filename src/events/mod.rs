//! Event dispatcher and callback management.

mod dispatcher;
mod callback;

pub use dispatcher::EventDispatcher;
pub use callback::{Callback, CallbackRegistry};
pub use crate::protocol::PusherEvent;
