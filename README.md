# Userv

This project involves analyzing the hardware of a computer system through a set
of microservices, and providing information on the energy consumption and
health status of the components using the collected data.

Version : 0.1.5

## Collected metrics

Currently, we collect the following data:

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

* Processor data

  * CPU cores usage in %.
  * CPU logical cores number.
  * CPU physical cores number.
  * CPU family foundation.
  * CPU clock frequency in MHz
  * CPU full model name.
  * Temperatures by identified CPU thermal zone in °C.
  * Power consumption in W (with RAPL domain zone analyze for INTEL).

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

* RAM memory data

  Retrieves data about the computing and SWAP memory of an IT equipment:

  * Available RAM in MB.
  * Free RAM in MB.
  * Total RAM available in MB.
  * RAM usage in MB.
  * Free SWAP memory in MB.
  * Total SWAP memory in MB.
  * Total SWAP usage in MB.
  * Test bandwidth for reading bytes in MB/s.
  * Test bandwidth for writing bytes in MB/s.

* Board data

  Retrieves data about the main board of the concerned IT equipment if are
  available:

  * Board name.
  * Board serial.
  * Board version.
  * Board vendor.
  * Bios date.
  * Bios release.
  * Bios version.
  * Bios vendor.

* Network data

  Retrieves all available network interfaces on a computer device and
  collects their associated data :

  * Interface MAC address.
  * Interface name.
  * Received data consumption in MB.
  * Transmitted data consumption in MB.

* System data

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
