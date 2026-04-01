#!/bin/bash

# Define a function to process dmesg output
extract_dmesg() {
    local output_file="$1" # The first parameter is output file path

    # Capture dmesg output
    local dmesg_output=$(dmesg)

    # Process output with awk
    echo "$dmesg_output" | awk '
    BEGIN {
        RS="\n"; # Set record separator to newline
    }
    {
        # Remove leading [] timestamps
        sub(/^\[[^]]*\] /, "")
        # Store processed lines in array
        lines[NR] = $0
    }
    END {
        # Initialize counters and output arrays
        count = 0
        output_lines[1] = ""
        output_lines[2] = ""
        # Traverse from last line backwards
        for (i = NR; i > 0; i--) {
            if (lines[i] !~ /random: fast init done/) {
                # If line does not contain - random: fast init done -
                if (count < 2) {
                    # Store line in output array
                    output_lines[2-count] = lines[i]
                    count++
                }
            }
            # Break loop when count reaches 2
            if (count >= 2) {
                break
            }
        }
        # Output lines in correct order
        if (output_lines[1] != "") {
            printf "%s\n", output_lines[1]
        }
        if (output_lines[2] != "") {
            printf "%s\n", output_lines[2]
        }
    }
    ' > "$output_file"
}

# Call function with output file path
extract_dmesg "$1"