use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Mutex, RwLock};

use ipnetwork::Ipv4Network;
use pnet::packet::PrimitiveValues;
use pnet::util::MacAddr;
use rusqlite::Connection;

// [note] 以降のconstはRFC2131で記載の以下図でのDHCPパケットの構成である。
// https://datatracker.ietf.org/doc/html/rfc2131#autoid-8

/*
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |     op (1)    |   htype (1)   |   hlen (1)    |   hops (1)    |
   +---------------+---------------+---------------+---------------+
   |                            xid (4)                            |
   +-------------------------------+-------------------------------+
   |           secs (2)            |           flags (2)           |
   +-------------------------------+-------------------------------+
   |                          ciaddr  (4)                          |
   +---------------------------------------------------------------+
   |                          yiaddr  (4)                          |
   +---------------------------------------------------------------+
   |                          siaddr  (4)                          |
   +---------------------------------------------------------------+
   |                          giaddr  (4)                          |
   +---------------------------------------------------------------+
   |                                                               |
   |                          chaddr  (16)                         |
   |                                                               |
   |                                                               |
   +---------------------------------------------------------------+
   |                                                               |
   |                          sname   (64)                         |
   +---------------------------------------------------------------+
   |                                                               |
   |                          file    (128)                        |
   +---------------------------------------------------------------+
   |                                                               |
   |                          options (variable)                   |
   +---------------------------------------------------------------+

                  Figure 1:  Format of a DHCP message
 */
const OP: usize = 0;
const HTYPE: usize = 1;
const HLEN: usize = 2;
/* const HOPS: usize = 3; 今回は利用しない */
const XID: usize = 4;
const SECS: usize = 8;
const FLAGS: usize = 10;
const CIADDR: usize = 12;
const YIADDR: usize = 16;
const SIADDR: usize = 20;
const GIADDR: usize = 24;
const CHADDR: usize = 28;
const SNAME: usize = 44;
/* const FILE: usize = 108; 今回は利用しない */
pub const OPTIONS: usize = 236;

const DHCP_MINIMUM_SIZE: usize = 237;
const OPTION_END: u8 = 255;

use super::database;
use super::util;

/**
 * DHCPのパケットを表現する。
 */
pub struct DhcpPacket {
    buffer: Vec<u8>,
}

impl DhcpPacket {
    pub fn new(buf: Vec<u8>) -> Option<DhcpPacket> {
        if buf.len() > DHCP_MINIMUM_SIZE {
            let packet = DhcpPacket { buffer: buf };
            return Some(packet);
        }
        None
    }

    pub fn get_buffer(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    pub fn get_op(&self) -> u8 {
        self.buffer[OP]
    }

    pub fn get_options(&self) -> &[u8] {
        &self.buffer[OPTIONS..]
    }

    pub fn get_xid(&self) -> &[u8] {
        &self.buffer[XID..SECS]
    }

    pub fn get_flags(&self) -> &[u8] {
        &self.buffer[FLAGS..CIADDR]
    }

    pub fn get_giaddr(&self) -> Ipv4Addr {
        let b = &self.buffer[GIADDR..CHADDR];
        Ipv4Addr::new(b[0], b[1], b[2], b[3])
    }

    pub fn get_chaddr(&self) -> MacAddr {
        let b = &self.buffer[CHADDR..SNAME];
        // [note] ↓ 本来は16オクテット分だが書籍の実装ではMACアドレスしか使わないから6オクテットだけにしている。
        MacAddr::new(b[0], b[1], b[2], b[3], b[4], b[5])
    }

    pub fn get_ciaddr(&self) -> Ipv4Addr {
        let b = &self.buffer[CIADDR..YIADDR];
        Ipv4Addr::new(b[0], b[1], b[2], b[3])
    }

    pub fn set_op(&mut self, op: u8) {
        self.buffer[OP] = op;
    }

    pub fn set_htype(&mut self, htype: u8) {
        self.buffer[HTYPE] = htype;
    }

    pub fn set_hlen(&mut self, hlen: u8) {
        self.buffer[HLEN] = hlen;
    }

    pub fn set_xid(&mut self, xid: &[u8]) {
        self.buffer[XID..SECS].copy_from_slice(xid);
    }

    pub fn set_ciaddr(&mut self, ciaddr: Ipv4Addr) {
        self.buffer[CIADDR..YIADDR].copy_from_slice(&ciaddr.octets());
    }

    pub fn set_yiaddr(&mut self, yiaddr: Ipv4Addr) {
        self.buffer[YIADDR..SIADDR].copy_from_slice(&yiaddr.octets());
    }

    pub fn set_flags(&mut self, flags: &[u8]) {
        self.buffer[FLAGS..CIADDR].copy_from_slice(flags);
    }

    pub fn set_giaddr(&mut self, giaddr: Ipv4Addr) {
        self.buffer[GIADDR..CHADDR].copy_from_slice(&giaddr.octets());
    }

    pub fn set_chaddr(&mut self, chaddr: MacAddr) {
        let t = chaddr.to_primitive_values();
        let macaddr_value = [t.0, t.1, t.2, t.3, t.4, t.5];
        // [著者コメント] ここだけCHADDR..SNAMEでないのは、chaddrフィールドが16オクテット確保されているため。
        // 今回はMACアドレスしかこのフィールドに入らないので、MACアドレスのサイズである6オクテット確保している。
        //
        // [note] ↓ 本来は16オクテット分だがMACアドレスしか使わないから6オクテットだけにしている。
        self.buffer[CHADDR..CHADDR + 6].copy_from_slice(&macaddr_value);
    }

    pub fn set_magic_cookie(&mut self, cursor: &mut usize) {
        self.buffer[*cursor..*cursor + 4].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
        *cursor += 4;
    }

    /**
     * DHCPパケットのoptionフィールドをセットする。
     * optionフィールドは複数のコードが存在する。
     * [note] DHCPパケットにおいてOptionは仕様が詰まっているが、可変長である。
     * 以下のように CodeとLenが先頭にありそれがそれぞれ1オクテットで固定あり、その後に可変の値が続く。
     * optionフィールドとしては複数の
     * 
        Code   Len         Address 1               Address 2
        +-----+-----+-----+-----+-----+-----+-----+-----+--
        |  6  |  n  |  a1 |  a2 |  a3 |  a4 |  a1 |  a2 |  ...
        +-----+-----+-----+-----+-----+-----+-----+-----+--
     */
    pub fn set_option(
        &mut self,
        cursor: &mut usize,
        code: u8,
        len: usize,
        contents: Option<&[u8]>,
    ) {
        // goal 概要
        // self.buffer[0] = code;// codeを入れる
        // self.buffer[1] = len as u8;// lenを入れる
        // self.buffer[2..len] = contents; //コンテンツを入れる。

        // Codeを埋める。
        self.buffer[*cursor] = code;
        *cursor += 1;
        // Optionのタイプによってコンテンツがあったりなかったりする。
        if code == OPTION_END {
            return;
        }

        // Lenを埋める。
        self.buffer[*cursor] = len as u8;
        *cursor += 1;

        // コンテンツを埋める。
        if let Some(contents) = contents {
            self.buffer[*cursor..*cursor+contents.len()].copy_from_slice(contents);
        }
        *cursor += len;
    }

    pub fn get_option(&self, code: u8) -> Option<Vec<u8>> {
        // [やろうとしていること]
        // パケットの中のcode の位置を探り、そのコンテンツを返す。
        let options = self.get_options();

        let mut index: usize = 4; // 最初の4バイトはクッキーなので飛ばす

        while options[index] != OPTION_END { // 走査の終了条件はindexがENDにたどり着いたとき
            if options[index] == code {
                // 目的のcodeを見つけたとき
                let len: u8 = options[index+1];
                let buf_index = index + 2;
                let option = options[buf_index..buf_index+(len as usize)].to_vec();
                return Some(option);
            } else if options[index] == 0 {
                // 0 は仕様上Paddingという埋める用のものなのでindexをインクリメントしてloopに戻る。
                index += 1;
            } else {
                // 目的ではないcodeを見つけたとき → indexを次のcodeがある位置まで引き上げる
                let len = options[index+1] as usize;
                index += 2 + len; // 2 はlenへの移動とcodeへの移動の2オクテット分
            }
        }
        None
    }

}

/**
 * DHCPサーバの情報を保持する。
 * 複数のスレッドで共有されるため、フィールドにmutアクセスする際はロックを取得する必要がある。
 * 読み出しだけならフィールドにロックは必要ない。
 */
pub struct DhcpServer {
    address_pool: RwLock<Vec<Ipv4Addr>>,  // 利用(割当)可能なアドレス。[note] DHCPサーバにおいて一番のメインのフィールド。
    pub db_connection: Mutex<Connection>, // データベースのコネクション。ConnectionはSyncを実装しないのでRwLockではだめ。
    pub network_addr: Ipv4Network,
    pub server_address: Ipv4Addr,
    pub default_gateway: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub dns_server: Ipv4Addr,
    pub lease_time: Vec<u8>,
}

impl DhcpServer {
    pub fn new() -> Result<DhcpServer, failure::Error> {
        let env = util::load_env();

        // DNSやゲートウェイなどのアドレス
        let static_addresses = util::obtain_static_addresses(&env)?;

        let network_addr_with_prefix: Ipv4Network = Ipv4Network::new(
            static_addresses["network_addr"],
            ipnetwork::ipv4_mask_to_prefix(static_addresses["subnet_mask"])?,
        )?;

        let con = Connection::open("dhcp.db")?;

        let addr_pool = Self::init_address_pool(&con, &static_addresses, network_addr_with_prefix)?;
        info!(
            "There are {} addresses in the address pool",
            addr_pool.len()
        );

        // [note] IPアドレスやサブネットマスクはRustの標準ライブラリ提供のIPアドレス型が裏でビッグエンディアンでバイト化してくれるが、
        // lease_time は普通に整数値なので自分(このプログラム)で u32型 をビッグエンディアンでバイト化する必要がある。
        let lease_time = util::make_big_endian_vec_from_u32(
            env.get("LEASE_TIME").expect("Missing lease_time").parse()?,
        )?;

        Ok(DhcpServer {
            address_pool: RwLock::new(addr_pool),
            db_connection: Mutex::new(con),
            network_addr: network_addr_with_prefix,
            server_address: static_addresses["dhcp_server_addr"],
            default_gateway: static_addresses["default_gateway"],
            subnet_mask: static_addresses["subnet_mask"],
            dns_server: static_addresses["dns_addr"],
            lease_time,
        })
    }

    /**
     * 新たなホストに割り当て可能なアドレスプールを初期化
     */
    fn init_address_pool(
        con: &Connection,
        static_addresses: &HashMap<String, Ipv4Addr>,
        network_addr_with_prefix: Ipv4Network,
    ) -> Result<Vec<Ipv4Addr>, failure::Error> {
        let network_addr = static_addresses.get("network_addr").unwrap();
        let default_gateway = static_addresses.get("default_gateway").unwrap();
        let dhcp_server_addr = static_addresses.get("dhcp_server_addr").unwrap();
        let dns_server_addr = static_addresses.get("dns_addr").unwrap();
        let broadcast = network_addr_with_prefix.broadcast();

        // すでに使用されていて、解放もされていないIPアドレス
        let mut used_ip_addrs = database::select_addresses(con, Some(0))?;

        used_ip_addrs.push(*network_addr);
        used_ip_addrs.push(*default_gateway);
        used_ip_addrs.push(*dhcp_server_addr);
        used_ip_addrs.push(*dns_server_addr);
        used_ip_addrs.push(broadcast);

        // ネットワークの全てのIPアドレスから、使用されているIPアドレスを除いたものを
        // アドレスプールとする。
        let mut addr_pool: Vec<Ipv4Addr> = network_addr_with_prefix
            .iter()
            .filter(|addr| !used_ip_addrs.contains(addr))
            .collect();

        // 気持ち的にIPアドレスの若い方から割り当てたいので、逆順にする。
        // 取り出すときは末尾からpop()を行う。
        addr_pool.reverse();

        Ok(addr_pool)
    }

    /*
    * 
    * 以降はメインであるアドレスプールの操作。
    * 
    */ 

    /**
     * アドレスプールからIPアドレスを引き抜く(割当するためのIPアドレス)
     */
    pub fn pick_available_ip(&self) -> Option<Ipv4Addr> {
        let mut lock = self.address_pool.write().unwrap();
        // [note] address_pool.write() によって pthread_rwlock_wrlock というOSでのLockを取るシステムコールが呼び出される。
        // Rustの仕組みによって上記で得られている `lock` 変数がスコープから抜けるとLockがUnlockされる。

        // コストを考えてベクタの末尾から取り出す。
        lock.pop()
    }

    /**
     * アドレスプールから指定のIPアドレスを引き抜く
     */
    pub fn pick_specified_ip(&self, requested_ip: Ipv4Addr) -> Option<Ipv4Addr> {
        let mut lock = self.address_pool.write().unwrap();
        for i in 0..lock.len() {
            if lock[i] == requested_ip {
                return Some(lock.remove(i));
            }
        }
        None
    }

    /**
     * アドレスプールの先頭にIPアドレスを返す。
     * 取り出しは後方から行われるため、返されたアドレスは当分他のホストに割り当てられない
     */
    pub fn release_address(&self, released_ip: Ipv4Addr) {
        let mut lock = self.address_pool.write().unwrap();
        lock.insert(0, released_ip);
    }
}