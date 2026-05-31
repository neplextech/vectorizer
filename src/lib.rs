#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

pub mod config;
pub mod converter;
mod errors;
mod image;
pub mod optimizer;
pub mod svg;
mod svg_bridge;
mod types;
mod vectorizer;

pub use image::*;
pub use optimizer::*;
pub use svg_bridge::*;
pub use types::*;
pub use vectorizer::*;
