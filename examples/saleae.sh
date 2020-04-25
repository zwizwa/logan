#!/bin/bash
HERE=$(dirname $0)
export LD_LIBRARY_PATH=$HERE/SaleaeDeviceSdk-1.1.14/lib
$HERE/saleae.elf "$@"

