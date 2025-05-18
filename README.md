# Userv

This project involves analyzing the hardware of a computer system through a set
of microservices, and providing information on the energy consumption and
health status of the components using the collected data.

## Collected metrics

Currently, we collect some metrics concerning the following components:

* CPU data

  * CPU cores usage in %.
  * CPU logical cores number.
  * CPU physical cores number.
  * CPU family foundation.
  * CPU clock frequency in MHz
  * CPU full model name.
  * Temperatures by identified CPU thermal zone in °C.
  * Power consumption in W (with RAPL domain zone analyze for INTEL).

* GPU data

  Check the number of GPU devices, identifying them, and retrieves for each of
  them (currently NVIDIA GPU) their associated data:

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

  List the running processes on a GPU device, and retrieves for each of them
  their associated data:

  * Process identification by its attributed PID on the system.
  * Process video decoding tasks in %.
  * Process video encoding tasks in %.
  * Process memory utilization in %.
  * Process streaming multiprocessor utilization in %.

* RAM memory data

* Board data

  Retrieves data about the main board of the concerned IT equipment if are
  available:

  * Board full name.
  * Board serial number.
  * Board hardware version.
  * Board vendor name.
  * Bios release date.
  * Bios release version.
  * Bios software version.
  * Bios vendor name.

* Network data

  Retrieves all available network interfaces on a computer device and
  collects their associated data :

  * Interface MAC address.
  * Interface name.
  * Received data consumption in MB.
  * Transmitted data consumption in MB.

* Storage devices data

  Retrieves information about device storage (disks, SD card, etc...) of an IT
  equipment. For each device detected, we retrieve the following data:

  * Bandwidth for reading bytes in MB.
  * Bandwidth for writing bytes in MB.
  * File system path where the device is mounted.
  * File system format of the device (ext, NTF, FAT, etc...).
  * Device kind (HDD or SSD).
  * Path name of the device on the system.
  * Available memory space in MB.
  * Total memory space in MB.

  If it's possible, we can get also smart information about device storage:

  * Reallocated sector count.
  * Reallocation event count.
  * Current pending sector count.
  * Disk operating temperature.
  * Power on hours.

* System data

  Retrieves all available data about the operating system where we run the
  program:

  * Operating system distribution name.
  * Operating system distribution version.
  * Operating system kernel info.
  * Operating system load given for 1, 5 and 15 minutes.
  * Total number of processes enabled on the system.
  * Uptime given in days, hours, and minutes.

  List the running processes on CPU, and retrieves for each of them
  their associated data:

  * CPU usage consumed by a process in %.
  * Disk reading data usage by a process in MB.
  * Disk writing data usage by a process in MB.
  * Memory usage consumed by a process in MB.
  * Virtual memory usage consumed by a process in MB.
  * Process name given by the system.
  * Process identification on the system by PID.
  * Time the process has been running in minutes.
  * System session used by a process.
  * System run status used by a process.

## Program utilization

To run the program to analyze and retrieve all data on all available components,
you can run its binary as follows:

```bash
./userv --all
```

To run the program precisely to retrieve only the information from the desired
components, you can specify a probe in binary arguments.

```bash
./userv --active probe_1,probe_2,...
```

You can select a probe to obtain data with the program, from the following
probe list:

* board
* cpu
* gpu
* net
* ram
* storage
* system

In addition to these arguments, you can add the `freq` parameter to set an
acquisition interval per second for the data collected by the probes:

```bash
./userv --active cpu,gpu --freq 5
```

Or with all probes:

```bash
./userv --all --freq 5
```
