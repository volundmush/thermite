use legion::*;
use std::{
    net::{SocketAddr, IpAddr}
};
use crate::{
    net::managers::{PollSystem, NetManager},
    components::net::{Protocol, Listen, Connection, NetComp}
};
use mio::prelude::*;

pub fn register_listener(world: &mut World, manager: &mut NetManager, listener: TcpListener, protocol: Protocol) -> std::io::Result {
    let new_ent = world.push((NetComp::default(),));
    let mut entr = world.entry(new_ent).unwrap();
    manager.listeners.register(&listener, new_ent)?;
    
    let mut listen_comp = Listen {
        entity: new_ent,
        protocol,
        addr: listener.local_addr().unwrap(),
        listener,
    };

    entry.add_component(listen_comp);
    Ok(())
}

pub fn accept_incoming(world: &mut World, manager: &mut NetManager) {
    manager.listeners.poll()?;
    for ev in manager.listeners.events {

    }
}

pub fn read_streams(world: &mut World, manager: &mut NetManager) {
    manager.readers.poll()?;
    for ev in manager.readers.events {
        
    }
}

pub fn write_streams(world: &mut World, manager: &mut Netmanager) {
    manager.writers.poll()?;
    for ev in manager.writers.events {
        
    }
}