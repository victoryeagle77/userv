# System

This sub-module of the project analyzing the operating system hardware on a IT
equipment,and providing information about detected operating system.

## Collected metrics

Currently, we collect the following metrics concerning the operating system:

* Operating system distribution name.
* Operating system distribution version.
* Operating system kernel info.
* Operating system load given for 1, 5 and 15 minutes.
* Total number of processes enabled on the system.
* Uptime given in days, hours, and minutes.

Also, we listing the running processes on CPU, and retrieves for each of them
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

## Usage

To run the program to retrieve the information from the system,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active system
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active system --freq 5
```
