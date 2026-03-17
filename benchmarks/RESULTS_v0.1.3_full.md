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
| B1.1 Append N characters | 88733 ms | 1 bytes | 555 µs | 6002 bytes | 6008 bytes | 6002 bytes | 70 µs | 2.8 MB |
| B1.2 Insert string of length N | 33 ms | 6000 bytes | 391 µs | 6002 bytes | 6008 bytes | 6002 bytes | 70 µs | 2.8 MB |
| B1.3 Prepend N characters | 14311 ms | 1 bytes | 387 µs | 6002 bytes | 6008 bytes | 6002 bytes | 71 µs | 2.3 MB |
| B1.4 Insert N characters at random positions | 43738 ms | 1 bytes | 393 µs | 6002 bytes | 6008 bytes | 6002 bytes | 69 µs | 2.4 MB |
| B1.5 Insert N words at random positions | 226611 ms | 4 bytes | 1 ms | 29178 bytes | 29184 bytes | 29179 bytes | 306 µs | 16.3 MB |
| B1.6 Insert string, then delete it | 74 ms | 6000 bytes | 13 µs | 2 bytes | 8 bytes | 1 bytes | 11 µs | 3.1 MB |
| B1.7 Insert/Delete strings at random positions | 152556 ms | 3 bytes | 1 ms | 14829 bytes | 14835 bytes | 14829 bytes | 275 µs | 10.3 MB |
| B1.8 Append N numbers | 112475 ms | 8 bytes | 96 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 69 ms | 4.3 MB |
| B1.9 Insert Array of N numbers | 112574 ms | — | 92 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 68 ms | 4.3 MB |
| B1.10 Prepend N numbers | 19146 ms | 8 bytes | 96 ms | n/a (non-string keys) | 1758090 bytes | 1073466 bytes | 69 ms | 3.6 MB |
| B1.11 Insert N numbers at random positions | 57428 ms | 8 bytes | 91 ms | n/a (non-string keys) | 1758090 bytes | 1078858 bytes | 67 ms | 3.8 MB |

### B2

| Benchmark | time | avgUpdateSize | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B2.1 Concurrently insert string of length N at index 0 | 39 ms | 12200 bytes | 772 µs | 12102 bytes | 12108 bytes | 12102 bytes | 124 µs | 6.9 MB |
| B2.2 Concurrently insert N characters at random positions | 46 ms | 12200 bytes | 1 ms | 12102 bytes | 12108 bytes | 12102 bytes | 143 µs | 6.2 MB |
| B2.3 Concurrently insert N words at random positions | 406 ms | 58514 bytes | 5 ms | 58416 bytes | 58422 bytes | 58417 bytes | 926 µs | 43.4 MB |
| B2.4 Concurrently insert & delete | 180 ms | 30436 bytes | 3 ms | 30410 bytes | 30416 bytes | 30411 bytes | 295 µs | 25.6 MB |

### B3

| Benchmark | time | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B3.1 20*sqrt(N) clients concurrently set number in Map | 1080 ms | 5 ms | n/a (non-string keys) | 107843 bytes | 48926 bytes | 7 ms | 3.5 MB |
| B3.2 20*sqrt(N) clients concurrently set Object in Map | 4503 ms | 23 ms | n/a (non-string keys) | 488665 bytes | 257810 bytes | 27 ms | 7.5 MB |
| B3.3 20*sqrt(N) clients concurrently set String in Map | 2367 ms | 6 ms | n/a (non-string keys) | 1656843 bytes | 1588632 bytes | 7 ms | 8.0 MB |
| B3.4 20*sqrt(N) clients concurrently insert text in Array | 1238 ms | 22 ms | n/a (non-string keys) | 501556 bytes | 317172 bytes | 17 ms | 3.7 MB |
