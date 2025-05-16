# Userv

This project involves analyzing the hardware of a computer equipment via a set
of micro-services, and providing information on the energy consumption and
health status of the components with the collected data.

Version : 0.1

## Collected metrics

Currently, we collect the following data :

* GPU data

* Processor data

* Storage devices data

* RAM memory data

  Retrieves in MB data above the computing and SWAP memory of an IT equipment :

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
  available :

  * board name.
  * board serial.
  * board version.
  * board vendor.
  * bios date.
  * bios release.
  * bios version.
  * bios vendor.

* Network data

  Retrieves all available network interfaces on a computer device and
  collects their associated data :

  * Interface MAC address.
  * Interface name.
  * Received data consumption in MB.
  * Transmitted data consumption in MB.

* System data

## Program usage

To run the program to analyze and retrieve all data about all available
components, you can currently run its binary like this :

```bash
./userv --all
```

To run the program precisely for only retrieves desired components information,
you can specifying a probe in binary arguments.

```bash
./userv --active probe_1,probe_2,...
```

You can select a `probe` to get data components with the program, among the following
probes list :

* board
* cpu
* gpu
* net
* ram
* storage
* system
