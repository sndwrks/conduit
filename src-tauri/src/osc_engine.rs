use crate::router::{IncomingMessage, OscArgValue, OutputAction};
use log::{error, info, warn};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;

pub async fn start_udp_listener(
    port: u16,
    tx: mpsc::UnboundedSender<IncomingMessage>,
    token: CancellationToken,
) -> Result<(), String> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    let socket = UdpSocket::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind UDP socket on port {}: {}", port, e))?;

    tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((size, _src)) => {
                            if let Some(msg) = decode_osc_udp(&buf[..size]) {
                                let _ = tx.send(msg);
                            }
                        }
                        Err(e) => {
                            error!("UDP recv error: {}", e);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn start_tcp_listener(
    port: u16,
    tx: mpsc::UnboundedSender<IncomingMessage>,
    token: CancellationToken,
) -> Result<(), String> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind TCP listener on port {}: {}", port, e))?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("OSC TCP client connected: {}", addr);
                            let tx = tx.clone();
                            let token = token.clone();
                            tokio::spawn(handle_tcp_client(stream, addr, tx, token));
                        }
                        Err(e) => {
                            error!("TCP accept error: {}", e);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

// SLIP framing constants (RFC 1055)
const SLIP_END: u8 = 0xC0;
const SLIP_ESC: u8 = 0xDB;
const SLIP_ESC_END: u8 = 0xDC;
const SLIP_ESC_ESC: u8 = 0xDD;
const SLIP_MAX_PACKET: usize = 65536;

async fn handle_tcp_client(
    mut stream: TcpStream,
    peer: SocketAddr,
    tx: mpsc::UnboundedSender<IncomingMessage>,
    token: CancellationToken,
) {
    let mut read_buf = [0u8; 4096];
    let mut packet_buf: Vec<u8> = Vec::new();
    let mut in_escape = false;

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            result = stream.read(&mut read_buf) => {
                match result {
                    Ok(0) => break, // connection closed
                    Ok(n) => {
                        for &byte in &read_buf[..n] {
                            if in_escape {
                                in_escape = false;
                                match byte {
                                    SLIP_ESC_END => packet_buf.push(SLIP_END),
                                    SLIP_ESC_ESC => packet_buf.push(SLIP_ESC),
                                    _ => {
                                        warn!("OSC TCP SLIP: invalid escape byte 0x{:02X} from {}", byte, peer);
                                        packet_buf.clear();
                                    }
                                }
                            } else {
                                match byte {
                                    SLIP_END => {
                                        if !packet_buf.is_empty() {
                                            if let Some(msg) = decode_osc_udp(&packet_buf) {
                                                let _ = tx.send(msg);
                                            }
                                            packet_buf.clear();
                                        }
                                    }
                                    SLIP_ESC => {
                                        in_escape = true;
                                    }
                                    _ => {
                                        packet_buf.push(byte);
                                        if packet_buf.len() > SLIP_MAX_PACKET {
                                            warn!("OSC TCP SLIP: oversized packet (>{} bytes) from {}, dropping", SLIP_MAX_PACKET, peer);
                                            packet_buf.clear();
                                            in_escape = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("OSC TCP read error from {}: {}", peer, e);
                        break;
                    }
                }
            }
        }
    }
    info!("OSC TCP client disconnected: {}", peer);
}

fn decode_osc_udp(data: &[u8]) -> Option<IncomingMessage> {
    match rosc::decoder::decode_udp(data) {
        Ok((_rest, packet)) => decode_osc_packet(packet),
        Err(e) => {
            warn!("OSC decode error: {:?}", e);
            None
        }
    }
}

fn decode_osc_packet(packet: OscPacket) -> Option<IncomingMessage> {
    match packet {
        OscPacket::Message(msg) => {
            let args: Vec<OscArgValue> = msg
                .args
                .into_iter()
                .filter_map(|a| match a {
                    OscType::Int(i) => Some(OscArgValue::Int(i)),
                    OscType::Float(f) => Some(OscArgValue::Float(f)),
                    OscType::String(s) => Some(OscArgValue::String(s)),
                    _ => None,
                })
                .collect();
            Some(IncomingMessage::Osc {
                address: msg.addr,
                args,
            })
        }
        OscPacket::Bundle(_) => None, // Post-MVP
    }
}

pub async fn send_osc_udp(
    host: &str,
    port: u16,
    address: &str,
    args: &[OscArgValue],
) -> Result<(), String> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid send address: {}", e))?;
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to create UDP socket: {}", e))?;

    let msg = build_osc_message(address, args);
    let data = rosc::encoder::encode(&OscPacket::Message(msg))
        .map_err(|e| format!("OSC encode error: {:?}", e))?;

    socket
        .send_to(&data, addr)
        .await
        .map_err(|e| format!("UDP send error: {}", e))?;
    Ok(())
}

fn slip_encode(data: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(data.len() + 2);
    encoded.push(SLIP_END);
    for &byte in data {
        match byte {
            SLIP_END => {
                encoded.push(SLIP_ESC);
                encoded.push(SLIP_ESC_END);
            }
            SLIP_ESC => {
                encoded.push(SLIP_ESC);
                encoded.push(SLIP_ESC_ESC);
            }
            _ => encoded.push(byte),
        }
    }
    encoded.push(SLIP_END);
    encoded
}

pub async fn send_osc_tcp(
    host: &str,
    port: u16,
    address: &str,
    args: &[OscArgValue],
    timeout_ms: u64,
) -> Result<(), String> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid send address: {}", e))?;

    let msg = build_osc_message(address, args);
    let data = rosc::encoder::encode(&OscPacket::Message(msg))
        .map_err(|e| format!("OSC encode error: {:?}", e))?;

    let slip_data = slip_encode(&data);

    let result = timeout(Duration::from_millis(timeout_ms), async {
        let mut stream = TcpStream::connect(addr).await?;
        stream.write_all(&slip_data).await?;
        Ok::<(), std::io::Error>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("TCP send error: {}", e)),
        Err(_) => Err(format!("TCP send timeout ({}ms) to {}:{}", timeout_ms, host, port)),
    }
}

fn build_osc_message(address: &str, args: &[OscArgValue]) -> OscMessage {
    let osc_args: Vec<OscType> = args
        .iter()
        .map(|a| match a {
            OscArgValue::Int(i) => OscType::Int(*i),
            OscArgValue::Float(f) => OscType::Float(*f),
            OscArgValue::String(s) => OscType::String(s.clone()),
        })
        .collect();
    OscMessage {
        addr: address.to_string(),
        args: osc_args,
    }
}

pub fn dispatch_output(
    action: &OutputAction,
    settings: &std::sync::Arc<std::sync::Mutex<crate::models::Settings>>,
    midi_out: &Option<std::sync::Arc<std::sync::Mutex<midir::MidiOutputConnection>>>,
    rt: &tokio::runtime::Handle,
) {
    match action {
        OutputAction::Midi {
            message_type,
            channel,
            note_or_cc,
            value,
        } => {
            if let Some(conn) = midi_out {
                let status = match message_type {
                    crate::models::MidiMessageType::NoteOn => 0x90 + (channel - 1),
                    crate::models::MidiMessageType::NoteOff => 0x80 + (channel - 1),
                    crate::models::MidiMessageType::Cc => 0xB0 + (channel - 1),
                    crate::models::MidiMessageType::ProgramChange => 0xC0 + (channel - 1),
                };
                if let Ok(mut conn) = conn.lock() {
                    if matches!(message_type, crate::models::MidiMessageType::ProgramChange) {
                        let _ = conn.send(&[status, *note_or_cc]);
                    } else {
                        let _ = conn.send(&[status, *note_or_cc, *value]);
                    }
                }
            }
        }
        OutputAction::Osc { address, args } => {
            let (host, port, protocol, timeout_ms) = {
                let s = match settings.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Settings mutex poisoned in dispatch_output(): {}", e);
                        return;
                    }
                };
                (
                    s.osc_send_host.clone(),
                    s.osc_send_port,
                    s.osc_send_protocol.clone(),
                    s.osc_tcp_send_timeout_ms,
                )
            };
            let address = address.clone();
            let args = args.clone();
            rt.spawn(async move {
                let result = match protocol {
                    crate::models::OscSendProtocol::Udp => {
                        send_osc_udp(&host, port, &address, &args).await
                    }
                    crate::models::OscSendProtocol::Tcp => {
                        send_osc_tcp(&host, port, &address, &args, timeout_ms).await
                    }
                };
                if let Err(e) = result {
                    error!("OSC send error: {}", e);
                }
            });
        }
    }
}
