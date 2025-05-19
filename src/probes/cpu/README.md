# CPU

This sub-module of the project analyzing the CPU hardware on a IT equipment,
and providing information about the detected processor.

## Collected metrics

Currently, we collect the following metrics concerning the CPU component:

* CPU cores usage in %.
* CPU logical cores number.
* CPU physical cores number.
* CPU family foundation.
* CPU clock frequency in MHz
* CPU full model name.
* Temperatures by identified CPU thermal zone in °C.
* Power consumption in W (with RAPL domain zone analyze for INTEL).

## Usage

To run the program to retrieve the information from the CPU,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active cpu
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active cpu --freq 5
```
