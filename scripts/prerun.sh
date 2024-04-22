
for i in {1,2,3,4,5,6,10,11}; do
    echo "Init logs on node$i"
    ssh guest@zion-$i.cs.ucla.edu "cd ~/DRust_home; rm -rf logs; mkdir logs; cd DRust; git checkout dev; git pull origin dev"
done