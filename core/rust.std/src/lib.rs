pub mod jobserver;
pub mod modifier;
pub mod objectstore;
pub mod permodule_functions;
pub mod priorityset;
pub mod utils;
pub mod value;

type Error = Box<dyn std::error::Error + Send + Sync>;
