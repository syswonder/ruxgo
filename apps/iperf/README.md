# Build and Run [`iperf3`](https://github.com/redis/redis) server in two ways and benchmark network performance

## 1. Build and run locally

Firstly, clone github repository of iperf and configure it: 

```bash
git clone -b 3.1-STABLE https://github.com/esnet/iperf.git && cd iperf && ./configure
```

Then, you need to copy `config_linux.toml` from `ruxgo/apps/iperf/local` and place it in the `iperf/` directory you just downloaded. 

Finally, cd into `iperf/` and execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r --bin-args=-s,-p,5555
```

## 2. Build and run on RuxOS:

Firstly, you need to copy `config_linux.toml` and `iperf.patch` from `ruxgo/apps/iperf/ruxos` and place it in the `ruxos/apps/c/iperf` at the same level as `iperf-3.1.3`.

Then, switch to `ruxos/apps/c/iperf` directory. If `iperf-3.1.3` does not exist in the `ruxos/apps/c/iperf` directory, execute the following prerequisite commands (if it does, it is not required):

```bash
wget https://downloads.es.net/pub/iperf/iperf-3.1.3.tar.gz
tar -zxvf iperf-3.1.3.tar.gz && rm -f iperf-3.1.3.tar.gz
patch -p1 -N -d iperf-3.1.3 --no-backup-if-mismatch -r - < iperf.patch
```

Finally, execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```

## 3. Benchmark

In another shell, run the `iperf3` client:

* iperf on RuxOS as the receiver:

  ```bash
  # TCP
  iperf3 -c 127.0.0.1 -p 5555
  # UDP
  iperf3 -uc 127.0.0.1 -p 5555 -b <sender_bitrate> -l <buffer_len>
  ```

  You need to set the `<sender_bitrate>` (in bits/sec) to avoid sending packets too fast from the client when use UDP.

* iperf on RuxOS as the sender:

  ```bash
  # TCP
  iperf3 -c 127.0.0.1 -p 5555 -R
  # UDP
  iperf3 -uc 127.0.0.1 -p 5555 -b 0 -l <buffer_len> -R
  ```

By default, the `<buffer_len>` is 128 KB for TCP and 8 KB for UDP. Larger buffer length may improve the performance. You can change it by the `-l` option of `iperf3`.

Note that if the `<buffer_len>` is greater than `1472` (total packet length is exceeded the MTU of the NIC) when use UDP, packets fragmentation will occur. You should enable fragmentation features in [smoltcp](https://github.com/smoltcp-rs/smoltcp):

```toml
# in ruxos/modules/axnet/Cargo.toml
[dependencies.smoltcp]
git = "https://github.com/rcore-os/smoltcp.git"
rev = "2ade274"
default-features = false
features = [
  "alloc", "log",   # no std
  "medium-ethernet",
  "proto-ipv4",
  "socket-raw", "socket-icmp", "socket-udp", "socket-tcp", "socket-dns",
  "fragmentation-buffer-size-65536", "proto-ipv4-fragmentation",
  "reassembly-buffer-size-65536", "reassembly-buffer-count-32",
  "assembler-max-segment-count-32",
]
```