mod base_test;
mod impl_test;
mod serde_test;
mod simple_error;

pub use foreignc::*;
pub use simple_error::SimpleError;

generate_free_string!();
generate_last_error!();