# README for OSDI 2024 Artifact Evaluation

Welcome to the artifact evaluation guide for our research artifact DRust. This README will guide you through the process of building, installing, running, and generating results from our system. The official README is also available in our [GitHub repository](https://github.com/uclasystem/DRust/).

## Overview

DRust is designed for distributed systems and requires multiple servers for optimal testing. This guide covers:

- Building and installing the system
- Running DRust and the baseline systems
- Generating performance plots and figures

## Getting Started

For initial testing, you can build and run the single-machine version of our tool on your local server. This setup doesn't require specialized hardware like InfiniBand. You can skip this step if you prefer to build our tool directly on the provided servers. To reproduce our key results, you'll need access to at least eight machines with InfiniBand, which we provide. To avoid conflicts with other reviewers, please contact us to coordinate server access. Although our provided hardware does not precisely match the servers we used for evaluation, but you should still observe similar performance trends.


## 1. Building and Installing DRust

Detailed instructions for building and installing DRust are available on GitHub:

- [Environment Setup](https://github.com/uclasystem/DRust/#1-environment-setup)
- [Download and Install DRust](https://github.com/uclasystem/DRust/#2-download-and-install-drust)

However, you can also build it on our provided servers. Ensure you're on zion-1 and follow these steps to build DRust:

```bash
# Clean previous builds
cd ~/DRust_home/DRust/comm-lib
make clean
cd ~/DRust_home/DRust/drust
cargo clean

# Compile communication static library
cd ~/DRust_home/DRust/comm-lib
make -j lib

# Copy the compiled library to the drust folder
cp libmyrdma.a ../drust/

# Build DRust
cd ~/DRust_home/DRust/drust
cargo build --release
```

## 2. Running DRust

To run DRust, we provide an all-in-one script that runs four applications with all configurations. Note that this process may take several hours, so we recommend running it in tmux to avoid interruptions:

```bash
# Move to the appropriate directory
cd ~/DRust_home/aescripts

# Start tmux
tmux

# Run the all-in-one script
bash run_all_drust.sh
```

After the script starts, you can safely detach from tmux and wait for it to finish. Performance results will be generated in ~/DRust_home/logs. If you want directly visualize the results for DRust, you can go to Section 5 for instructions to generate the performance figues.

Once the script starts, you can safely detach from tmux and allow the process to complete. Performance results will be generated in `~/DRust_home/logs`. To directly visualize the results for DRust after the script finishes, refer to Section 5 for instructions on generating performance figures.

### Running Specific Applications

You can also run specific applications separately using the scripts provided in this section.

#### 2.1 Dataframe

To run Dataframe on 1 server to 8 servers, use the following command:

```bash
cd ~/DRust_home/aescripts
bash dataframe.sh 2>&1 | tee df.log
```

To view all Dataframe results (e.g. `dataframe_drust_8.txt`), check the ~/DRust_home/logs directory:

```
ls ~/DRust_home/logs
```

#### 2.2 GEMM

To run GEMM on 1 server to 8 servers, use the following command:

```bash
cd ~/DRust_home/aescripts
bash gemm.sh 2>&1 | tee ge.log
```

To view all GEMM results (e.g. `gemm_drust_8.txt`), check the ~/DRust_home/logs directory:

```bash
ls ~/DRust_home/logs
```


#### 2.3 KVStore

To run KVStore on 1 server to 8 servers, use the following command:

```bash
cd ~/DRust_home/aescripts
bash kv.sh 2>&1 | tee kv.log
```

To view all KVStore results (e.g. `kv_drust_8.txt`), check the ~/DRust_home/logs directory:

```bash
ls ~/DRust_home/logs
```

#### 2.4 SocialNet

To run SocialNet on 1 server to 8 servers, use the following command:

```bash
cd ~/DRust_home/aescripts
bash sn.sh 2>&1 | tee sn.log
```

To view all SocialNet results (e.g. `sn_drust_8.txt`), check the ~/DRust_home/logs directory:

```bash
ls ~/DRust_home/logs
```


## 3. Run Baseline System GAM


To streamline the process of running the baseline system GAM for comparisons with DRust, we have prepared scripts that reviewers can use. GAM is a baseline system we used for evaluation, which typically significantly outperforms the other baseline system Grappa. Despite its higher performance, GAM can be unstable, so it's important to follow the steps and precautions outlined here.

### Important Notes About GAM

- Random Crashes and Long Hangs: GAM has known issues where it may crash or hang unexpectedly for long periods. To handle this, you might need to regularly check its status. If it crashes or gets stuck, kill the process and restart.
- Performance Instability: GAM's performance can be inconsistent, leading to significant fluctuations. In our evaluation, we ran the system multiple times to calculate the average throughput of each application. Expect similar variability if you run it yourself.

### Pre-Collected Performance Data

We have already collected performance data for GAM, stored in ~/DRust_home/logs/baseline. This can be used for comparison with DRust. Reviewers are welcome to use this pre-collected data or run GAM themselves for additional testing.

### Running All Applications on GAM

If you want to run all applications on GAM, you can use the following all-in-one script:

```bash
# Move to the appropriate directory
cd ~/DRust_home/aescripts

# Start tmux
tmux

# Run the all-in-one script
bash run_all_gam.sh
```

### Running Individual Applications on GAM

In addition to running all applications at once, you can run individual applications with GAM. This might be useful if you want to isolate specific tests or encounter issues when running all applications together. Follow the steps below to run each application separately:

#### 3.1 Dataframe

To run Dataframe with GAM on 1 server to 8 servers, use this command:

```bash
cd ~/DRust_home/aescripts
bash dataframe_gam.sh 2>&1 | tee df_gam.log
```

The results (`dataframe_gam_8.txt`) will be stored in `~/DRust_home/logs`. You can view the logs by listing the directory:

```bash
ls ~/DRust_home/logs
```

#### 3.2 GEMM

To run GEMM with GAM on 1 server to 8 servers, use this command:

```bash
cd ~/DRust_home/aescripts
bash gemm_gam.sh 2>&1 | tee gemm_gam.log
```

The results (`gemm_gam_8.txt`) will be stored in `~/DRust_home/logs`. You can view the logs by listing the directory:

```bash
ls ~/DRust_home/logs
```

#### 3.3 KVStore

To run KVStore with GAM on 1 server to 8 servers, use this command:

```bash
cd ~/DRust_home/aescripts
bash kv_gam.sh 2>&1 | tee kv_gam.log
```

The results (`kv_gam_8.txt`) will be stored in `~/DRust_home/logs`. You can view the logs by listing the directory:

```bash
ls ~/DRust_home/logs
```

#### 3.4 SocialNet

To run SocialNet with GAM on 1 server to 8 servers, use this command:

```bash
cd ~/DRust_home/aescripts
bash sn_gam.sh 2>&1 | tee sn_gam.log
```

The results (`sn_gam_8.txt`) will be stored in `~/DRust_home/logs`. You can view the logs by listing the directory:

```bash
ls ~/DRust_home/logs
```

## 4. Run Non-DSM Applications:

We have already collected performance numbers for the non-DSM versions of each application. These are located in `~/DRust_home/logs/baseline` and can be used for comparison with DRust. Reviewers can also run these applications by following the steps below.


```bash
# Navigate to the script directory
cd ~/DRust_home/aescripts

# Open a new tmux session
tmux

# Run the all-in-one script
bash run_all_single.sh
```

The results (`dataframe_single.txt`) will be stored in `~/DRust_home/logs`.


## 5. Generate Plots

To generate performance plots, ensure no other scripts are currently running. If you have only run the DRust system, you can generate the plots using pre-computed GAM and Non-DSM application logs with the following Python commands:

```bash
cd ~/DRust_home/aescripts
python3 plot.py dataframe
python3 plot.py gemm
python3 plot.py kv
python3 plot.py sn
```

The generated plots (e.g., `dataframe_performance.pdf`) will be located in `~/DRust_home/aescripts/figures`.

If you have run GAM and Non-DSM applications yourself and want to visualize the performance using your own time logs, use these commands to generate the plots:

```bash
cd ~/DRust_home/aescripts
python3 plot.py dataframe all
python3 plot.py gemm all
python3 plot.py kv all
python3 plot.py sn all
```

The generated plots (e.g., `dataframe_performance.pdf`) will also be located in `~/DRust_home/aescripts/figures`. To list them, use:

```bash
ls ~/DRust_home/aescripts/figures
```