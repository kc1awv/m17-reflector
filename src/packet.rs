use crate::callsign::decode_callsign;
use crate::crc::crc16_m17;
use std::convert::TryInto;

#[derive(Debug)]
pub enum Packet {
    Control(ControlKind),
    Stream(StreamPacket),
    // PacketMode(PacketModePacket), // future
}

#[derive(Debug)]
pub enum ControlKind {
    Conn { from: String, module: char },
    Lstn { from: String, module: char },
    Ackn,
    Nack,
    Ping { from: String },
    Pong { from: String },
    Disc { from: String },
}

#[derive(Debug)]
pub struct StreamPacket {
    pub stream_id: u16,
    pub dst: String,
    pub src: String,
    pub frame_num: u16,
    pub last_frame: bool,
    pub payload: [u8; 16],
    pub crc_ok: bool,
}

#[derive(Debug)]
pub enum PacketError {
    InvalidLength,
    InvalidMagic,
    InvalidCrc,
}

pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 4 {
        return Err(PacketError::InvalidLength);
    }

    match &data[0..4] {
        b"CONN" => parse_conn(data),
        b"LSTN" => parse_lstn(data),
        b"ACKN" => Ok(Packet::Control(ControlKind::Ackn)),
        b"NACK" => Ok(Packet::Control(ControlKind::Nack)),
        b"PING" => parse_ping(data),
        b"PONG" => parse_pong(data),
        b"DISC" => parse_disc(data),
        b"M17 " => parse_stream(data),
        b"M17P" => Err(PacketError::InvalidMagic), // TODO: implement packet mode
        _ => Err(PacketError::InvalidMagic),
    }
}

fn parse_conn(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 11 {
        return Err(PacketError::InvalidLength);
    }
    let from = decode_callsign(data[4..10].try_into().unwrap());
    let module = data[10] as char;
    Ok(Packet::Control(ControlKind::Conn { from, module }))
}

fn parse_lstn(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 11 {
        return Err(PacketError::InvalidLength);
    }
    let from = decode_callsign(data[4..10].try_into().unwrap());
    let module = data[10] as char;
    Ok(Packet::Control(ControlKind::Lstn { from, module }))
}

fn parse_ping(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 10 {
        return Err(PacketError::InvalidLength);
    }
    let from = decode_callsign(data[4..10].try_into().unwrap());
    Ok(Packet::Control(ControlKind::Ping { from }))
}

fn parse_pong(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 10 {
        return Err(PacketError::InvalidLength);
    }
    let from = decode_callsign(data[4..10].try_into().unwrap());
    Ok(Packet::Control(ControlKind::Pong { from }))
}

fn parse_disc(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < 10 {
        return Err(PacketError::InvalidLength);
    }
    let from = decode_callsign(data[4..10].try_into().unwrap());
    Ok(Packet::Control(ControlKind::Disc { from }))
}

fn parse_stream(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() != 54 {
        return Err(PacketError::InvalidLength);
    }

    let crc_calc = crc16_m17(&data[..52]);
    let crc_field = u16::from_be_bytes(data[52..54].try_into().unwrap());
    let crc_ok = crc_calc == crc_field;

    if !crc_ok {
        log::debug!("CRC check failed for stream packet (continuing to route)");
    }

    let stream_id = u16::from_be_bytes(data[4..6].try_into().unwrap());

    let dst = decode_callsign(data[6..12].try_into().unwrap());
    let src = decode_callsign(data[12..18].try_into().unwrap());

    let frame_num_raw = u16::from_be_bytes(data[34..36].try_into().unwrap());
    let last_frame = (frame_num_raw & 0x8000) != 0;
    let frame_num = frame_num_raw & 0x7FFF;

    let payload: [u8; 16] = data[36..52].try_into().unwrap();

    Ok(Packet::Stream(StreamPacket {
        stream_id,
        dst,
        src,
        frame_num,
        last_frame,
        payload,
        crc_ok,
    }))
}
