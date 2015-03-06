FILE=$(mktemp beaglelogic_XXXX.lz4)
echo $FILE
ssh root@beaglebone lz4 /dev/beaglelogic | pv >$FILE


