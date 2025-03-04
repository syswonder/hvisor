#!/bin/bash

# set -x
# Define two arrays to store filenames from two lists
testcase_file_list=(
    ./test/testcase/tc_ls.txt
    ./test/testcase/tc_pwd.txt
    ./test/testcase/tc_insmod.txt
    # ./test/testcase/tc_zone1_start.txt
    ./test/testcase/tc_zone1_ls.txt
    ./test/testcase/tc_zone_list2.txt
    # ./test/testcase/tc_zone_list1.txt
)

testresult_file_list=(
    ./test/testresult/test_ls.txt
    ./test/testresult/test_pwd.txt
    ./test/testresult/test_insmod.txt
    # ./test/testresult/test_zone1_start.txt
    ./test/testresult/test_zone1_ls.txt
    ./test/testresult/test_zone_list2.txt
    # ./test/testresult/test_zone_list1.txt
)

testcase_name_list=(
    ls_out
    pwd_out
    insmod_hvisor.ko
    # zone1_start_out
    zone1_start
    zone_list
    # zone1_shutdown
)

# Get the length of the file lists
testcase_file_list_len=${#testcase_file_list[@]}
testresult_file_list_len=${#testresult_file_list[@]}

# Check if the lengths of the two lists are equal
if [ "$testcase_file_list_len" -ne "$testresult_file_list_len" ]; then
    echo "Error: The length of the two file lists is not equal."
    exit 1  # Return error status code 1
fi

fail_count=0
# Loop through the file lists
for ((i = 0; i < testcase_file_list_len; i++)); do
    # Get the ith filename from the lists
    testcase_file=${testcase_file_list[i]}
    testresult_file=${testresult_file_list[i]}
    testcase_name=${testcase_name_list[i]}

    # Send the diff command and wait for it to complete
    diff "$testcase_file" "$testresult_file"
    exit_status=$?

    # Output the result based on the exit status
    if [ "$exit_status" -eq 0 ]; then
        echo "$testcase_name $testresult_file PASS" >> ./test/result.txt
    else
        fail_count=$((fail_count+1))  # Increment fail_count
        echo "$testcase_name $testresult_file FAIL" >> ./test/result.txt
    fi
done


cat ./test/result.txt
# Format the output file content
printf "\n%-17s | %-40s | %s\n" "test name" "test result file" "result"
# Read the file content
while IFS= read -r line; do
    # Use regex to extract the test case name and result
    if [[ $line =~ ([^[:space:]]+)\ +(.*)\ +([A-Z]+)$ ]]; then
        testname=${BASH_REMATCH[1]}
        testcase=${BASH_REMATCH[2]}
        result=${BASH_REMATCH[3]}
        
        # Format the output
        printf "%-17s | %-40s | %s\n" "$testname" "$testcase" "$result"
    fi
done < "./test/result.txt"
printf "\n"

# Delete the generated files
rm -v ./test/testresult/test_*.txt
rm -v ./test/result.txt

# Check if failcount is greater than 0
if [ "$fail_count" -gt 0 ]; then
    echo "Error: Test fail. Exiting script."
    exit 1  # Exit with error, return status code 1
else
    echo "All tests passed. Script is exiting normally."
    exit 0  # Exit normally, return status code 0
fi