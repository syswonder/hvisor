#!/bin/bash

copyright_line="// Copyright (c) 2025 Syswonder"
ec=0

while read -r file; do
  first_line=$(head -n 1 "$file")
  
  if [[ "$first_line" != "$copyright_line" ]]; then
    echo "$file"
    ec=1
  fi
done < <(find . -type f -name "*.rs" \
  -not \( -path "./vendor/*" -prune \) \
  -not \( -path "./target/*" -prune \))

echo $ec
exit $ec