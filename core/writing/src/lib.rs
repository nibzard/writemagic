//! Writing domain - Document management, content editing, and project organization

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;
pub mod events;

// Re-export public types
pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use repositories::*;
pub use events::*;