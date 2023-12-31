#! /bin/sh
cargo b --release

ext=$?
echo "$ext"
if [[ $ext -ne 0 ]]; then
    exit $ext
fi

sudo setcap cap_net_admin=eip $CARGO_TARGET_DIR/release/trust

$CARGO_TARGET_DIR/release/trust &
pid=$!

sudo ip addr add 192.168.0.1/24 dev my_tun0
sudo ip link set up dev my_tun0

trap "kill $pid" TERM INT

wait $pid
