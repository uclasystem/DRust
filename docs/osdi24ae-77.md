# README for OSDI 2024 Artifact Evaluation

Welcome to the artifact evaluation guide for our research artifact DRust. This README will help you run our system and reproduce the key results. The official README is in our [GitHub repository](https://github.com/uclasystem/DRust/).


## General Instructions

For initial testing, we recommend try building and running the single-machine version of our tool on your own server. This setup doesn't require specialized hardware like InfiniBand. But feel free to skip this step if you want to directly try building our tool on our provided servers.
To reproduce the key results, you'll need at least eight machines with InfiniBand, which we provide along with scripts to run our system. Since concurrent execution on the same server can affect results, please contact us to coordinate access and avoid conflicts with other reviewers. Although our provided hardware does not exactly match the servers we used and there may be performance fluctuations between different runs, you should still observe similar performance trends.

Below are the detailed instructions.

## 1. Build and Install

Detailed instructions for building and installing our system are available in the following GitHub sections:

- [Environment Setup](https://github.com/uclasystem/DRust/tree/dev?tab=readme-ov-file#1-environment-setup)
- [Download and Install DRust](https://github.com/uclasystem/DRust/tree/dev?tab=readme-ov-file#2-download-and-install-drust)

An easier way is to try building it on our provided server. Ensure you're on zion-1 and follow these steps:

```bash
# First clean previous build
cd ~/DRust_home/DRust/comm-lib
make clean
cd ~/DRust_home/DRust/drust
cargo clean

# Compile communication static library
cd ~/DRust_home/DRust/comm-lib
make -j lib

# Copy it drust folder
cp libmyrdma.a ../drust/

# Compile the rust part
cd ~/DRust_home/DRust/drust
cargo build --release
```

## 2. Run DRust

Note that the following running process may take several hours, so we recommend reviewers to run our all-in-one script in tmux:

```bash
# switch to the directory
cd ~/DRust_home/aescripts

# launch tmux
tmux

# run our all-in-one script
bash run_all_drust.sh
```

After that, you can safely detach the tmux window and close the terminal. This process 

### 2.1 Dataframe

Run the following command to automatically run dataframe on 1 server to 8 servers.

```bash
cd ~/DRust_home/aescripts
bash dataframe.sh 2>&1 | tee df.log
```

You can see all the results (e.g. `dataframe_drust_8.txt`) in `~/DRust_home/logs`:
```
ls ~/DRust_home/logs
```

### 2.2 GEMM

Run the following command to automatically run GEMM on 1 server to 8 servers.

```bash
cd ~/DRust_home/aescripts
bash gemm.sh 2>&1 | tee ge.log
```

You can see all the results (e.g. `gemm_drust_8.txt`) in `~/DRust_home/logs`:

```bash
ls ~/DRust_home/logs
```


### 2.3 KVStore

Run the following command to automatically run KVStore on 1 server to 8 servers.

```bash
cd ~/DRust_home/aescripts
bash kv.sh 2>&1 | tee kv.log
```

You can see all the results (e.g. `kv_drust_8.txt`) in `~/DRust_home/logs`:

```bash
ls ~/DRust_home/logs
```

### 2.4 SocialNet

Run the following command to automatically run socialnet on 1 server to 8 servers.

```bash
cd ~/DRust_home/aescripts
bash sn.sh 2>&1 | tee sn.log
```

You can see all the results (e.g. `sn_drust_8.txt`) in `~/DRust_home/logs`:

```bash
ls ~/DRust_home/logs
```


## 3. Run Baseline System

The configuration and running of the baseline systems requires much effort. To make this process more smooth, we prepared scripts for reviewers to run GAM to compare with DRust. GAM is the baseline system we used in our evaluation which has better performance that the other baseline system Grappa.

Besides, one important note. The baseline system GAM itself has some bugs which may randomly crash or hangs for very long time. So you may need to frequently check the running status. And if it crashes or stucks, please kill the system and rerun it.



## 4. Generate Plots

To generate performance figures, make sure that you the running of the scripts on all configurations is done. Execute the following python scripts to generate the plots:

```bash
cd ~/DRust_home/aescripts
python3 plot.py dataframe
python3 plot.py gemm
python3 plot.py kv
python3 plot.py sn
```

The generated plots(e.g. `dataframe_performance.pdf`) are located at `~/DRust_home/aescripts/figures/`


```bash
ls ~/DRust_home/aescripts/figures
```
