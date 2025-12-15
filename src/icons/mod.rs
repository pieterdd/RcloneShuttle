#![allow(dead_code)]
#![allow(unused_imports)]
pub mod icon_names {
    pub use shipped::*; // Include all shipped icons by default
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}
