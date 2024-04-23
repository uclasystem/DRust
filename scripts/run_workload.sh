#!bin/sh

wid=$1
num=$2
echo "Running workload $wid"

if [ $num -eq 8 ]; then
    echo "Running workload $wid on 8 servers"
    ssh guest@zion-11.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 7 -a $wid" &
    ssh guest@zion-10.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 6 -a $wid" &
    ssh guest@zion-6.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 5 -a $wid" &
    ssh guest@zion-5.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 4 -a $wid" &
    ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 7 ]; then
    echo "Running workload $wid on 7 servers"
    ssh guest@zion-10.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 6 -a $wid" &
    ssh guest@zion-6.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 5 -a $wid" &
    ssh guest@zion-5.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 4 -a $wid" &
    ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 6 ]; then
    echo "Running workload $wid on 6 servers"
    ssh guest@zion-6.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 5 -a $wid" &
    ssh guest@zion-5.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 4 -a $wid" &
    ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 5 ]; then
    echo "Running workload $wid on 5 servers"
    ssh guest@zion-5.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 4 -a $wid" &
    ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 4 ]; then
    echo "Running workload $wid on 4 servers"
    ssh guest@zion-4.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 3 -a $wid" &
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 3 ]; then
    echo "Running workload $wid on 3 servers"
    ssh guest@zion-3.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 2 -a $wid" &
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 2 ]; then
    echo "Running workload $wid on 2 servers"
    ssh guest@zion-2.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 1 -a $wid" &
    sleep 2
    ssh guest@zion-1.cs.ucla.edu "cd ~/DRust_home/DRust/drust; ./../drust.out -s 0 -a $wid" &
elif [ $num -eq 1 ]; then
    echo "Running workload $wid on 1 server"
    cd ~/DRust_home/DRust/drust 
    ./../drust.out -s 0 -a $wid &
fi

sleep 20

while true; do
    # Check if the process named 'drust' is running
    if pgrep -f "./../drust.out" > /dev/null; then
        # If it's running, sleep for 5 seconds
        sleep 5
    else
        # If it's not running, exit the loop
        break
    fi
done


echo "Process 'drust' is not running anymore. Exiting in 20 seconds."
sleep 20
