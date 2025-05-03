mod entry;
mod model;

// APIs that are accessed by the host bsc,
// these are not useful to brain implementers.
pub mod internal;

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "native")]
pub type NativeApi = native::NativeApi;

pub use entry::*;
pub use model::*;
