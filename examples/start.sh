#!/bin/bash
#
# This is the main entry point used by lars.erl Dispatch happens based
# on some environment variables.  We exec socat here to ensure that
# closing of stdin will exit everything.

[ -z "$DEV" ] && echo "DEV not set (saleae, ...)">&2 && exit 1
[ -z "$TYPE" ] && echo "analyzer TYPE not set (uart, ...)">&2 && exit 1
case "$DEV" in
    saleae)
        INPUT=$(dirname $0)/saleae.sh
        ;;
    *)
        echo "DEV=$DEV unknown">&2
        exit 1
        ;;
esac

FILTER="$(dirname $0)/$TYPE.elf"
[ ! -x "$FILTER" ] && echo "TYPE=$TYPE unknown">&2 && exit 1

# Note that all input drivers need to exit when their stdin closes.
$INPUT | $FILTER



