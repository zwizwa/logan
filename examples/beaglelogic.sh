# Load the BeagleLogic device tree overlay:
echo BB-BEAGLELOGIC > /sys/devices/bone_capemgr.*/slots

# Load BeagleLogic
modprobe beaglelogic

# Allocate memory
echo 33554432 > /sys/devices/virtual/misc/beaglelogic/memalloc

# Set samplerate
echo 20000000 > /sys/devices/virtual/misc/beaglelogic/samplerate

# Set continuous mode
echo 1 > /sys/devices/virtual/misc/beaglelogic/triggerflags

