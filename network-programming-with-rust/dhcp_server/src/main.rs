use byteorder::{BigEndian, ByteOrder};
use env_logger;
use failure;
use log::{debug, error, info};
use pnet::util::MacAddr;
use std::env;
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::thread;
#[macro_use]
extern crate log;

use dhcp::DhcpPacket;

mod dhcp;

const HTYPE_ETHER: u8 = 1;

// オプションの数を考慮して決定する。
// 大きめにとっても問題はない。
const DHCP_SIZE: usize = 400;

enum Code {
    MessageType = 53,
    IPAddressLeaseTime = 51,
    ServerIdentifier = 54,
    RequestedIpAddress = 50,
    SubnetMask = 1,
    Router = 3,
    DNS = 6,
    End = 255,
}

const DHCPDISCOVER: u8 = 1;
const DHCPOFFER: u8 = 2;
const DHCPREQUEST: u8 = 3;
const DHCPACK: u8 = 5;
const DHCPNAK: u8 = 6;
const DHCPRELEASE: u8 = 7;

const BOOTREQUEST: u8 = 1;
const BOOTREPLY: u8 = 2;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let server_socket = UdpSocket::bind("0.0.0.0:67").expect("Failed to bind socket");

    // [note] Udpソケットを(IPレイヤーでの)ブロードキャストに設定する。
    // → クライアントはIPアドレスが設定されていないので必然的にブロードキャストになる。
    // ref from Rust std library
    // > Sets the value of the `SO_BROADCAST` option for this socket.
    // > When enabled, this socket is allowed to send packets to a broadcast address.
    server_socket.set_broadcast(true).unwrap();

    // ヒープ上にDhcpServer構造体を確保し、複数のスレッドから共有するためArcを利用している。
    let dhcp_server = Arc::new(
        DhcpServer::new().unwrap_or_else(|e| panic!("Failed to start dhcp server. {:?}", e)),
    );

    loop {
        let mut recv_buf = [0u8; 1024];
        // [note] ここの socket.recv_from は libc::recvfrom のシステムコールを同期(待つ)でする。
        match server_socket.recv_from(&mut recv_buf) {
            Ok((size, src)) => {
                debug!("received data from {}, size: {}", src, size);
                let transmission_socket = server_socket
                    .try_clone()
                    .expect("Failed to create client socket");
                
                // [note] サーバオブジェクトをCloneして別スレッドで動かす。
                // これによって非同期処理を実現させる。
                let cloned_dhcp_server = dhcp_server.clone();

                thread::spawn(move || {
                    if let Some(dhcp_packet) = DhcpPacket::new(recv_buf[..size].to_vec()) {
                        if dhcp_packet.get_op() != BOOTREQUEST {
                            // クライアントからのリクエストでなければ無視
                            return;
                        }
                        if let Err(e) =
                            dhcp_handler(&dhcp_packet, &transmission_socket, cloned_dhcp_server)
                        {
                            error!("{}", e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Could not recieve a datagram: {}", e);
            }
        }
    }
}

/**
 * DHCPのパケットを作成して返す。
 */
fn make_dhcp_packet(
    received_packet: &DhcpPacket,
    dhcp_server: &Arc<DhcpServer>,
    message_type: u8,
    ip_to_be_leased: Ipv4Addr,
) -> Result<DhcpPacket, failure::Error> {
    // パケットの本体となるバッファ。ヒープに確保する。
    let buffer = vec![0u8; DHCP_SIZE];
    let mut dhcp_packet = DhcpPacket::new(buffer).unwrap();

    // 各種フィールドの設定
    dhcp_packet.set_op(BOOTREPLY);
    dhcp_packet.set_htype(HTYPE_ETHER);
    dhcp_packet.set_hlen(6); //MACアドレスのオクテット長
    dhcp_packet.set_xid(received_packet.get_xid());
    if message_type == DHCPACK {
        dhcp_packet.set_ciaddr(received_packet.get_ciaddr());
    }
    dhcp_packet.set_yiaddr(ip_to_be_leased);
    dhcp_packet.set_flags(received_packet.get_flags());
    dhcp_packet.set_giaddr(received_packet.get_giaddr());
    dhcp_packet.set_chaddr(received_packet.get_chaddr());

    // 各種オプションの設定
    let mut cursor = dhcp::OPTIONS;
    dhcp_packet.set_magic_cookie(&mut cursor);
    dhcp_packet.set_option(
        &mut cursor,
        Code::MessageType as u8,
        1,
        Some(&[message_type]),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::IPAddressLeaseTime as u8,
        4,
        Some(&dhcp_server.lease_time),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::ServerIdentifier as u8,
        4,
        Some(&dhcp_server.server_address.octets()),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::SubnetMask as u8,
        4,
        Some(&dhcp_server.subnet_mask.octets()),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::Router as u8,
        4,
        Some(&dhcp_server.default_gateway.octets()),
    );
    dhcp_packet.set_option(
        &mut cursor,
        Code::DNS as u8,
        4,
        Some(&dhcp_server.dns_server.octets()),
    );

    dhcp_packet.set_option(&mut cursor, Code::End as u8, 0, None);
    Ok(dhcp_packet)
}