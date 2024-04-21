#!bin/sh
#Deprecated, do not use this script


for i in {1,2,3}; do
    echo "Updating node$i"
    ssh guest@zion-$i.cs.ucla.edu 'cd ~/DRust_home/DRust; git pull origin dev; git reset --hard origin/dev; cd comm-lib; make clean; make -j lib; cp libmyrdma.a ../drust; cd ../drust; ~/.cargo/bin/cargo clean --release -p drust; ~/.cargo/bin/cargo build --release' &
done