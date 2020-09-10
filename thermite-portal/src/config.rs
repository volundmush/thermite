// This struct stores the loaded configuration for the

use std::{
    collections::HashMap,
    net::IpAddr,
    error::Error,
    fs::read_to_string
};

use tokio_rustls::TlsAcceptor;

use toml;

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Keys {
    pub key: String,
    pub pem: String
}


#[derive(Deserialize)]
pub struct ServerConfig {
    pub tls: Option<String>,
    pub protocol: String,
    pub interface: String,
    pub port: u16
}

#[derive(Deserialize)]
pub struct Config {
    pub tls: HashMap<String, Keys>,
    pub interfaces: HashMap<String, IpAddr>,
    pub database: HashMap<String, String>,
    pub listeners: HashMap<String, ServerConfig>
}

impl Config {
    // Reads a toml file and
    pub fn from_file(file_name: String) -> Result<Self, Box<dyn Error>> {
        let conf_txt = read_to_string(String::from(file_name))?;
        let conf: Self = toml::from_str(&conf_txt)?;
        Ok(conf)
    }
}