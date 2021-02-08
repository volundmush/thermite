use tokio::{
    sync::mpsc::{Sender, Receiver, channel}
};

pub enum Msg2Portal {
    Kill,
    ClientDisconnected(String)
}

pub struct Portal {
    pub tx_portal: Sender<Msg2Portal>,
    rx_portal: Receiver<Msg2Portal>
}

impl Portal {
    pub fn new() -> Self {
        let (tx_portal, rx_portal) = channel(10);
        Self {
            tx_portal,
            rx_portal
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(f_msg) = self.rx_portal.recv().await {
                match f_msg {
                    Msg2Portal::Kill => {
                        break;
                    }
                    _ => {

                    }
                }
            }
        }
    }

}