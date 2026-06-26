#!/bin/sh
# SPDX-License-Identifier: MulanPSL-2.0
set -u

vmx_count="$(grep -Ec '(^flags|^Features).* (vmx|svm)( |$)' /proc/cpuinfo 2>/dev/null || echo 0)"
if [ "${vmx_count:-0}" -le 0 ]; then
    echo "nested virtualization CPU flag not found"
    exit 1
fi
if [ ! -e /dev/kvm ]; then
    echo "/dev/kvm not present"
    exit 1
fi
if [ ! -r /dev/kvm ] || [ ! -w /dev/kvm ]; then
    echo "/dev/kvm exists but is not readable/writable by this user"
    exit 1
fi
echo "nested virtualization appears available"
