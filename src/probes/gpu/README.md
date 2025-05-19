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
|`gpu_architecture`|none|Hardware architecture|
|`gpu_bus_id`|none|PCIe bus device identification|
|`gpu_clock_graphic`|megahertz|Clock concerning graphic unit|
|`gpu_clock_memory`|megahertz|Clock concerning memory unit|
|`gpu_clock_sm`|megahertz|Clock concerning streaming multiprocessor unit|
|`gpu_clock_video`|megahertz|Clock concerning video unit|
|`gpu_energy_consumption`|millijoule|Energy consumption|
|`gpu_name`|none|Full name|
|`gpu_usage`|percentage|GPU usage|
|`gpu_temperature`|celsius|Thermal zone value|
|`gpu_memory_free`|bytes|Memory free|
|`gpu_memory_stat`|percentage|Memory utilization|
|`gpu_memory_total`|bytes|Memory total|
|`gpu_memory_usage`|percentage|Memory usage|
|`gpu_pci_data_sent`|bytes/second|PCIe bus sent data consumption|
|`gpu_pci_data_received`|bytes/second|PCIe bus received data consumption|
|`gpu_power_consumption`|milliwatt|Power consumption|
|`gpu_power_ratio`|milliwatt|Ration concerning the consumed power on the maximum power consumption|

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
