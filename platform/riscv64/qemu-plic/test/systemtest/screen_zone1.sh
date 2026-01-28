# screen_output=$(grep "char device" nohup.out)
# echo "$screen_output"
# device=$(echo "$screen_output" | awk '{print $NF}')
# echo "$device"
# screen -S screen_linux2 $device
screen -S screen_linux2 /dev/pts/0