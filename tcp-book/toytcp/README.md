
## Run the ToyTCP client

### 3章を終えた段階で

nsコマンドで受信側ホストでTCP受信(自作TCPじゃない)

```
sudo ip netns exec host2 nc -l 10.0.1.1 40000
```

自作TCPのクライアントアプリで送信側ホストからTCP送信

```
sudo ip netns exec host1 ./target/debug/examples/echoclient 10.0.1.1 40000
```

tcpdump すると以下のように通信できていることを確認した。

```
ubuntu@toytcp:~/toytcp/toytcp$ sudo ip netns exec host1 tcpdump -l
tcpdump: verbose output suppressed, use -v[v]... for full protocol decode
listening on host1-veth1, link-type EN10MB (Ethernet), snapshot length 262144 bytes
17:03:58.311417 IP 10.0.0.1.50467 > 10.0.1.1.40000: Flags [S], seq 1815885635, win 4380, length 0
17:03:58.311465 IP 10.0.1.1.40000 > 10.0.0.1.50467: Flags [S.], seq 2758249402, ack 1815885636, win 64240, options [mss 1460], length 0
17:03:59.386761 IP 10.0.1.1.40000 > 10.0.0.1.50467: Flags [S.], seq 2758249402, ack 1815885636, win 64240, options [mss 1460], length 0
17:03:59.387369 IP 10.0.0.1.50467 > 10.0.1.1.40000: Flags [.], ack 1, win 4380, length 0
```

## TCP Header Format from [RFC](https://datatracker.ietf.org/doc/html/rfc793#section-3.1)

```
    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |          Source Port          |       Destination Port        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                        Sequence Number                        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Acknowledgment Number                      |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |  Data |           |U|A|P|R|S|F|                               |
   | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
   |       |           |G|K|H|T|N|N|                               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |           Checksum            |         Urgent Pointer        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Options                    |    Padding    |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                             data                              |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

                            TCP Header Format
```