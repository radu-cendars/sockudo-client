//! Delta compression for bandwidth optimization.

mod channel_state;
pub mod decoders;
mod manager;
mod types;

pub use channel_state::ChannelState;
pub use decoders::{
    decode_base64, encode_base64, DeltaDecoder, FossilDeltaDecoder, Xdelta3Decoder,
};
pub use manager::DeltaManager;
pub use types::*;
