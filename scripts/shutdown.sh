
for i in {1,2,3,4,5,6,10,11}; do
    echo "Kill executable on node$i"
    ssh guest@zion-$i.cs.ucla.edu 'pkill drust'
done