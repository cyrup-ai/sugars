//! Collection utilities and data structures

#![feature(auto_traits, negative_impls)]

pub mod byte_size;
/// A non-empty collection guaranteed to hold at least one value.
pub mod one_or_many;
/// A collection that can hold zero, one, or many values, optimized for minimal allocations.
pub mod zero_one_or_many;

/// Extension traits for JSON object syntax support
pub mod json_ext;

// Re-export main types
pub use byte_size::{ByteSize, ByteSizeExt};
pub use one_or_many::OneOrMany;
pub use zero_one_or_many::ZeroOneOrMany;

// Re-export extension traits
#[cfg(feature = "hashbrown-json")]
pub use json_ext::{
    CollectionJsonExtKString, CollectionJsonExtKV, CollectionJsonExtStringString,
    CollectionJsonExtStringV, JsonObjectExtKString, JsonObjectExtKV, JsonObjectExtStringString,
    JsonObjectExtStringV, TryCollectionJsonExtKString, TryCollectionJsonExtKV,
    TryCollectionJsonExtStringString, TryCollectionJsonExtStringV,
};
