# Userv

This project involves analyzing the hardware of a computer system through a set
of microservices, and providing information on the energy consumption and
health status of the components using the collected data.

## Collected metrics

Currently, we collect metrics concerning the following components:

* CPU data
* GPU data
* Memory data
* Board data
* Network data
* Storage devices data
* System data

## Usage

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
* memory
* net
* storage
* system

In addition to these arguments, you can add the `freq` parameter to set an
acquisition interval per second for the data collected by the probes:

```bash
./userv --active probe_1,probe_2,... --freq 5
```

Or with all probes:

```bash
./userv --all --freq 5
```
