use crate::packet::TCPPacket;
use crate::tcpflags;
use anyhow::{Context, Result, Ok};
use pnet::packet::{ip::IpNextHeaderProtocols, Packet};
use pnet::transport::{self, TransportChannelType, TransportProtocol, TransportSender};
use pnet::util;
use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::net::{IpAddr, Ipv4Addr};
use std::time::SystemTime;

const SOCKET_BUFFER_SIZE: usize = 4380;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct SockID(pub Ipv4Addr, pub Ipv4Addr, pub u16, pub u16);

// [note] ソケットは情報を持つ
pub struct Socket {
    pub local_addr: Ipv4Addr,
    pub remote_addr: Ipv4Addr,
    pub local_port: u16,
    pub remote_port: u16,

    pub send_param: SendParam,
    pub recv_param: RecvParam,
    pub status: TcpStatus,

    // Section 3.7.4 確認応答と再送
    pub retransmission_queue: VecDeque<RetransmissionQueueEntry>,

    // Section 3.6 for Passive Open
    pub connected_connection_queue: VecDeque<SockID>, // 接続済みソケットを保持するキュー．リスニングソケットのみ使用
    pub listening_socket: Option<SockID>, // 生成元のリスニングソケット．接続済みソケットのみ使用

    pub sender: TransportSender, // 送信機構
}

/*
    [note] ソケットは分割されたTCPパケットの順番保証の機能のための情報を保持しておく必要がある。
    以下がそのTCPでの仕様になる。

    RFC section URL: https://datatracker.ietf.org/doc/html/rfc793#section-3.2

    Send Sequence Variables (送信においてソケットが保持するべきデータ)

      SND.UNA - send unacknowledged
      SND.NXT - send next
      SND.WND - send window
      SND.UP  - send urgent pointer
      SND.WL1 - segment sequence number used for last window update
      SND.WL2 - segment acknowledgment number used for last window
                update
      ISS     - initial send sequence number

    Receive Sequence Variables (受信においてソケットが保持するべきデータ)

      RCV.NXT - receive next
      RCV.WND - receive window
      RCV.UP  - receive urgent pointer
      IRS     - initial receive sequence number    


    [note] 以下の図は送信するべき全体データ(Payload)にて、上記データがそのとき示していたところにおいて、
    1,2,3,4それぞれの領域部分がどのような意味の状態であるかを示している。


  Send Sequence Space (送信)

                   1         2          3          4
              ----------|----------|----------|----------
                     SND.UNA    SND.NXT    SND.UNA
                                          +SND.WND

        1 - old sequence numbers which have been acknowledged
        2 - sequence numbers of unacknowledged data
        3 - sequence numbers allowed for new data transmission
        4 - future sequence numbers which are not yet allowed

                          Send Sequence Space

                               Figure 4.



  The send window is the portion of the sequence space labeled 3 in
  figure 4.

  Receive Sequence Space (受信)

                       1          2          3
                   ----------|----------|----------
                          RCV.NXT    RCV.NXT
                                    +RCV.WND

        1 - old sequence numbers which have been acknowledged
        2 - sequence numbers allowed for new reception
        3 - future sequence numbers which are not yet allowed

                         Receive Sequence Space

                               Figure 5.

*/

#[derive(Clone, Debug)]
pub struct SendParam {
    pub unacked_seq: u32, // 送信後まだACKされていないseqの先頭
    pub next: u32,        // 次の送信(予定)
    pub window: u16,      // 送信ウィンドウサイズ
    pub initial_seq: u32, // 初期送信seq
}

#[derive(Clone, Debug)]
pub struct RecvParam {
    pub next: u32,        // 次受信するseq
    pub window: u16,      // 受信ウィンドウ
    pub initial_seq: u32, // 初期受信seq
    pub tail: u32,        // 受信seqの最後尾
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TcpStatus {
    Listen,
    SynSent,
    SynRcvd,
    Established,
    FinWait1,
    FinWait2,
    TimeWait,
    CloseWait,
    LastAck,
}

/// 失敗時の再送用のセグメント(Packet)を保管するためのキュー。各ソケットが保持する。
#[derive(Clone, Debug)]
pub struct RetransmissionQueueEntry {
    pub packet: TCPPacket,                    // 再送用のセグメント
    pub latest_transmission_time: SystemTime, // タイムアウトを判定するための最後に送信された時刻
    pub transmission_count: u8,               // 送信回数
}

impl RetransmissionQueueEntry {
    fn new(packet: TCPPacket) -> Self {
        Self {
            packet,
            latest_transmission_time: SystemTime::now(),
            transmission_count: 1,
        }
    }
}

impl Display for TcpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TcpStatus::Listen => write!(f, "LISTEN"),
            TcpStatus::SynSent => write!(f, "SYNSENT"),
            TcpStatus::SynRcvd => write!(f, "SYNRCVD"),
            TcpStatus::Established => write!(f, "ESTABLISHED"),
            TcpStatus::FinWait1 => write!(f, "FINWAIT1"),
            TcpStatus::FinWait2 => write!(f, "FINWAIT2"),
            TcpStatus::TimeWait => write!(f, "TIMEWAIT"),
            TcpStatus::CloseWait => write!(f, "CLOSEWAIT"),
            TcpStatus::LastAck => write!(f, "LASTACK"),
        }
    }
}

impl Socket {
    pub fn new(
        local_addr: Ipv4Addr,
        remote_addr: Ipv4Addr,
        local_port: u16,
        remote_port: u16,
        status: TcpStatus,
    ) -> Result<Self> {
        let (sender, _) = transport::transport_channel(65535,
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
        )?;
        Ok(Self { 
            local_addr, 
            remote_addr, 
            local_port, 
            remote_port, 
            send_param: SendParam { 
                unacked_seq: 0,
                next: 0,
                window: SOCKET_BUFFER_SIZE as u16,
                initial_seq: 0,
            },
            recv_param: RecvParam { 
                next: 0,
                window: SOCKET_BUFFER_SIZE as u16,
                initial_seq: 0,
                tail: 0,
            },
            status,
            retransmission_queue: VecDeque::new(),
            connected_connection_queue: VecDeque::new(),
            listening_socket: None,
            sender,
        })
    }

    pub fn send_tcp_packet(
        &mut self,
        seq: u32,
        ack: u32,
        flag: u8,
        payload: &[u8],
    ) -> Result<usize> {
        let mut tcp_packet = TCPPacket::new(payload.len());
        tcp_packet.set_src(self.local_port);
        tcp_packet.set_dest(self.remote_port);

        tcp_packet.set_seq(seq);
        tcp_packet.set_ack(ack);
        tcp_packet.set_data_offset(5); // オプションフィールドは使わないので固定

        tcp_packet.set_flag(flag);

        tcp_packet.set_window_size(self.recv_param.window);
        tcp_packet.set_payload(payload);
        tcp_packet.set_checksum(util::ipv4_checksum(
            &tcp_packet.packet(),
            8,
            &[],
            &self.local_addr,
            &self.remote_addr,
            IpNextHeaderProtocols::Tcp,
        ));

        let sent_size = self
            .sender
            .send_to(tcp_packet.clone(), IpAddr::V4(self.remote_addr))
            .context(format!("failed to send: \n{:?}", tcp_packet))?;

        dbg!("sent", &tcp_packet);

        // [note] 【再送制御】
        // Payloadが存在しない通常の"応答としてのACK"を再送すること(ACKのACKが来ることを期待すること)は無いので、
        // ここで早期returnをしている。
        // なぜなら、通信双方で応答に対する応答を期待してしまうと、無限にそのやり取りをすることになるので、
        // ACKのACKは返さないことを仕様で決めている。
        if payload.is_empty() && tcp_packet.get_flag() == tcpflags::ACK {
            return Ok(sent_size);
        }
        self.retransmission_queue
            .push_back(RetransmissionQueueEntry::new(tcp_packet));
        Ok(sent_size)
    }

    pub fn get_sock_id(&self) -> SockID {
        SockID(
            self.local_addr,
            self.remote_addr,
            self.local_port,
            self.remote_port,
        )
    }
}