use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags};
use pnet::transport::{
    self, TransportChannelType, TransportProtocol, TransportReceiver, TransportSender,
};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use std::{env, fs, process, thread};
#[macro_use]
extern crate log;

const TCP_SIZE: usize = 20;

#[derive(Clone, Copy)]
enum ScanType {
    Syn= tcp::TcpFlags::SYN as isize,
    Fin= tcp::TcpFlags::FIN as isize,
    Xmas= (tcp::TcpFlags::FIN | tcp::TcpFlags::URG | tcp::TcpFlags::PSH ) as isize,
    Null= 0
}

struct PacketInfo {
    my_ipaddr: Ipv4Addr,
    target_ipaddr: Ipv4Addr,
    my_port: u16,
    maximum_port: u16,
    scan_type: ScanType,
}

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Bad nunber of arguemnts. [ipaddr] [scantype]");
        std::process::exit(1);
    }

    let packet_info = {
        let contents = fs::read_to_string(".env").expect("failed to read env file");
        let lines: Vec<_> = contents.split('\n').collect();
        let mut map = HashMap::new();
        for line in lines {
            let elm: Vec<_> = line.split('=').collect();
            if elm.len() == 2 {
                map.insert(elm[0], elm[1]);
            }
        }
        PacketInfo {
            my_ipaddr: map["MY_IPADDR"].parse().expect("invalid ipaddr"),
            target_ipaddr: args[1].parse().expect("invalid target ipaddr"),
            my_port: map["MY_PORT"].parse().expect("invalid port number"),
            maximum_port: map["MAXIMUM_PORT_NUM"]
                .parse()
                .expect("invalid maximum port num"),
            scan_type: match args[2].as_str() {
                "sS" => ScanType::Syn,
                "sF" => ScanType::Fin,
                "sX" => ScanType::Xmas,
                "sN" => ScanType::Null,
                _ => {
                    error!("Undefined scan method, only accept [sS|sF|sN|sX].");
                    process::exit(1);
                }
            },
        }
    };

    // Open channel of Transport layer
    // 内部的にはソケット
    /*
    【My Note】
    transport::transport_channel の内部では以下の実行によって socketシステムコールが呼ばれてソケットが生成されている。

    pnet_sys::socket(pnet_sys::AF_INET, pnet_sys::SOCK_RAW, proto as libc::c_int)

     */
    let (mut ts, mut tr) = transport::transport_channel(
        1024,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
    )
    .expect("Failed to open channel.");
    
    // Packet sending and receiving with parallel
    rayon::join(
        || send_packet(&mut ts, &packet_info),
        || receive_packets(&mut tr, &packet_info),
    );

}

/**
 * Send packets to specified range
 */
fn send_packet(
    ts: &mut TransportSender,
    packet_info: &PacketInfo,
) -> Result<(), failure::Error> {
    // パケット作成 (// TCPの本来の仕様をハックする。)
    let mut packet = build_packet(packet_info);
    for i in 1..=packet_info.maximum_port {
        let mut tcp_header =
            MutableTcpPacket::new(&mut packet).ok_or_else(|| failure::err_msg("invalid packet"))?;

        // TCPの本来の仕様をハックする。
        reregister_destination_port(i, &mut tcp_header, packet_info);

        // 送信する
        thread::sleep(Duration::from_millis(5));
        ts.send_to(tcp_header, IpAddr::V4(packet_info.target_ipaddr))?;
    }
    Ok(())
}

// Generate a packet (TCPヘッダ)
fn build_packet(packet_info: &PacketInfo) -> [u8; TCP_SIZE] {
    let mut tcp_buffer = [0u8; TCP_SIZE];

    // pnet のこの MutableTcpPacket が細かいTCP仕様用の詰める作業や
    // CPUアーキテクチャとTCP仕様でのリトルエンディアン・ビッグエンディアンの違いの吸収処理をやってくれている。
    let mut tcp_header = MutableTcpPacket::new(&mut tcp_buffer[..]).unwrap();
    tcp_header.set_source(packet_info.my_port);

    // オプションを含まないので、20オクテットまでがTCPヘッダ。
    // 4オクテット単位で指定する
    tcp_header.set_data_offset(5);
    tcp_header.set_flags(packet_info.scan_type as u16);

    // カスタマイズしたヘッダー情報からチェックサムを生成する。
    // チェックサムが正しくないと送信先で無効とみなされてしまう。
    let checksum = tcp::ipv4_checksum(
        &tcp_header.to_immutable(),
        &packet_info.my_ipaddr,
        &packet_info.target_ipaddr,
    );
    tcp_header.set_checksum(checksum);

    tcp_buffer
}

/**
 * TCPヘッダの宛先ポート情報を書き換える。
 * チェックサムを計算し直す必要がある。
 */
fn reregister_destination_port(
    target: u16,
    tcp_header: &mut MutableTcpPacket,
    packet_info: &PacketInfo,
) {
    tcp_header.set_destination(target);
    let checksum = tcp::ipv4_checksum(
        &tcp_header.to_immutable(),
        &packet_info.my_ipaddr,
        &packet_info.target_ipaddr,
    );
    tcp_header.set_checksum(checksum);
}

/**
 * パケットを受信してスキャン結果を出力する。
 * 
 * TCPパケットがそのまま取得できるので、そのコントロールフラグを読み取ります。
 * SYNスキャンの場合だとSYN|ACKパケットが返ってきた時、それ以外のスキャン手法の場合だと
 * 何も返ってこない時にターゲットのポートは開いていると判断します。
 */
fn receive_packets(
    tr: &mut TransportReceiver,
    packet_info: &PacketInfo,
) -> Result<(), failure::Error> {
    let mut reply_ports = Vec::new();
    let mut packet_iter = transport::tcp_packet_iter(tr);
    loop {
        println!("here1");
        // Returned packet from target
        let tcp_packet = match packet_iter.next() {
            Ok((tcp_packet, _)) => {
                if tcp_packet.get_destination() == packet_info.my_port {
                    tcp_packet
                } else {
                    continue;
                }
            },
            Err(_) => continue,
        };
        println!("here2");
        
        let target_port = tcp_packet.get_source();
        match packet_info.scan_type {
            ScanType::Syn => {
                if tcp_packet.get_flags() == TcpFlags::SYN | TcpFlags::ACK {
                    println!("port {} is open", target_port);
                }
            }
            // SYNスキャン以外はレスポンスが返ってきたポート（＝閉じているポート）を記録
            ScanType::Fin | ScanType::Xmas | ScanType::Null => {
                reply_ports.push(target_port);
            }
        }

        /* [1] 手抜き：スキャン対象の最後のポートに対する返信が帰ってこれば終了 */
        if target_port != packet_info.maximum_port {
            continue;
        }
        match packet_info.scan_type {
            ScanType::Fin | ScanType::Xmas | ScanType::Null => {
                for i in 1..=packet_info.maximum_port {
                    if reply_ports.iter().find(|&&x| x == i).is_none() {
                        println!("port {} is open", i);
                    }
                }
            }
            _ => {}
        }
        
        return Ok(());
    }
}