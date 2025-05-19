# Board

This sub-module of the project analyzing the main board / motherboard hardware
on a IT equipment, and providing information about it.

## Collected metrics

Currently, we collect the following metrics concerning the board component,
if are available:

* `bios_date` : Bios release date.
* `bios_release` : Bios release version.
* `bios_version` : Bios software version.
* `bios_vendor` : Bios vendor name.
* `board_name` : Board full name.
* `board_serial` : Board serial number.
* `board_version` : Board hardware version.
* `board_vendor` : Board vendor name.

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
