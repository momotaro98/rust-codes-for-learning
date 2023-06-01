extern crate pnet;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;

#[macro_use]
extern crate log;

use std::env;

mod packets;
use packets::GettableEndPoints;


fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();    
    if args.len() != 2 {
        error!("args num should be 2");
        std::process::exit(1);
    }
    let interface_name = &args[1];

    // choose interface
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == *interface_name)
        .expect("failed to get interface");

    // [1]: get the channel(like socket of tcp) of data link
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("unhandled channel type"),
        Err(_e) => panic!("failed to create data link channel"),
    };

    loop {
        match rx.next() {
            Ok(frame) => {
                // build Ethernet frame from received data
                let frame = EthernetPacket::new(frame).unwrap();
                match frame.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        ipv4_handler(&frame);
                    },
                    EtherTypes::Ipv6 => {
                        ipv6_handler(&frame);
                    }
                    _ => {
                        info!("Not an IPv4 or IPv6 packet");
                    },
                }
            },
            Err(e) => {
                error!("failed to read: {}", e);
            },
        }
    }
}

/**
 * Build an IPv4 packet and then call the next layer's handler
 */
fn ipv4_handler(ethernet: &EthernetPacket) {
    if let Some(packet) = Ipv4Packet::new(ethernet.payload()) {
        match packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            },
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            },
            _ => {
                info!("not a TCP or UDP packet");
            },
        }
    }
}

/**
 * Build an IPv6 packet and then call the next layer's handler
 */
fn ipv6_handler(ethernet: &EthernetPacket) {
    if let Some(packet) = Ipv6Packet::new(ethernet.payload()) {
        match packet.get_next_header() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            },
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            },
            _ => {
                info!("not a TCP or UDP packet");
            },
        }
    }
}

/**
 * Build TCP packet
 */
fn tcp_handler(packet: &dyn GettableEndPoints) {
    let tcp = TcpPacket::new(packet.get_payload());
    if let Some(tcp) = tcp {
        print_packet_info(packet, &tcp, "TCP");
    }
}

/**
 * Build UDP packet
 */
fn udp_handler(packet: &dyn GettableEndPoints) {
    let udp = UdpPacket::new(packet.get_payload());
    if let Some(udp) = udp {
        print_packet_info(packet, &udp, "UDP");
    }
}

const WIDTH:usize = 20;

/**
 * Show data of application layer with binary
 */
fn print_packet_info(
    l3: &dyn GettableEndPoints,
    l4: &dyn GettableEndPoints,
    proto: &str,
) {
    println!(
        "captured a {} packet from {}|{} to {}|{}\n",
        proto,
        l3.get_source(),
        l4.get_source(),
        l3.get_destination(),
        l4.get_destination(),
    );
    let payload = l4.get_payload();
    let len = payload.len();

    // Show payload
    // Do the showing with the specified const width
    for i in 0..len {
        print!("{:<02X} ", payload[i]); // print normally
        if i%WIDTH == WIDTH-1 || i == len-1 {
            // do something like padding
            for _j in 0..WIDTH-1-(i % (WIDTH)) {
                print!("   ");
            }
            print!("| ");
            for j in i-i%WIDTH..i+1 {
                if payload[j].is_ascii_alphabetic() {
                    print!("{}", payload[j] as char);
                } else {
                    print!(".");
                }
            }
            print!("\n");
        }
    }
}