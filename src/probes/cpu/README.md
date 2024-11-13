# CPU

This sub-module of the project analyzing the CPU hardware on a IT equipment,
and providing information about the detected processor.

## Requirements

* Linux operating system.
* Root permissions to have RAPL files access available for INTEL/AMD CPU models.

## Collected metrics

We collect following metrics about generic CPU information:

|Name|Unity|Description|
|----|-----|-----------|
|`architecture`|none|CPU architecture name and version|
|`cores_logic`|none|CPU logical cores number|
|`cores_physic`|none|CPU physical cores number|
|`family`|none|CPU family foundation|
|`frequency`|megahertz|CPU clock frequency|
|`model`|none|CPU full model name|

CPU cores usage information:

|Name|Unity|Description|
|----|-----|-----------|
|`core_name`|none|CPU cores index|
|`usage`|percentage|CPU cores usage|

CPU power consumption information:

|Name|Unity|Description|
|----|-----|-----------|
|`zone_name`|none||
|`power`|watt|CPU power consumption (with RAPL domain zone analyze for INTEL)|

CPU temperatures information by identified thermal zone:

|Name|Unity|Description|
|----|-----|-----------|
|`zone_name`|none|CPU thermal zone|
|`temperature`|celsius|CPU CPU temperature|

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
