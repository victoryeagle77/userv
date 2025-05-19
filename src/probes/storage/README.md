# Storage devices

This sub-module of the project analyzing the storage hardware on a IT
equipment, and providing information about the types of detected devices.

## Collected metrics

Currently, we retrieve information about device storage (disks, SD card...)
of a IT equipment. For each device detected, we retrieve the following data:

* Bandwidth for reading bytes in MB.
* Bandwidth for writing bytes in MB.
* File system path where the device is mounted.
* File system format of the device (ext, NTF, FAT, etc...).
* Device kind (HDD or SSD).
* Path name of the device on the system.
* Available memory space in MB.
* Total memory space in MB.

If it's possible and available, we can get also smart information for a storage
device, which corresponding to more detailed data generally about disks:

* Reallocated sector count.
* Reallocation event count.
* Current pending sector count.
* Disk operating temperature.
* Power on hours.

## Usage

To run the program to retrieve the information from the storage devices,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active storage
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by the probes:

```bash
./userv --active storage --freq 5
```
