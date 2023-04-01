pub const NULL: u8 = 0;
pub const BEL: u8 = 7;
pub const CR: u8 = 13;
pub const LF: u8 = 10;
pub const SGA: u8 = 3;
pub const TELOPT_EOR: u8 = 25;
pub const NAWS: u8 = 31;
pub const LINEMODE: u8 = 34;
pub const EOR: u8 = 239;
pub const SE: u8 = 240;
pub const NOP: u8 = 241;
pub const GA: u8 = 249;
pub const SB: u8 = 250;
pub const WILL: u8 = 251;
pub const WONT: u8 = 252;
pub const DO: u8 = 253;
pub const DONT: u8 = 254;
pub const IAC: u8 = 255;

// The following are special MUD specific protocols.

// MNES: Mud New-Environ standard
pub const MNES: u8 = 39;

// MUD eXtension Protocol
// NOTE: Disabled due to too many issues with it.
pub const MXP: u8 = 91;

// Mud Server Status Protocol
pub const MSSP: u8 = 70;

// Compression
// pub const MCCP1: u8 = 85 - this is deprecrated
// NOTE: MCCP2 and MCCP3 is currently disabled.
pub const MCCP2: u8 = 86;
pub const MCCP3: u8 = 87;

// GMCP - Generic Mud Communication Protocol
pub const GMCP: u8 = 201;

// MSDP - Mud Server Data Protocol
pub const MSDP: u8 = 69;

// MTTS - Terminal Type
pub const MTTS: u8 = 24;
