//! Open Solutions — core library.
//!
//! An independent, clean-room reimplementation of a combustion equipment
//! selection toolkit. No vendor code, databases, brand assets or copyrighted
//! text are included. All engineering data lives in a user-supplied SQLite
//! database; see `schema.sql` for the schema.

pub mod calc;
pub mod model;
pub mod storage;
pub mod units;

pub use model::*;
pub use storage::Database;
