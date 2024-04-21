cd ~/DRust_home/DRust/comm-lib
make clean
make -j lib
cp libmyrdma.a ../drust/
cd ~/DRust_home/DRust/drust
~/.cargo/bin/cargo build --release
for i in {1,2,3,4,5,6,10,11}; do
    echo "Updating node$i"
    scp ~/DRust_home/DRust/target/release/drust guest@zion-$i.cs.ucla.edu:DRust_home/DRust/drust.out
done