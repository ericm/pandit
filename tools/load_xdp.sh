sudo ip link set dev lo xdpgeneric off
sudo ip link set dev lo xdpgeneric obj /vagrant/build/xdp_parser.bpf.o sec xdp