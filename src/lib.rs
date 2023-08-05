pub mod networking;
pub mod protocols;
pub mod util;
pub mod portal;
pub mod msg;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::{sync::mpsc::{Sender}};
use crate::msg::Msg2Portal;

pub static IS_TLS_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
pub static TX_PORTAL: Lazy<Mutex<Option<Sender<Msg2Portal>>>> = Lazy::new(|| Mutex::new(None));