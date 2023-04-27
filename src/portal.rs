use std::{
    collections::{HashMap},
    time::{Duration, Instant}
};

use tokio::{
    sync::mpsc::{Sender, Receiver, channel},
    task::JoinHandle,
    time
};

use tokio_stream::wrappers::IntervalStream;

use serde_json::Value as JsonValue;

use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

use crate::{
    protocols::link::protocol::{LinkStub}
};
use crate::msg::{Msg2Link, Msg2MudProtocol, Msg2Portal, Msg2PortalFromClient, Msg2PortalFromLink};
use crate::protocols::ProtocolLink;


pub struct Portal {
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>,
    link: Option<LinkStub>,
    clients: HashMap<usize, ProtocolLink>
}

impl Portal {
    pub fn new() -> Self {
        let (tx_portal, rx_portal) = channel(10);
        Self {
            tx_portal,
            rx_portal,
            clients: Default::default(),
            link: Default::default()
        }
    }
}