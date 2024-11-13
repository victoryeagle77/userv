# Board

This sub-module of the project analyzing the main board / motherboard hardware
on a IT equipment, and providing information about it.

## Requirements

* Linux operating system.
* Root permissions to have DMI files access available.

## Collected metrics

Currently, we collect the following metrics concerning the board component,
if are available:

|Name|Unity|Description|
|----|-----|-----------|
|`bios_date`|none|Bios release date|
|`bios_release`|none|Bios release version|
|`bios_version`|none|Bios software version|
|`bios_vendor`|none|Bios vendor name|
|`board_name`|none|Board full name|
|`board_serial`|none|Board serial number|
|`board_version`|none|Board hardware version|
|`board_vendor`|none|Board vendor name|

## Usage

To run the program to retrieve the information from the main board,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active board
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active board --freq 5
```
