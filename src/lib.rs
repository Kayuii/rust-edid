mod cp437;
mod edid;
#[cfg(test)]
mod edid_test;
mod extension;
#[cfg(test)]
mod extension_test;

pub use edid::{parse, EDID, };
