# GPU

This sub-module of the project analyzing the GPU hardware on a IT equipment,
and providing information about the types of detected devices.

## Collected metrics

Currently, we check the number of NVIDIA GPU devices, identifying them,
and retrieves for each of them their associated data:

* Gpu architecture.
* Gpu PCIe bus device identification.
* Gpu full name.
* Gpu clock concerning graphic in MHz.
* Gpu clock concerning memory in MHz.
* Gpu clock concerning streaming multiprocessor in MHz.
* Gpu clock concerning video in MHz.
* Gpu fans speeds in %.
* Gpu temperature in °C.
* Gpu memory free in GB.
* Gpu memory total in GB.
* Gpu memory usage in GB.
* Gpu memory usage in %.
* Gpu global usage in %.
* Gpu energy consumption in J.
* Gpu power consumption in W.
* Gpu limit device power consumption in W.
* Gpu PCIe bus received data consumption in MB.
* Gpu PCIe bus sent data consumption in MB.

Also, we listing the running processes on a GPU device, and retrieves for each
of them their associated data:

* Process identification by its attributed PID on the system.
* Process video decoding tasks in %.
* Process video encoding tasks in %.
* Process memory utilization in %.
* Process streaming multiprocessor utilization in %.

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
