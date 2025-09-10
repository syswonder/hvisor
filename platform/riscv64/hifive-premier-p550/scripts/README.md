
Note: `boot_zone1.sh` is an example usage with `hvisor-tool`

`boot_zone1.sh` will start virtio backend and boot zone1, please don't create backend repeatedly.

If you want to reattch the same one virtio-console, please use `screen -r`.

One example is below:

```bash
# boot zone1, in this script insmod hvisor.ko
./boot_zone1.sh

# list all zones
./hvisor zone list

# attach zone1's virtio-console, then use ctrl+a+d to return to zone0's terminal
screen -S zone1 /dev/pts/0 

# shutdown zone1
./hvisor zone shutdown -id 1

# restart zone1
./hvisor zone start zone1-linux.json

# reattach to virtio-console
screen -r zone1
```