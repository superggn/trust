use std::io;
use std::io::prelude::*;

enum State {
    Closed,
    Listen,
    // SynRvcd,
    // Estab,
}

// 实际上就是 tcb， transmission control block
pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

/// State of Send Sequence Space (RFC 793 S3.2 F4).
/// (https://www.rfc-editor.org/rfc/rfc793#page-20)
///
/// ```
/// 1         2          3          4
/// ----------|----------|----------|----------
///   SND.UNA    SND.NXT    SND.UNA
///                        +SND.WND
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
///        Send Sequence Space
/// ```

struct SendSequenceSpace {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// initial send sequence number
    iss: u32,
}

/// State of Receive Sequence Space (RFC 793 S3.2 F5).
/// (https://www.rfc-editor.org/rfc/rfc793#page-20)
///
/// ```
///                        1          2          3
///                    ----------|----------|----------
///                           RCV.NXT    RCV.NXT
///                                     +RCV.WND
///         1 - old sequence numbers which have been acknowledged
///         2 - sequence numbers allowed for new reception
///         3 - future sequence numbers which are not yet allowed
/// ```

struct RecvSequenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];
        if !tcph.syn() {
            // only expect syn packet
            return Ok(None);
        }
        let mut c: Connection = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                una: (),
                nxt: (),
                wnd: (),
                up: (),
                wl1: (),
                wl2: (),
                iss: (),
            },
            recv: RecvSequenceSpace {
                nxt: (),
                wnd: (),
                up: (),
                irs: (),
            },
        };

        // keep track of sender info
        self.recv.irs = tcph.sequence_number();
        self.recv.wnd = tcph.window_size();
        self.recv.nxt = tcph.sequence_number() + 1;

        // decide the stuff we're sending them
        self.send.iss = 0; // 暂时填0，后面再改成 random number
        self.send.una = self.send.iss;
        self.send.nxt = self.send.una + 1;
        self.send.wnd = 10; // randomly chosen, todo 待会儿改

        // need to start establishing a connection
        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            self.send.iss,
            self.send.wnd,
        );
        // todo 感觉这里应该是 sequence number + segment.len() + 1
        syn_ack.acknowledgment_number = self.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        let mut ip = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            etherparse::IpTrafficClass::Tcp,
            [
                iph.destination()[0],
                iph.destination()[1],
                iph.destination()[2],
                iph.destination()[3],
            ],
            [
                iph.source()[0],
                iph.source()[1],
                iph.source()[2],
                iph.source()[3],
            ],
        );

        // write out the headers
        // unwritten: how much space is remaining in the buffer
        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(unwritten);
            syn_ack.write(unwritten);
            unwritten.len()
        };
        nic.send(&buf[..unwritten])
    }
}
