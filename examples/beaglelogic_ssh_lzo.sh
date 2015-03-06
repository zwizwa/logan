FILE=$(mktemp beaglelogic_XXXX.lzo)
echo $FILE
ssh root@beaglebone "lzop </dev/beaglelogic" | pv >$FILE


