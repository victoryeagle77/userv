# Memory

This sub-module of the project analyzing the memory hardware on a IT equipment,
and providing information about the types of detected memories.

## Collected metrics

Currently, we collect the following metrics concerning the memory component:

* `bandwidth_read` : Test bandwidth for reading bytes in MB/s.
* `bandwidth_write` : Test bandwidth for writing bytes in MB/s.
* `ram_available` : Available RAM memory in MB.
* `ram_free` : Free RAM memory in MB.
* `ram_power_consumption` : Power consumption retrieved according the detected
type of memory in W.
* `ram_total` : Total RAM memory available in MB.
* `ram_types` : List of detected hardware device memory.
* `ram_total` : RAM memory usage in MB.
* `swap_free` : Free SWAP memory in MB.
* `swap_total` : Total SWAP memory in MB.
* `swap_usage` : Total SWAP usage in MB.

## Precision

The power consumption estimation is based on a approximation per GB according
the memory type detected on a IT equipment, based on voltage specifications and
average module datasheets:

* [Wikipedia - SDRAM](https://en.wikipedia.org/wiki/Synchronous_dynamic_random-access_memory)
* [Crucial - DDR vs DDR2 vs DDR3 vs DDR4](https://www.crucial.fr/articles/about-memory/difference-between-ddr2-ddr3-ddr4)
* [Kingston - DDR2 vs DDR3](https://www.kingston.com/fr/blog/pc-performance/ddr2-vs-ddr3)
* [Crucial - DDR3 Power Consumption](https://www.crucial.com/articles/about-memory/power-consumption-of-ddr3)
* [FS.com - DDR3 vs DDR4 vs DDR5](https://community.fs.com/blog/ddr3-vs-ddr4-vs-ddr5.html)
* [Tom's Hardware - DDR5 vs DDR4 Power](https://www.tomshardware.com/news/ddr5-vs-ddr4-ram)
* [Micron - LPDDR2/LPDDR3 Power](https://www.micron.com/products/dram/lpdram)
* [Logic-fruit - DDR3 vs DDR4 vs LPDDR4](https://www.logic-fruit.com/blogs/ddr3-vs-ddr4-vs-lpddr4/)
* [Samsung - LPDDR5 Whitepaper](https://semiconductor.samsung.com/resources/white-paper/5th-generation-lpddr5/)
* [Micron - eMMC Power Consumption](https://media-www.micron.com/-/media/client/global/documents/products/technical-note/nand-flash/tn2961_emmc_power_consumption.pdf)
* [Kiatoo - DDR2/DDR3/DDR4/DDR5 Comparison (fr)](https://www.kiatoo.com/blog/ddr2-vs-ddr3-vs-ddr4-vs-ddr5/)
* [Granite River Labs - Overview DDR Standards](https://graniteriverlabs.com/technology/ddr/)
* [Reddit - Power consumption of RAM modules](https://www.reddit.com/r/buildapc/comments/7w3m2g/ram_power_consumption/)

Power values are indicative and may vary depending on manufacturer, frequency,
and module density.

```text
| Type     | Voltage   | Typical for 8GB | W/GB |
|----------|-----------|-----------------|------|
| SDRAM    | 3.3V      | 5.5W            | 0.70 |
| DDR      | 2.5V      | 5W              | 0.62 |
| DDR2     | 1.8V      | 3.8W            | 0.48 |
| DDR3     | 1.5V      | 3–4W            | 0.45 |
| DDR4     | 1.2V      | 2–3W            | 0.32 |
| DDR5     | 1.1V      | 1.5–2.5W        | 0.25 |
| LPDDR2   | 1.2V      | 1.5W            | 0.19 |
| LPDDR3   | 1.2V      | 1.3W            | 0.16 |
| LPDDR4   | 1.1V      | 1–1.5W          | 0.16 |
| LPDDR5   | 1.05V     | 0.8–1.2W        | 0.12 |
| eMMC     | 3.3V/1.8V | < 0.8W          | 0.10 |
```

According that, we can estimate the computing memory power consumption with the
following formula:

$$
E = (M_\text{total} \times P) \times \left( \frac{M_\text{used}}{M_\text{total}} \right)
$$

* `E` : Estimated power consumption by computing memory.
* `M` : Computing memory.
* `P` : Power defined by the previous table.

## Usage

To run the program to retrieve the information from the memory,
you can specify its corresponding probe in binary arguments.

```bash
./userv --active ram
```

In addition to this argument, you can add the `freq` parameter,
to set an acquisition interval per second for the data collected by this probe:

```bash
./userv --active ram --freq 5
```
