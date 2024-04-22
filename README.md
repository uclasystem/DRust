# DRust-DSM: an easy-to-use, efficient, and consistent distributed shared memory library

This repository contains a research artifact which is an easy-to-use, efficient, and consistent distributed shared memory library. With DRust-DSM you can easily distribute your single machine application to multiple servers with great performance.

**Please see [ae.md](./docs/osdi24ae-77.md) for instructions to run our tool on provided servers.**


## 1. Environment Setup

### 1.1 Prerequisites

- N(>=2) physical servers with infiniband installed
- Ubuntu 18.04
- Linux 5.4 Kernel
- GCC 5.5 

### 1.2 Install MLNX_OFED driver

On each of your server, download and install the MLNX OFED driver for Ubuntu 18.04. DRust is tested using MLNX_OFED driver 4.9-2.2.4.0.

```bash
#Download MLNX_OFED driver 4.9-2.2.4.0
wget https://content.mellanox.com/ofed/MLNX_OFED-4.9-2.2.4.0/MLNX_OFED_LINUX-4.9-2.2.4.0-ubuntu18.04-x86_64.tgz
tar xzf MLNX_OFED_LINUX-4.9-2.2.4.0-ubuntu18.04-x86_64.tgz
cd MLNX_OFED_LINUX-4.9-2.2.4.0-ubuntu18.04-x86_64

# Remove the incompatible libraries
sudo apt remove ibverbs-providers:amd64 librdmacm1:amd64 librdmacm-dev:amd64 libibverbs-dev:amd64 libopensm5a libosmvendor4 libosmcomp3 -y

# Install the MLNX OFED driver against the kernel 5.4.0
sudo ./mlnxofedinstall --add-kernel-support
```

#### 1.2.1 Enable the opensm and openibd services

(1) Enable and start the openibd service

```bash
sudo systemctl enable openibd
sudo systemctl start  openibd
# confirm the service is running and enabled:
sudo systemctl status openibd

# the log shown as:
● openibd.service - openibd - configure Mellanox devices
   Loaded: loaded (/lib/systemd/system/openibd.service; enabled; vendor preset: enabled)
   Active: active (exited) since Mon 2022-05-02 14:40:53 CST; 1min 24s ago
``` 

(2) Enable and start the opensmd service:

```bash
sudo systemctl enable opensmd
sudo systemctl start opensmd

# confirm the service status
sudo systemctl status opensmd

# the log shown as:
opensmd.service - LSB: Manage OpenSM
   Loaded: loaded (/etc/init.d/opensmd; generated)
   Active: active (running) since Mon 2022-05-02 14:53:39 CST; 10s ago

#
# Warning: you may encounter the problem:
#
opensmd.service is not a native service, redirecting to systemd-sysv-install.
Executing: /lib/systemd/systemd-sysv-install enable opensmd
update-rc.d: error: no runlevel symlinks to modify, aborting!
```

Please refer to the following instructions for how to solve the problem encountered when enabling opensmd.

- Update the service start level in `/etc/init.d/opensmd`. The original `/etc/init.d/opensmd` is shown below.

  ```text
   8 ### BEGIN INIT INFO
   9 # Provides: opensm
  10 # Required-Start: $syslog openibd
  11 # Required-Stop: $syslog openibd
  12 # Default-Start: null
  13 # Default-Stop: 0 1 6
  14 # Description:  Manage OpenSM
  15 ### END INIT INFO
  ```

- Change line 12 in `/etc/init.d/opensmd` to be:
  
  ```text
  12 # Default-Start: 2 3 4 5
  ```

-  Enable and Start the opensmd service
  
    ```bash
    sudo update-rc.d opensmd remove -f
    sudo systemctl enable opensmd
    sudo systemctl start opensmd

    # confirm the service status
    sudo systemctl status opensmd
    ```

- The log shown as:
  
  ```text
  opensmd.service - LSB: Manage OpenSM
    Loaded: loaded (/etc/init.d/opensmd; generated)
    Active: active (running) since Mon 2022-05-02 14:53:39 CST; 10s ago
  ```

#### 1.2.2 Confirm the InfiniBand is available

```bash
#Get the InfiniBand information
ibstat
```

Example desired log is shown below (adapter's stat should be Active)

```bash
Port 1:
  State: Active
  Physical state: LinkUp
  Rate: 100
  Base lid: 3
  LMC: 0
  SM lid: 3
  Capability mask: 0x2651e84a
  Port GUID: 0x0c42a10300605e88
  Link layer: InfiniBand
```

### 1.3 Installing Rust with rustup

DRust is tested with `cargo 1.71.0-nightly`. On each server, install rustc and cargo with the following instructions.

```bash
sudo apt remove cargo rustc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
# fix the toolchain version to nightly-2023-04-25 to avoid dependency errors
rustup toolchain install nightly-2023-04-25
rustup default nightly-2023-04-25
```

### 1.4 Disable ASLR

To facilitate remote thread spawning, we require users to disable ASLR on each server. The following command should be executed every time the server is restarted.

```bash
echo 0 | sudo tee /proc/sys/kernel/randomize_va_space
```

## 2. Download and Install DRust

On all your servers' home directory, create a folder called `DRust_home`, and then download DRust into that directory:

```bash
cd ~/
mkdir DRust_home
cd DRust_home
git clone git@github.com:uclasystem/DRust.git
```

### 2.1 Download required datasets

Create a folder with name `dataset` in `DRust_home`. Download datasets and extract them to that folder.

```bash
cd ~/DRust_home
mkdir dataset
cd dataset
# Put the dataset here
```

### 2.2 Configuration

1. Configure number of servers. Suppose you have 8 servers. 
   - In line 27 of `comm-lib/rdma-common.h`, configure the `TOTAL_NUM_SERVERS` as 8.
   - In line 1 of `drust/src/conf.rs`, configure the `NUM_SERVERS` as 8.
2. Configure the size of your distributed heap. Suppose you want to allocate 16GB heap memory on each server.
   - In line 3 of `drust/src/conf.rs`, configure the `UNIT_HEAP_SIZE_GB` as 16.
3. In `comm-lib/rdma-server-lib.c:drust_start_server`, configure the infiniband ip address and port of your execution servers. E.g.

    ```C
    const char *ip_str[8] = {"10.0.0.1", "10.0.0.2", "10.0.0.3", "10.0.0.4", "10.0.0.5", "10.0.0.6", "10.0.0.10", "10.0.0.11"};
    const char *port_str[8] = {"9400", "9401", "9402", "9403", "9404", "9405", "9406", "9407"};
    ```
  
4. Configure the server IP addresses. In `drust/drust.json`. Input the IP address of each server and for each server, give it three available ports. E.g.

    ```json
    {
      "servers": [
        {
          "ip": "131.xxx.xxx.201:36758",
          "alloc_ip": "131.xxx.xxx.201:36759",
          "safepoint_ip": "131.xxx.xxx.201:36760"
        },
        {
          "ip": "131.xxx.xxx.202:36758",
          "alloc_ip": "131.xxx.xxx.202:36759",
          "safepoint_ip": "131.xxx.xxx.202:36760"
        }
      ]
    }
    ```

### 2.3 Build DRust

On each of your server, build release version of DRust using `cargo` by running 

```bash
cd ~/DRust_home/DRust/scripts
bash local_build.sh
```

or using the following step-by-step commands:

```bash
# Compile communication static library
cd ~/DRust_home/DRust/comm-lib
make clean
make -j lib

# Copy it drust folder
cp libmyrdma.a ../drust/

# Compile the rust part
cd ~/DRust_home/DRust/drust
~/.cargo/bin/cargo build --release
```

Note that you may encounter the following error:

```bash
error: package `clap v4.5.4` cannot be built because it requires rustc 1.74 or newer, while the currently active rustc version is 1.71.0-nightly
Either upgrade to rustc 1.74 or newer, or use
cargo update -p clap@4.5.4 --precise ver
where `ver` is the latest version of `clap` supporting rustc 1.71.0-nightly
```

Please run the following command and then run building process again:

```bash
~/.cargo/bin/cargo update -p clap@4.5.4 --precise 4.3.0
~/.cargo/bin/cargo build --release
```

Make sure that this building process runs successfully on each server.

### 2.4 Deployment

Copy the executable from your main server (server 0) to other servers using the following command:

```bash
scp ~/DRust_home/DRust/target/release/drust user@ip:DRust_home/DRust/drust.out
```

## 3. Running Applications

We have four different example applications:

- Dataframe
- GEMM
- KVStore
- SocialNet

To run each application, you need to run the executable on each server through the following process:

1. On all your servers except the main server(server 0), run the following command:

    ```bash
    cd ~/DRust_home/DRust/drust

    # Replace the server_id with the index for the server that is executing this executable. The main server should have 0 as its index. Suppose you have 8 servers, then the last server's index is 7.
    # Replace the app_name with the name for the application you want to run. The example applications' names are dataframe, gemm, kv, sn.
    # For example, ./../target/release/drust -s 7 -a dataframe
    ./../drust.out -s server_id -a app_name
    ```

2. After starting the executable on all other servers, wait for 2 seconds, and then on your main server, run:

    ```bash
    cd ~/DRust_home/DRust/drust

    # Replace the app_name with the dataframe, gemm, kv, or sn.
    ./../drust.out -s 0 -a app_name
    ```

## 4. Code structure

| Directory	 | Description |
| :-----| :---- |
| applications | The single machine version of each application |
| comm-lib | RDMA communication library code written in C |
| docs | Documentations of DRust |
| drust/app | Applications integrated with DRust |
| drust/drust_std | DRust library code |
| scripts | Scripts to faciliate deployment of DRust |

## 5. Add Your Own Application

Please see [library.md](./docs/library.md).

