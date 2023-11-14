use std::io;
fn main() -> io::Result<()> {
    let nic = tun_tap::Iface::new("my_tun0", tun_tap::Mode::Tun).expect("failed to create");
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        eprintln!("read {} bytes: {:x?}", nbytes, &buf[..nbytes]);
        println!("nic: {:?}", nic);
    }
    Ok(())
}
