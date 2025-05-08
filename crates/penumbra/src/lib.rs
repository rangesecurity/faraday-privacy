//! We need to work around the recursion limit for send+sync evaluation of penumbra types
#![recursion_limit = "512"]

#[cfg(any(feature = "api-server", feature = "api-client"))]
pub mod api;
pub mod client;
pub mod types;

//#[cfg(feature = "cli")]
//pub mod bin;