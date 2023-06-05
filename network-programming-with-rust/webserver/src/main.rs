use mio::tcp::{TcpListener, TcpStream};
use mio::{Event, Events, Poll, PollOpt, Ready, Token};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::SocketAddr;
use std::{env, process, str};
#[macro_use]
extern crate log;

// リスニングソケットのトークン 
// [note] リスニングソケットはサーバに1つしかない。TCP3Wayハンドシェイク後のAccept()なソケットは複数ある。 
const SERVER: Token = Token(0);

// ドキュメントルートのパス
const WEBROOT: &str = "/webroot";

struct WebServer {
	listening_socket: TcpListener,
	connections: HashMap<usize, TcpStream>, // サーバに接続されているクライアントを管理するハッシュテーブル
	next_connection_id: usize,
}

impl WebServer {
    /**
     * Initiate the web server
     */
    fn new(addr: &str) -> Result<Self, failure::Error> {
        let address = addr.parse::<SocketAddr>()?; // [Rust知識] ::<SocketAddr>が無くとも次の行にて型推論によって決まるが学習のためあえて型を指定している。
        let listening_socket = TcpListener::bind(&address)?;
        Ok(WebServer { 
            listening_socket: listening_socket,
            connections: HashMap::new(),
            next_connection_id: 1,
        })
    }

    /*
    ※1 
    読み込みの例で言うと、準備完了に切り替わった時だけ読み込みを行う場合はエッジトリガー、準備完了の間は常に読み込みを行う場合がレベルトリガーとなります。
    mioのドキュメントによると、ある監視対象に対してエッジトリガーモードでイベントが発生すると、Poll::poll()はその対象がWouldBlockを返すまで、
    トークンと監視対象操作（読み込み、書き込み）が一致しているイベントを返さないという記述があります。
    WouldBlockはRustの公式ドキュメントで定義されており、操作がブロックされてはならない時に、ブロックが発生してしまうような操作を呼び出した場合に、
    返却されるエラーの種類です。要するにmioのドキュメントでは、エッジトリガーモードでイベントが発生した場合、「待ち1」が発生し得るような「オフ」の状態を経ない限りは、
    同じトークンで再びイベントが発生することはないと述べているのです。
    Teruya Ono. Introduction to network programming with Rust (Japanese Edition) (pp. 104-105). Kindle Edition. 
    */

    /**
     * Start the web server
     */
    fn run(&mut self) -> Result<(), failure::Error> {
        let poll = Poll::new()?; // selectシステムコールをするmioのオブジェクト
        // Register server socket status to monitoring target
        poll.register(
            &self.listening_socket, // I/O多重化機構による監視対象のオブジェクト
            SERVER,                  // ソケットの複数生成においてトラックするためのもの。Using `Token` to track which socket generated the notification.
            Ready::readable(),    // 監視する命令の種類(読み込み or 書き込み)
            PollOpt::level(),         // イベント発生条件に関するオプション(※1)
        )?;

        // Event queue (実際のHTTPリクエストだと思えばよい)
        let mut events = Events::with_capacity(1024);
        // Buffer for HTTP response
        let mut response = Vec::new();

        loop {
            // Wait for event by blocking current thread
            match poll.poll(&mut events, None) {
                // poll()はI/O多重化機構の中心的なメソッド。
                // registerで登録した対象を監視し、対象の準備が完了したらイベントをキュー(`events`)に入れてスレッドを再開させる。
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            }

            for event in &events {
                match event.token() {
                    SERVER => {
                        // リスニングソケットの読み込み準備完了イベントが発生
                        // [note] Webサーバに1つしかないリスニングソケットにイベントが来たとき(TCP初期接続)
                        let (tcp_stream, remote_socket_addr) = match self.listening_socket.accept() {
                            Ok(t) => t,
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            },
                        };
                        debug!("Connection from {}", &remote_socket_addr);
                        // Register the new connected socket to monitoring target
                        self.register_connection(&poll, tcp_stream)
                            .unwrap_or_else(|e| error!("{}", e));
                    },
                    Token(cond_id) => {
                        // 接続済みソケットでイベントが発生
                        // [note] TCP3Wayハンドシェイク後のAccept()な複数あるソケットにイベントが来たとき(TCP接続後通信)
                        self.http_handler(cond_id, event, &poll, &mut response)
                            .unwrap_or_else(|e| error!("{}", e));
                    },
                }
            }

        }
    }

    /**
     * Register the new connected socket to monitoring target
     */
    fn register_connection(&mut self, poll: &Poll, stream: TcpStream) -> Result<(), failure::Error> {
        let token = Token(self.next_connection_id);
        poll.register(
            &stream,             // I/O多重化機構による監視対象のオブジェクト
            token,        // ソケットの複数生成においてトラックするためのもの。Using `Token` to track which socket generated the notification.
            Ready::readable(), // 監視する命令の種類(読み込み or 書き込み)
            PollOpt::edge(),       // イベント発生条件に関するオプション(※1)
        )?;

        // Webサーバにもコネクションを保持させる。
        if self.connections.insert(self.next_connection_id, stream).is_some() {
            // [Rust仕様] If the map(Hash) did not have this key present, [`None`] is returned.
            // ↑なので、Someが返っているということは重複しているということ
            error!("connection ID is already exists.");
        }
        self.next_connection_id += 1;
        Ok(())
    }

    /**
	 * 接続済みソケットで発生したイベントのハンドラ
	 */
	fn http_handler(
		&mut self,
		conn_id: usize,
		event: Event,
		poll: &Poll,
		response: &mut Vec<u8>,
	) -> Result<(), failure::Error> {
		let stream = self
			.connections
			.get_mut(&conn_id) // mutでコネクションを引っ張り出す。
			.ok_or_else(|| failure::err_msg("Failed to get connection."))?;
		if event.readiness().is_readable() {
            // リクエストが来た初めはリクエストデータを読み取る。
			debug!("readable conn_id: {}", conn_id);
			let mut buffer = [0u8; 1024];
			let nbytes = stream.read(&mut buffer)?;

			if nbytes != 0 {
                // 【このプログラムの一番のハイライト】[Point] 
                // リクエストを読み取り後、レスポンスを返す際はコネクションをWritableにしてEventループへ登録しなおす。
                // [note] 本来はここで読み取ったデータをアプリケーションサーバのハンドラへ渡すはずだが、
                // ここではWebサーバ側でレスポンスを作る。
                //
                // book: 送信バッファが満杯のときはブロックが発生してしまう。
                // > それを防ぐために、ソケットに対して監視する操作を「書き込み」に切り替えたのち関数から返ります。
                // > こうすることで書き込みの準備が完了した際にイベントが発生するため、ブロックされることなくレスポンスの送信を行うことができます。
                // > Teruya Ono. Introduction to network programming with Rust (Japanese Edition) (p. 109). Kindle Edition. 
				*response = make_response(&buffer[..nbytes])?;
				// 書き込み操作の可否を監視対象に入れる
				poll.reregister(
                    stream,
                    Token(conn_id),
                    Ready::writable(), // [Point] Writableにして"re"register
                    PollOpt::edge(),
                )?;
			} else {
				// 読み取りデータに何もなければ、通信終了(このWebサーバの仕様)
				self.connections.remove(&conn_id);
			}
			Ok(())
		} else if event.readiness().is_writable() {
            // [Point] レスポンスを返す状態のコネクション。
			// ソケットに書き込み可能。
			debug!("writable conn_id: {}", conn_id);
			stream.write_all(response)?;
			self.connections.remove(&conn_id);
			Ok(())
		} else {
			Err(failure::err_msg("Undefined event."))
		}
	}

}

/**
 * レスポンスをバイト列で作成して返す
 */
fn make_response(request_buffer: &[u8]) -> Result<Vec<u8>, failure::Error> {
    // request の分析(パースまではしない)
    // method情報だけが欲しい。
    let http_pattern = Regex::new(r"(.*) (.*) HTTP/1.([0-1])\r\n.*")?;
    let captures = match http_pattern.captures(str::from_utf8(request_buffer)?) {
        Some(cap) => cap,
        None => {
            // invalid request
            return create_msg_from_code(400, None);
        },
    };
    let method = captures[1].to_string();
	let path = format!(
		"{}{}{}",
		env::current_dir()?.display(),
		WEBROOT,
		&captures[2]
	);
	// let _version = captures[3].to_string();

    // response を生成
    // webrootにあるHTMLファイルを返す。
    if method == "GET" {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                return create_msg_from_code(400, None);
            },
        };
        let mut reader = BufReader::new(file);
		let mut buf = Vec::new();
		reader.read_to_end(&mut buf)?;
		return create_msg_from_code(200, Some(buf));
    } else {
        return create_msg_from_code(501, None);
    }
}

/**
 * HTTPステータスコードからメッセージを生成する。
 */
fn create_msg_from_code(status_code: u16, msg: Option<Vec<u8>>) -> Result<Vec<u8>, failure::Error> {
	match status_code {
		200 => {
			let mut header = "HTTP/1.0 200 OK\r\n\
			                  Server: mio webserver\r\n\r\n"
				.to_string()
				.into_bytes();
			if let Some(mut msg) = msg {
				header.append(&mut msg);
			}
			Ok(header)
		}
		400 => Ok("HTTP/1.0 400 Bad Request\r\n\
		           Server: mio webserver\r\n\r\n"
			.to_string()
			.into_bytes()),
		404 => Ok("HTTP/1.0 404 Not Found\r\n\
		           Server: mio webserver\r\n\r\n"
			.to_string()
			.into_bytes()),
		501 => Ok("HTTP/1.0 501 Not Implemented\r\n\
		           Server: mio webserver\r\n\r\n"
			.to_string()
			.into_bytes()),
		_ => Err(failure::err_msg("Undefined status code.")),
	}
}

fn main() {
    env::set_var("RUST_LOG", "debug");
	env_logger::init();
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		error!("Bad number of argments.");
		process::exit(1);
	}
	let mut server = WebServer::new(&args[1]).unwrap_or_else(|e| {
		error!("{}", e);
		panic!();
	});
	server.run().unwrap_or_else(|e| {
		error!("{}", e);
		panic!();
	});
}
