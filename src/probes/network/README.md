# Network

This sub-module of the project analyzing the network hardware on a IT equipment,
and providing information about the detected interfaces.

## Collected metrics

Currently, we get all available network interfaces and collects their
associated data :

* Interface MAC address.
* Interface name.
* Received data consumption in MB.
* Transmitted data consumption in MB.
* Network errors received in MB.
* Network errors transmitted in MB.
* Number of incoming packets in MB.
* Number of outcoming packets in MB.

## Usage

To run the program to retrieve the information from network interfaces,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active net
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active net --freq 5
```
