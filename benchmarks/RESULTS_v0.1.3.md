# MDCS Benchmark Results

> N = 6000 (default). Generated from raw benchmark log.

Comparable to [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks) output.

**Notes**

> The tests are run on a Windows Laptop (WSL UBUNTU 22.04) with Ryzen 7 5800HS, 16GB 3200mhz SO-DIMM.

> **N=6000**
> MDCS-SDK `V0.1.2`

### B1

| Benchmark | time | avgUpdateSize | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B1.1 Append N characters | 13919 ms | 1 bytes | 92 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.8 MB |
| B1.2 Insert string of length N | 12 ms | 6000 bytes | 26 µs | 6002 bytes | 6008 bytes | 6002 bytes | 6 µs | 2.8 MB |
| B1.3 Prepend N characters | 2472 ms | 1 bytes | 23 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.3 MB |
| B1.4 Insert N characters at random positions | 7652 ms | 1 bytes | 20 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.4 MB |
| B1.5 Insert N words at random positions | 44097 ms | 4 bytes | 75 µs | 29236 bytes | 29242 bytes | 29237 bytes | 14 µs | 16.3 MB |
| B1.6 Insert string, then delete it | 21 ms | 6000 bytes | 1 µs | 2 bytes | 8 bytes | 1 bytes | 2 µs | 3.1 MB |
| B1.7 Insert/Delete strings at random positions | 26766 ms | 4 bytes | 33 µs | 15418 bytes | 15424 bytes | 15418 bytes | 7 µs | 10.4 MB |
| B1.8 Append N numbers | 17202 ms | 8 bytes | 7 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 8 ms | 4.3 MB |
| B1.9 Insert Array of N numbers | 16997 ms | — | 6 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 8 ms | 4.3 MB |
| B1.10 Prepend N numbers | 3554 ms | 8 bytes | 6 ms | n/a (non-string keys) | 1758090 bytes | 1073466 bytes | 8 ms | 3.6 MB |
| B1.11 Insert N numbers at random positions | 8429 ms | 8 bytes | 9 ms | n/a (non-string keys) | 1758090 bytes | 1078831 bytes | 10 ms | 3.8 MB |

### B2

| Benchmark | time | avgUpdateSize | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B2.1 Concurrently insert string of length N at index 0 | 14 ms | 12200 bytes | 58 µs | 12102 bytes | 12108 bytes | 12102 bytes | 10 µs | 6.9 MB |
| B2.2 Concurrently insert N characters at random positions | 22 ms | 12200 bytes | 373 µs | 12102 bytes | 12108 bytes | 12102 bytes | 11 µs | 6.2 MB |
| B2.3 Concurrently insert N words at random positions | 198 ms | 58968 bytes | 1 ms | 58870 bytes | 58876 bytes | 58871 bytes | 419 µs | 43.5 MB |
| B2.4 Concurrently insert & delete | 83 ms | 30816 bytes | 1 ms | 30781 bytes | 30787 bytes | 30782 bytes | 15 µs | 25.6 MB |

### B3

| Benchmark | time | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B3.1 20*sqrt(N) clients concurrently set number in Map | 429 ms | 461 µs | n/a (non-string keys) | 107843 bytes | 48926 bytes | 642 µs | 3.5 MB |
| B3.2 20*sqrt(N) clients concurrently set Object in Map | 2034 ms | 1 ms | n/a (non-string keys) | 488665 bytes | 257810 bytes | 2 ms | 7.5 MB |
| B3.3 20*sqrt(N) clients concurrently set String in Map | 702 ms | 791 µs | n/a (non-string keys) | 1656843 bytes | 1588632 bytes | 1 ms | 8.0 MB |
| B3.4 20*sqrt(N) clients concurrently insert text in Array | 628 ms | 1 ms | n/a (non-string keys) | 501585 bytes | 317201 bytes | 1 ms | 3.7 MB |
