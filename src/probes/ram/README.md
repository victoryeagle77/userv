# RAM

This sub-module of the project analyzing the memory hardware on a IT equipment,
and providing information about the types of detected memories.

## Collected metrics

Currently, we collect the following metrics concerning the RAM components:

* `bandwidth_read` : Test bandwidth for reading bytes in MB/s.
* `bandwidth_write` : Test bandwidth for writing bytes in MB/s.
* `ram_available` : Available RAM memory in MB.
* `ram_free` : Free RAM memory in MB.
* `ram_power_consumption` : Power consumption according the type of detected memory in W.
* `ram_total` : Total RAM memory available in MB.
* `ram_types` : List of detected hardware device memory.
* `ram_total` : RAM memory usage in MB.
* `swap_free` : Free SWAP memory in MB.
* `swap_total` : Total SWAP memory in MB.
* `swap_usage` : Total SWAP usage in MB.

## Usage

To run the program to retrieve the information from the RAM,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active ram
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by the probes:

```bash
./userv --active ram --freq 5
```
