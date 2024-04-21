#!bin/sh

wid=$1
echo "Running workload $wid"

ssh guest@zion-11.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 7 -a $wid" &
ssh guest@zion-10.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 6 -a $wid" &
ssh guest@zion-6.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 5 -a $wid" &
ssh guest@zion-5.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 4 -a $wid" &
ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
sleep 2
ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &