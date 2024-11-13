# Network

This sub-module of the project analyzing the network hardware on a IT equipment,
and providing information about the detected interfaces.

## Collected metrics

Currently, we get all available network interfaces and collects their
associated data :

|Name|Description|Unity|
|----|-----------|-----|
|`address_mac`|Network interface MAC address|none|
|`energy_consumed`|Power consumption by network interface|watt|
|`name`|Network interface name|none|
|`network_type`|Name type of the network interface|none|
|`errors_received`|Network errors received|megabyte|
|`errors_transmitted`|Network errors transmitted|megabyte|
|`packet_received`|Number of incoming packets|megabyte|
|`packet_transmitted`|Number of outcome packets|megabyte|
|`received`|Received data consumption|megabyte|
|`transmitted`|Transmitted data consumption|megabyte|

## Details

The power consumption estimation is based on a approximation per gigabytes
according the detected network interface type (Ethernet, Wifi, Cellular,
InfiniBand), running on a IT equipment, based on a defined ratio in GB/Wh.

Those data coming from studies leaded by the ADEME, ARCEP and CNRS institutes
in 2020 until 2025 :

* [Evaluation de l'impact environnemental du numérique en France](https://librairie.ademe.fr/societe-et-politiques-publiques/7880-evaluation-de-l-impact-environnemental-du-numerique-en-france.html)
* [EVALUATION DE L’IMPACT ENVIRONNEMENTAL DU NUMERIQUE EN FRANCE ET ANALYSE PROSPECTIVE](https://www.arcep.fr/uploads/tx_gspublication/note-synthese-au-gouvernement-prospective-2030-2050_mars2023.pdf)
* [Etude ADEME – Arcep sur l’empreinte environnementale du numérique en 2020, 2030 et 2050](https://www.arcep.fr/la-regulation/grands-dossiers-thematiques-transverses/lempreinte-environnementale-du-numerique/etude-ademe-arcep-empreinte-environnemental-numerique-2020-2030-2050.html)
* [EVALUATION DE L’IMPACT ENVIRONNEMENTAL DU NUMERIQUE EN FRANCE](https://ecoresponsable.numerique.gouv.fr/docs/2024/etude-ademe-impacts-environnementaux-numerique.pdf)
* [NUMÉRIQUE ET ENVIRONNEMENT](https://www.ademe.fr/wp-content/uploads/2025/01/dossier-de-presse-numerique-et-environnement-090125.pdf)
* [ADEME](https://librairie.ademe.fr/consommer-autrement/5226-evaluation-de-l-impact-environnemental-du-numerique-en-france-et-analyse-prospective.html)
* [Evaluation de l'impact environnemental du numérique en France et analyse prospective](https://ecoresponsable.numerique.gouv.fr/actualites/actualisation-ademe-impact/)
* [Données complémentaires (analyses techniques, fiches constructeurs, études sectorielles)](https://www.afnic.fr/wp-media/uploads/2022/10/afnic-JCSA22-Arcep.pdf)

```text
| Type      | Estimated Ratio Wh/GB | Average Ratio | Main sources         |
|-----------|-----------------------|---------------|----------------------|
| ETHERNET  | 0.1-0.3 Wh/GB         | 0.2 Wh/GB     | ARCEP, CNRS, ADEME   |
| INFINBAND | 0.05–0.2 Wh/GB        | 0.1 Wh/GB     | HPC datasheet        |
| WIFI      | 0.2-0.6 Wh/GB         | 0.4 Wh/GB     | ARCEP, CNRS, ADEME   |
| 5G        | 0.5-2 Wh/GB           | 0.8 Wh/GB     | ARCEP, FFT           |
| 4G        | 0.5-2 Wh/GB           | 1.0 Wh/GB     | ARCEP, FFT           |
| 3G        | 5–70 Wh/GB            | 37.5 Wh/GB    | ARCEP, FFT           |
| 2G        | 15–150 Wh/GB          | 80 Wh/GB      | ARCEP, FFT           |
```

According that, we can estimate the power consumed by a network interface with
the following formulas:

### Transferred energy

```math
E_{\text{transferred}} = \frac{D_{\text{rx}} + D_{\text{tx}}}{1000}
\times R_{\text{interface}} \times R_{\text{traffic}}
```

* $`E_{\text{transferred}}`$ : Transfer energy (Wh)
* $`D_{\text{rx}}`$ : Received data (MB)
* $`D_{\text{tx}}`$ : Transmitted data (MB)
* $`R_{\text{traffic}}`$ : Ratio according the traffic type
* $`R_{\text{interface}}`$ : Network interface energy ratio (Wh/Gb)

### Energy Idle

```math
E_{\text{idle}} = P_{\text{idle}} \times t
```

* $`E_{\text{idle}}`$ : Energy idle (Wh)
* $`P_{\text{idle}}`$ : Power idle (W)
* $`t`$ : Observation duration (hours)

### Total Energy consumption

```math
E_{\text{total}} = E_{\text{transferred}} + E_{\text{idle}}
```

* $E_{\text{transferred}}`$ : Transferred energy (Wh)
* $P_{\text{idle}}`$ : Power idle (W)
* $E_{\text{total}}`$ : Total consumed energy (Wh)
* If no data are consumed, $E_{\text{transferred}} = 0$

### Average power consumption

```math
P_{\text{average}} = \frac{E_{\text{total}}}{t}
```

* $`P_{\text{average}}`$ : Average power (W)
* $`E_{\text{total}}`$ : Total consumed energy (Wh)
* $`t`$ : Observation duration (hours)

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

These commands will produce JSON files with all retrieved data about the
concerning components.
