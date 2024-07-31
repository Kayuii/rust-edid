mod cp437;
mod edid;
#[cfg(test)]
mod edid_test;

pub use edid::{parse, EDID, };
