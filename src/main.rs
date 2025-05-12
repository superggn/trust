use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}
fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::new("my_tun0", tun_tap::Mode::Tun).expect("failed to create");
    let mut buf = [0u8; 1504];
    loop {
        // 每次 recv 就是往 buf 里面填充数据帧
        let nbytes = nic.recv(&mut buf[..])?;
        let eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        if eth_proto != 0x0800 {
            // ipv4: 0x0800
            // if current packet is not ipv4: continue
            continue;
        }
        // &buf[4..nbytes] 是网络层数据帧， 从网络层数据帧能扒出来 ipv4 header
        // ipv4 header 里面的 protocol number 会说明 传输层用啥协议
        // 注意，这里的 buf 的全长还是上面定义的 nbytes
        // 我们只是填充了一部分, 所以 buf 只读到 nbytes 就不往后读了
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                let tx_layer_proto = ip_header.protocol();
                if tx_layer_proto != 0x06 {
                    continue;
                };
                // &buf[4 + ipv4_header.slice().len()..nbytes] 是传输层数据帧， 能扒出来 tcp header
                match etherparse::TcpHeaderSlice::from_slice(
                    &buf[4 + ip_header.slice().len()..nbytes],
                ) {
                    Ok(tcp_header) => {
                        let data_index = 4 + ip_header.slice().len() + tcp_header.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcp_header.source_port()),
                                dst: (dst, tcp_header.destination_port()),
                            })
                            .or_default()
                            .on_packet(&mut nic, ip_header, tcp_header, &buf[data_index..nbytes])?;
                        // eprintln!(
                        //     "{} - {} {}b of tcp to port {} ",
                        //     src,
                        //     dst,
                        //     tcp_header.slice().len(),
                        //     tcp_header.destination_port(),
                        // );
                    }
                    Err(e) => {
                        eprintln!("ignoring weird tcp packet: {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("ignoring weird ipv4 packet: {:?}", e);
            }
        }
        println!("nic: {:?}", nic);
    }
    Ok(())
}
