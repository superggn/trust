use std::io;
fn main() -> io::Result<()> {
    let nic = tun_tap::Iface::new("my_tun0", tun_tap::Mode::Tun).expect("failed to create");
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto = u16::from_be_bytes([buf[2], buf[3]]);
        eprintln!(
            "read {} bytes (flags: {:x}, proto: {:x}): {:x?}",
            nbytes - 4,
            flags,
            proto,
            &buf[4..nbytes]
        );
        println!("nic: {:?}", nic);
    }
    Ok(())
}
