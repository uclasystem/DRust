# DRust-DSM: an easy-to-use, efficient, and consistent distributed shared memory system

This repository contains a research artifact DRust, an efficient, consistent, and easy-to-use distributed shared memory system. With DRust, you can quickly scale your applications from a single machine to a multi-server environment without compromising performance. 

**Please see [AE.md](./docs/osdi24ae-77.md) for instructions to run our tool on provided servers.**


## 1. Environment Setup

This section will help you set up the necessary environment. Before you begin, ensure you have the following prerequisites.

### 1.1 Prerequisites

- N(>=2) physical servers with infiniband installed
- Ubuntu 18.04
- Linux 5.4 Kernel
- GCC 5.5 

### 1.2 Install MLNX_OFED driver

DRust-DSM requires the MLNX OFED driver for Ubuntu 18.04. To install it on each server, follow these steps:

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

If you encounter errors while enabling opensmd, follow these instructions:

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

### 1.3 Installing Rust with `rustup`

DRust-DSM requires a specific version of the Rust toolchain. Here's how to install it on each server:

```bash
sudo apt remove cargo rustc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
# fix the toolchain version to nightly-2023-04-25 to avoid dependency errors
rustup toolchain install nightly-2023-04-25
rustup default nightly-2023-04-25
```

### 1.4 Disabling ASLR

To allow remote thread spawning, disable Address Space Layout Randomization (ASLR) on each server. This command must be executed after every reboot:

```bash
echo 0 | sudo tee /proc/sys/kernel/randomize_va_space
```

## 2. Download and Install DRust

Follow these steps to download and install DRust on all your servers.

### 2.1 Download DRust

On each server, create a directory named `DRust_home` in the home directory and use the following command to clone the DRust repository into DRust_home:

```bash
cd ~/
mkdir DRust_home
cd DRust_home
git clone git@github.com:uclasystem/DRust.git
```

### 2.2 Download required datasets

After setting up the DRust directory, inside `DRust_home`, create a `dataset` folder. Download datasets and extract them to that folder.

```bash
cd ~/DRust_home
mkdir dataset
cd dataset
# Put the dataset here
```

### 2.3 Configure DRust

Several configurations need to be adjusted based on your server setup and requirements. Follow these steps to configure DRust:

1. Set the Number of Servers
    - In `comm-lib/rdma-common.h`, set `TOTAL_NUM_SERVERS` to the total number of servers you have.
    - In `drust/src/conf.rs`, set `NUM_SERVERS` to the same number.
2. Configure Distributed Heap Size
    - In `drust/src/conf.rs`, set `UNIT_HEAP_SIZE_GB` to the desired heap size for each server (e.g., 16 for 16GB).
3. Set InfiniBand IP Addresses and Ports
    - In `comm-lib/rdma-server-lib.c:drust_start_server`, update `ip_str` and `port_str` arrays with your servers' IP addresses and ports. Example:

      ```C
      const char *ip_str[8] = {"10.0.0.1", "10.0.0.2", "10.0.0.3", "10.0.0.4", "10.0.0.5", "10.0.0.6", "10.0.0.10", "10.0.0.11"};
      const char *port_str[8] = {"9400", "9401", "9402", "9403", "9404", "9405", "9406", "9407"};
      ```
  
4. Configure Server IP Addresses in `drust.json`
    - Modify `drust/drust.json` with each server's IP address and three available ports. Example: 
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

### 2.4 Build DRust

To build DRust, follow these steps on each server:

#### Run the Build Script

You can build DRust using a script provided in the scripts directory:

```bash
cd ~/DRust_home/DRust/scripts
bash local_build.sh
```

#### Manually Compile the Communication Library and Rust Code

Alternatively, use these step-by-step commands to build DRust:

```bash
# Compile communication static library
cd ~/DRust_home/DRust/comm-lib
make clean
make -j lib

# Copy it to the DRust folder
cp libmyrdma.a ../drust/

# Compile the Rust part
cd ~/DRust_home/DRust/drust
~/.cargo/bin/cargo build --release
```

#### Fix Build Errors with clap Dependency

If you encounter errors related to the clap package, update it to a compatible version. For the following error:

```bash
error: package `clap v4.5.4` cannot be built because it requires rustc 1.74 or newer, while the currently active rustc version is 1.71.0-nightly
Either upgrade to rustc 1.74 or newer, or use
cargo update -p clap@4.5.4 --precise ver
where `ver` is the latest version of `clap` supporting rustc 1.71.0-nightly
```

Run the following command to fix it and then run building process again:

```bash
~/.cargo/bin/cargo update -p clap@4.5.4 --precise 4.3.0
~/.cargo/bin/cargo build --release
```


### 2.5 Deployment

After building DRust, deploy the compiled binary to other servers:

#### Copy the Executable to Other Servers

From your main server (server 0), use scp to copy the drust.out executable to the same directory on other servers:
bash

```bash
scp ~/DRust_home/DRust/target/release/drust user@ip:DRust_home/DRust/drust.out
```

#### Ensure Successful Deployment

Verify that the executable is copied correctly to each server.


## 3. Running Applications

DRust comes with four example applications:

- Dataframe
- GEMM
- KVStore
- SocialNet


### 3.1 Start the DRust Executable on All Servers Except the Main Server

On all servers except the main server (server 0), navigate to the DRust executable, and run the executable with the appropriate server ID and application name:

```bash
cd ~/DRust_home/DRust/drust

# Replace the server_id with the index for the server that is executing this executable. The main server should have 0 as its index. Suppose you have 8 servers, then the last server's index is 7.
# Replace the app_name with the name for the application you want to run. The example applications' names are dataframe, gemm, kv, sn.
# For example, ./../target/release/drust -s 7 -a dataframe
./../drust.out -s server_id -a app_name
```

### 3.2 Start the DRust Executable on the Main Server

After running the executable on all other servers, wait for 2 seconds, then on the main server (server 0), Start the executable with index 0 and the desired application name:

```bash
cd ~/DRust_home/DRust/drust

# Replace the app_name with the dataframe, gemm, kv, or sn.
./../drust.out -s 0 -a app_name
```

## 4. Code Structure

The DRust codebase is organized into several directories. Here's what each directory contains:

| Directory	 | Description |
| :-----| :---- |
| applications | The single machine version of each application |
| comm-lib | RDMA communication library code written in C |
| docs | Documentations of DRust |
| drust/app | Applications integrated with DRust |
| drust/drust_std | DRust library code |
| scripts | Scripts to faciliate deployment of DRust |

## 5. Adding Your Own Application


To add your own application to DRust-DSM, please refer to [library.md](./docs/library.md) for detailed instructions on how to integrate your custom applications.
