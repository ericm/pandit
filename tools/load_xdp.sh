llvm-strip-13 --no-strip-all --remove-section .BTF /vagrant/target/bpf/programs/entrypoint/entrypoint.elf 
sudo ip link set dev lo xdpgeneric off
sudo ip link set dev lo xdpgeneric obj /vagrant/target/bpf/programs/entrypoint/entrypoint.elf sec "xdp/entrypoint"