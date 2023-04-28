use std::sync::atomic::AtomicUsize;

pub mod link;
pub mod telnet;
pub mod web;

pub static CONNECTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);