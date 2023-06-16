
## Run the ToyTCP client

### Active Open (Client side) 3.5節

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

### Passive Open (Server side) 3.6節

自作TCPでのサーバで受け付け開始

```
sudo ip netns exec host1 ./target/debug/examples/echoserver 10.0.0.1 40000
```

nc コマンドでクライアントとして接続(自作TCPじゃない)

```
sudo ip netns exec host2 nc 10.0.0.1 40000
```

tcpdump すると以下のように通信できていることを確認した。

```
ubuntu@toytcp:~/toytcp/toytcp$ sudo ip netns exec host1 tcpdump -l
tcpdump: verbose output suppressed, use -v[v]... for full protocol decode
listening on host1-veth1, link-type EN10MB (Ethernet), snapshot length 262144 bytes
15:25:07.079747 IP 10.0.1.1.39632 > 10.0.0.1.40000: Flags [S], seq 1917544382, win 64240, options [mss 1460,sackOK,TS val 2581156742 ecr 0,nop,wscale 7], length 0
15:25:07.080144 IP 10.0.0.1.40000 > 10.0.1.1.39632: Flags [S.], seq 2132066080, ack 1917544383, win 4380, length 0
15:25:07.080277 IP 10.0.1.1.39632 > 10.0.0.1.40000: Flags [.], ack 1, win 64240, length 0
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

## Socketが持つ情報

RFCの仕様に従う。

具体的な内容は [./src/socket.rs](./src/socket.rs) の冒頭のコメント部分を参照

## アクティブオープン (3.5節) パッシブオープン (3.6節)

[note] 厳密にはそう表現してはダメっぽいが、アクティブオープンがクライアントで、パッシブオープンがサーバと理解して良さそう。

下図にて、赤がアクティブオープンで青がパッシブオープンにあたる。

![image](https://upload.wikimedia.org/wikipedia/en/5/57/Tcp_state_diagram.png)

## 再送制御 (Section 3.7.4)

TCPでは自身のACK応答送信以外(※1)は、送信後にACKが返って来ない場合は再送をする。

再送実装において、ソケットごとに再送用のパケットキューを持たせておく。

そして、監視用のタイマースレッドを用意し各ソケットが持つ再送用パケットキューにて経過時間とタイムアウト時間(RTO)を比較し再送をする。

## スライディングウィンドウ (Section 3.7.6)

送信側はデータを立て続けに送信するため，受信者のキャパシティを超えた送信を行ってしまう可能性がある。

では1つずつ送信したセグメントの確認応答を受け取るまで次のデータを送らないようにすれば良さそうだが、それではパフォーマンスが大きく悪化してしまう。

そのため，ある程度のセグメントを確認応答無しで一気に送信できるように，送信可能データのサイズを管理する仕組みがあり、この送信サイズを制御する仕組みをウィンドウ制御と言う。

![image](https://i.ytimg.com/vi/klDhO9N01c4/maxresdefault.jpg)