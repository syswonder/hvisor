screen_output=$(cat nohup.out | grep "char device")
echo "$screen_output"
device=$(echo "$screen_output" | awk '{print $NF}')
echo "$device"
screen -S screen_linux2 $device
