use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

/**
 * connect to specified IP address and port number
 */
pub fn connect(address: &str) -> Result<(), failure::Error> {
    let mut stream = TcpStream::connect(address)?;
    // この段階でTCPの3 Wayハンドシェイクが行われてコネクションが確立される。

    loop {
        // send input data from socket
        let mut input: String = String::new();
        io::stdin().read_line(&mut input)?;
        stream.write_all(input.as_bytes())?; // streamに書き込む → すなわちTCPでの送信

        // show received data from socket
        let mut reader = BufReader::new(&stream); // BufReaderはデータ元を保持する
        let mut buffer = Vec::new();
        reader.read_until(b'\n', &mut buffer)?;
        print!("{}", str::from_utf8(&buffer)?);
    }
}