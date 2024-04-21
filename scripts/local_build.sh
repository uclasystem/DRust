cd ~/DRust_home/DRust/comm-lib
make clean
make -j lib
cp libmyrdma.a ../drust/
cd ~/DRust_home/DRust/drust
~/.cargo/bin/cargo build --release