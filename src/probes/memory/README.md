# Memory

This sub-module of the project analyzing the memory hardware on a IT equipment,
and providing information about the types of detected memories.

## Requirements

* Linux operating system.
* Root permissions to have DMI files access available.

## Collected metrics

With `dmidecode` crate, we open DMI file system and decode it to collect the
following metrics concerning the memory:

|Name|Unity|Description|
|----|-----|-----------|
|`bandwidth_read`|megabyte/second|Test bandwidth for reading bytes|
|`bandwidth_write`|megabyte/second|Test bandwidth for writing bytes|
|`ram_available`|megabyte|Available RAM memory|
|`ram_free`|watt|Free RAM memory|
|`ram_power_consumption`|megabyte|Power consumed by memory devices|
|`ram_total`|megabyte|RAM memory usage|
|`ram_types`|megabyte|Detected hardware device memory|
|`swap_free`|megabyte|Free SWAP memory|
|`swap_total`|megabyte|Total SWAP memory|
|`swap_usage`|megabyte|Total SWAP usage|

For each memory device module, we collect these metrics:

|Name|Unity|Description|
|----|-----|-----------|
|`kind`|megabyte|Type of a memory device|
|`id`|none|Serial number of a memory device|
|`voltage`|millivolt|Configured voltage of a memory device|
|`size`|megabyte|Size of a memory device|
|`speed`|megabyte|Size of a memory device|

## Details

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
* [Kiatoo - DDR2/DDR3/DDR4/DDR5 Comparison (fr)](https://www.kiatoo.com/blog/ddr2-vs-ddr3-vs-ddr4-vs-ddr5/)
* [Granite River Labs - Overview DDR Standards](https://graniteriverlabs.com/technology/ddr/)
* [Reddit - Power consumption of RAM modules](https://www.reddit.com/r/buildapc/comments/7w3m2g/ram_power_consumption/)

Power values are indicative and may vary depending on manufacturer, frequency,
and module density.

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

According that, we can estimate the computing memory power consumption with the
following formulas:

### Energy calculated according voltage variation of the computing memory type

```math
E_\text{i} = E_\text{ref} \times \left( \frac{V_\text{i}}{V_\text{ref}} \right)
```

### Multiplying by the present computing memory devices

```math
P_\text{i} = E_\text{i} \times S_\text{i}
```

### Estimated power consumption according RAM devices

We sum for all the computing memory devices, and we multiply by the fraction
of used RAM. So we have finally :

```math
P_\text{est} = \left( \sum_{i=1}^{N} \left[ E_{\text{ref},i} \times
\frac{V_i}{V_{\text{ref},i}} \times S_i \right] \right) \times \frac{U}{S_{\text{tot}}}
```

* $`P_\text{est}`$ : Total consumed power estimated by memory.
* $`N`$ : Number of memory modules.
* $`E_{\text{ref},i}`$ : Reference power (in W/GB) for the type of the module $i$.
* $`V_i`$ : Voltage of module $i$.
* $`V_{\text{ref},i}`$ : Reference voltage for the type of the module $i$.
* $`S_i`$ : Size (in GB) for the module $i$.
* $`U`$ : Amount of really used memory (in GB).
* $`S_{\text{tot}}`$ : Total size of the memory device installed ($`S_{\text{tot}} = \sum_{i=1}^N S_i`$).

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
