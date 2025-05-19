# GPU

This sub-module of the project analyzing the GPU hardware on a IT equipment,
and providing information about the types of detected devices.

## Requirements

* Linux operating system.
* Linux nvidia driver installed.

## Collected metrics

Currently, we check the number of NVIDIA GPU devices, identifying them,
and retrieves for each of them their associated data:

|Name|Unity|Description|
|----|-----|-----------|
|`architecture`|none|Hardware architecture|
|`bus_id`|none|PCIe bus device identification|
|`clock_graphic`|megahertz|Clock concerning graphic unit|
|`clock_memory`|megahertz|Clock concerning memory unit|
|`clock_sm`|megahertz|Clock concerning streaming multiprocessor unit|
|`clock_video`|megahertz|Clock concerning video unit|
|`energy_consumption`|millijoule|Energy consumption|
|`fan_speed`|percentage|Fans speeds|
|`name`|none|Full name|
|`usage`|percentage|GPU usage|
|`temperature`|celsius|Thermal zone value|
|`memory_free`|bytes|Memory free|
|`memory_stat`|percentage|Memory utilization|
|`memory_total`|bytes|Memory total|
|`memory_usage`|percentage|Memory usage|
|`pci_data_sent`|bytes/second|PCIe bus sent data consumption|
|`pci_data_received`|bytes/second|PCIe bus received data consumption|
|`power_consumption`|milliwatt|Power consumption|
|`power_limit`|milliwatt|Limit power consumption|

Also, we listing the running processes on a GPU device, and retrieves for each
of them their associated data:

|Name|Unity|Description|
|----|-----|-----------|
|`pid`|none|Process identification by its attributed PID on the system|
|`decoding`|percentage|Process video decoding|
|`encoding`|percentage|Process video encoding tasks|
|`memory`|percentage|Process memory utilization|
|`streaming_multiprocessor`|percentage|Process streaming multiprocessor utilization|

## Usage

To run the program to retrieve the information from the GPU,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active gpu
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active gpu --freq 5
```
