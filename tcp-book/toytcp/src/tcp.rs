use crate::packet::TCPPacket;
use crate::socket::{SockID, Socket, TcpStatus};
use crate::tcpflags;
use anyhow::{Context, Result, Ok};
use pnet::packet::{ip::IpNextHeaderProtocols, tcp::TcpPacket, Packet};
use pnet::transport::{self, TransportChannelType};
use rand::{rngs::ThreadRng, Rng};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex, RwLock, RwLockWriteGuard};
use std::time::{Duration, SystemTime};
use std::{cmp, ops::Range, str, thread};

const UNDETERMINED_IP_ADDR: std::net::Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const UNDETERMINED_PORT: u16 = 0;
const MAX_TRANSMITTION: u8 = 5;
const RETRANSMITTION_TIMEOUT: u64 = 3;
const MSS: usize = 1460;
const PORT_RANGE: Range<u16> = 40000..60000;

#[derive(Debug, Clone, PartialEq)]
pub enum TCPEventKind {
    ConnectionCompleted,
    Acked,
    DataArrived,
    ConnectionClosed,
}

#[derive(Debug, Clone, PartialEq)]
struct TCPEvent {
    sock_id: SockID, // イベント発生元のソケットID
    kind: TCPEventKind,
}

impl TCPEvent {
    fn new(sock_id: SockID, kind: TCPEventKind) -> Self {
        Self { sock_id, kind }
    }
}

pub struct TCP {
    // TCPが持つソケット群は、送信用スレッド・受信スレッド・再送管理用のタイマースレッドの、
    // 少なくとも3つのスレッドで共有・書き込みされるため、RwLockでハッシュテーブルを保護し、
    // TCP::new() ではArcを返すようにする。
    sockets: RwLock<HashMap<SockID, Socket>>,

    // 「コネクションを確立した」「ペイロードを受信した」といったイベントを他のスレッドから
    // 受け取るまで待機する処理のために、Condvarを利用する。
    event_condvar: (Mutex<Option<TCPEvent>>, Condvar),
}

impl TCP {
    pub fn new() -> Arc<Self> {
        // let sockets = HashMap::new();
        // let tcp = Self { sockets };
        // tcp
        let sockets = RwLock::new(HashMap::new());

        // パケットの送信用スレッド(この関数で返す先のメインスレッド)
        let tcp = Arc::new(Self {
            sockets,
            event_condvar: (Mutex::new(None), Condvar::new()),
        });

        // パケットの受信用スレッドの生成
        let cloned_tcp = tcp.clone();
        std::thread::spawn(move || {
            cloned_tcp.receive_handler().unwrap();
        });

        tcp
    }

    // TCP接続のためにローカルポート番号をランダム関数を利用して選ぶ
    fn select_unused_port(&self, rng: &mut ThreadRng) -> Result<u16> {
        for _ in 0..(PORT_RANGE.end - PORT_RANGE.start) { // [note] ここは別にRANGE分試行しなくても良いはず
            let local_port_from_random = rng.gen_range(PORT_RANGE);
            let table = self.sockets.read().unwrap();
            if table.keys().all(|k| k.2 != local_port_from_random) {
                // 既存のどのsocketsのポートとも重複していなければそれに決定する
                return Ok(local_port_from_random);
            }
        }
        anyhow::bail!("no available port found.")
    }

    // ターゲットに接続し、接続済みソケットIDを返す
    pub fn connect(&self, addr: Ipv4Addr, port: u16) -> Result<SockID> {
        let mut rng = rand::thread_rng();
        let mut socket = Socket::new(
            get_source_addr_to(addr)?,
            addr,
            self.select_unused_port(&mut rng)?,
            port,
            TcpStatus::SynSent,
        )?;

        // [note] TCPシーケンス番号予測攻撃を避けるために、初期シーケンス番号は乱数を用いて生成する。
        socket.send_param.initial_seq = rng.gen_range(1..1 << 31);

        // 生成したソケットを使って初期TCP送信する
        socket.send_tcp_packet(
            socket.send_param.initial_seq,
            0,
            tcpflags::SYN,
            &[],
        );

        // TCP初期送信(SYN)後に、ソケット上のデータを更新する。
        socket.send_param.unacked_seq = socket.send_param.initial_seq; // TCP仕様のソケット情報の更新
        socket.send_param.next = socket.send_param.initial_seq + 1;    // TCP仕様のソケット情報の更新
        /* ↑のnextを +1 している箇所に関して
        【書籍】
        SYNセグメントを送信する際にsend_param.nextを1つ進める:SYNセグメントはペイロードを持たないため，
        send_param.nextは進まないように思いますが，実際には確認応答を受けるために1つインクリメントします.
        これは受信側のrecv_param.nextにおいても同様で，SYNセグメントの他にFINセグメントも同様の働きを持ちます．
        > Teruya Ono. Rust TCP Book (Japanese Edition) (pp. 72-73). Kindle Edition. 
        */

        // ソケット群へこの新規のソケットを追加する。
        let mut table = self.sockets.write().unwrap();
        let sock_id = socket.get_sock_id();
        table.insert(sock_id, socket);

        // 上記でソケットテーブルに入れたのでtable変数に持たせていたLockを早々に外して、他のスレッドが触れるようにする。
        drop(table);

        // コネクション確立が成功するまで待ってから呼び出し元へソケットデータを返す。
        self.wait_event(sock_id, TCPEventKind::ConnectionCompleted);
        Ok(sock_id)
    }

    // 指定のソケットに指定のイベントが来るまでwaitするメソッド
    // TCP受信ソケット側のスレッドがEvent通知してくるのでそれを待つ。
    fn wait_event(&self, sock_id: SockID, kind: TCPEventKind) {
        // [note] TCP構造体で持っている (Mutex, CondVar) のペアを取得する。
        // このペアはEvent監視・通知を制御するためのもの。
        let (lock, cvar) = &self.event_condvar;
        let mut event = lock.lock().unwrap();
        loop {
            if let Some(ref e) = *event {
                if e.sock_id == sock_id && e.kind == kind {
                    break;
                }
            }
            // cvarがnotifyされるまでeventのロックを外して待機
            event = cvar.wait(event).unwrap();
            // 【Condvar.wait(guard)の仕様】.wait から返ったときはLockは再度取得される。
        }
        dbg!(&event);
        // [note] このメソッドを抜けるときは期待していたイベント(SockID, EventKind)を消化できているので、
        // *eventをNoneで上書きして戻しておく。
        *event = None;
        // このメソッドが終わるとき(eventがスコープから抜けるとき)eventが持っているLockは開放される。
    }

    /// 受信スレッド用の関数．
    /// [note] 受信スレッドのEntry Point
    /// [やっていること] IPレイヤからパケットを受け取り、自作のTCPソケット群で対応するソケットを検索し、処理ハンドラへ渡す。
    fn receive_handler(&self) -> Result<()> {
        dbg!("begin recv thread");

        // 初めにIPレイヤのパケットを受け取る口を用意する。
        let (_, mut receiver) = transport::transport_channel(
            65535,
            TransportChannelType::Layer3(IpNextHeaderProtocols::Tcp), // IPアドレスが必要なので，IPパケットレベルで取得．
        )
        .unwrap();
        let mut packet_iter = transport::ipv4_packet_iter(&mut receiver);

        // [note] ループで永続的にIPレイヤの口からパケットを受け付け→取得する
        loop {
            // 受信Waitをする
            let (packet, remote_addr) = match packet_iter.next() {
                std::result::Result::Ok((p, r)) => (p, r),
                Err(_) => continue,
            };
            let local_addr = packet.get_destination();

            // pnet(PureなIP/TCPにあたる)のTcpPacketから自作のtcp::TCPPacketに変換する
            let tcp_packet = match TcpPacket::new(packet.payload()) {
                Some(p) => p,
                None => {
                    continue;
                }
            };
            let packet = TCPPacket::from(tcp_packet); // 変換処理
            let remote_addr = match remote_addr {
                IpAddr::V4(addr) => addr,
                _ => {
                    continue;
                }
            };

            // [note] TCPPacketに記述されている情報から対応するTCPソケットを紐付ける
            let mut table = self.sockets.write().unwrap();
            let socket = match table.get_mut(&SockID( // [note] 既存の作成済みソケットに関わる受信パケットであるか判断
                local_addr,
                remote_addr,
                packet.get_dest(),
                packet.get_src(),
            )) {
                Some(socket) => socket, // 接続済みソケット
                None => match table.get_mut(&SockID( // [note] 既存の作成済みでは無いならばリスニングソケット(初期接続)であるか判断
                    local_addr,
                    UNDETERMINED_IP_ADDR,
                    packet.get_dest(),
                    UNDETERMINED_PORT,
                )) {
                    Some(socket) => socket, // リスニングソケット
                    None => continue,       // どのソケットにも該当しないものは無視
                },
            };

            // [note] チェックサム処理
            if !packet.is_correct_checksum(local_addr, remote_addr) {
                dbg!("invalid checksum");
                continue;
            }

            // [note] 受信したパケットとその受信したパケットに対応するソケットを引数にして、ソケットのステータス状況に応じてハンドリングする
            let sock_id = socket.get_sock_id();
            if let Err(error) = match socket.status {
                // TcpStatus::Listen => self.listen_handler(table, sock_id, &packet, remote_addr),
                // TcpStatus::SynRcvd => self.synrcvd_handler(table, sock_id, &packet),
                TcpStatus::SynSent => self.synsent_handler(socket, &packet),
                // TcpStatus::Established => self.established_handler(socket, &packet),
                // TcpStatus::CloseWait | TcpStatus::LastAck => self.close_handler(socket, &packet),
                // TcpStatus::FinWait1 | TcpStatus::FinWait2 => self.finwait_handler(socket, &packet),
                _ => {
                    dbg!("not implemented state");
                    Ok(())
                }
            } {
                dbg!(error);
            }
        }
    }    

    /// SYNSENT状態のソケットに到着したパケットの処理
    /// [note] TCPの仕様(RFC)に従って SYNSENT状態であるソケットの状態(recv_param, send_param)を更新する。
    fn synsent_handler(&self, socket: &mut Socket, packet: &TCPPacket) -> Result<()> {
        dbg!("synsent handler");

        /*
        ここのif式ですが，次のようなTCPにおけるセグメントの受信時全般に当てはまる条件を述べています．

        ・ACKビットが立っている:セグメント基本的にACKビットがONになっている必要があります．
        例外は受信するソケットがLISTEN状態の時です．
        なおセグメントのACKビットと確認応答番号フィールド(ack)は役割が異なるので混同してはいけません．

        ・socket.send_param.unacked_seq<=packet.get_ack()<=socket.send_param.nextを満たす:セグメントが運んでくる確認応答番号は正しい範囲内に含まれる必要があります．
        socket.send_param.unacked_seq>packet.get_ack()の時は既に確認応答されたシーケンス番号に対する確認応答が二重に届いたことになり，packet.get_ack()>socket.send_param.nextの時はまだ送信していないセグメントに対する確認応答が届いたことになります．
        
        いずれも不正な状態なので届いたセグメントは破棄されます．

        > Teruya Ono. Rust TCP Book (Japanese Edition) (pp. 79-80). Kindle Edition. 
         */
        if packet.get_flag() & tcpflags::ACK > 0
            && socket.send_param.unacked_seq <= packet.get_ack()
            && packet.get_ack() <= socket.send_param.next
            && packet.get_flag() & tcpflags::SYN > 0
        {
            socket.recv_param.next = packet.get_seq() + 1;
            socket.recv_param.initial_seq = packet.get_seq();
            socket.send_param.unacked_seq = packet.get_ack();
            socket.send_param.window = packet.get_window_size();
            if socket.send_param.unacked_seq > socket.send_param.initial_seq {
                // [note] 【ここのスコープが正常系】SYNSENT状態で待ち受けていて、
                // ちゃんと相手から期待どおりSYN|ACKセグメントがきたとき

                // ソケットのステータスを変更
                socket.status = TcpStatus::Established;

                // 相手へACKで返す
                socket.send_tcp_packet(
                    socket.send_param.next,
                    socket.recv_param.next,
                    tcpflags::ACK,
                    &[],
                )?;
                dbg!("status: synsent ->", &socket.status);

                // 送信側のスレッドに対して相手から期待どおりにSYN|ACKが返ってソケットステータスをEstablishedにしたことを通知する。
                // これによって、送信側のスレッドにて送信リクエストをしたアプリケーション側へ処理を返すことができる。
                self.publish_event(socket.get_sock_id(), TCPEventKind::ConnectionCompleted);
            } else {
                // [note] どういう状況のスコープか理解できていない。ソケットをコネクション構築OKにはせずACK送信している。
                socket.status = TcpStatus::SynRcvd;
                socket.send_tcp_packet(
                    socket.send_param.next,
                    socket.recv_param.next,
                    tcpflags::ACK,
                    &[],
                )?;
                dbg!("status: synsent ->", &socket.status);
            }
        }
        Ok(())
    }

    /// 指定のソケットIDにイベントを発行する
    fn publish_event(&self, sock_id: SockID, kind: TCPEventKind) {
        let (lock, cvar) = &self.event_condvar;
        let mut e = lock.lock().unwrap();
        *e = Some(TCPEvent::new(sock_id, kind));
        cvar.notify_all();
    }
}

/// 宛先IPアドレスに対する送信元インタフェースのIPアドレスを取得する
/// 
/// [note] 以下のように ip route コマンドを叩いた結果を取得している。
/// 
/// $ ip route get 10.0.0.1
/// 10.0.0.1 via 192.168.64.1 dev enp0s1 src 192.168.64.7 uid 1000
/// 
/// iproute2-ss180129で動作を確認．バージョンによって挙動が変わるかも
fn get_source_addr_to(addr: Ipv4Addr) -> Result<Ipv4Addr> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("ip route get {} | grep src", addr))
        .output()?;
    let mut output = str::from_utf8(&output.stdout)?
        .trim()
        .split_ascii_whitespace();
    while let Some(s) = output.next() {
        if s == "src" {
            break;
        }
    }
    let ip = output.next().context("failed to get src ip")?;
    dbg!("source addr", ip);
    ip.parse().context("failed to parse source ip")
}