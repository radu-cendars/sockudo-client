//! Delta compression for bandwidth optimization.

mod manager;
mod decoders;
mod channel_state;
mod types;

pub use manager::DeltaManager;
pub use decoders::{DeltaDecoder, FossilDeltaDecoder};
pub use channel_state::ChannelState;
pub use types::*;
