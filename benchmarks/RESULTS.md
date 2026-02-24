# MDCS Benchmark Results

> N = 6000 (default). Generated from raw benchmark log.

Comparable to [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks) output.

### B1

| Benchmark | time | avgUpdateSize | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B1.1 Append N characters | 11531 ms | 1 bytes | 97 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.8 MB |
| B1.2 Insert string of length N | 9 ms | 6000 bytes | 10 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.8 MB |
| B1.3 Prepend N characters | 2126 ms | 1 bytes | 10 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.3 MB |
| B1.4 Insert N characters at random positions | 5706 ms | 1 bytes | 17 µs | 6002 bytes | 6008 bytes | 6002 bytes | 4 µs | 2.4 MB |
| B1.5 Insert N words at random positions | 37657 ms | 4 bytes | 69 µs | 29006 bytes | 29012 bytes | 29007 bytes | 14 µs | 16.3 MB |
| B1.6 Insert string, then delete it | 28 ms | 6000 bytes | 1 µs | 2 bytes | 8 bytes | 1 bytes | 1 µs | 3.1 MB |
| B1.7 Insert/Delete strings at random positions | 24580 ms | 4 bytes | 50 µs | 15611 bytes | 15617 bytes | 15611 bytes | 8 µs | 10.4 MB |
| B1.8 Append N numbers | 14846 ms | 8 bytes | 6 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 7 ms | 4.3 MB |
| B1.9 Insert Array of N numbers | 14761 ms | — | 6 ms | n/a (non-string keys) | 1758090 bytes | 1079337 bytes | 7 ms | 4.3 MB |
| B1.10 Prepend N numbers | 2984 ms | 8 bytes | 6 ms | n/a (non-string keys) | 1758090 bytes | 1073466 bytes | 7 ms | 3.6 MB |
| B1.11 Insert N numbers at random positions | 7214 ms | 8 bytes | 9 ms | n/a (non-string keys) | 1758090 bytes | 1078830 bytes | 7 ms | 3.8 MB |

### B2

| Benchmark | time | avgUpdateSize | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B2.1 Concurrently insert string of length N at index 0 | 14 ms | 12200 bytes | 30 µs | 12102 bytes | 12108 bytes | 12102 bytes | 7 µs | 6.9 MB |
| B2.2 Concurrently insert N characters at random positions | 14 ms | 12200 bytes | 298 µs | 12102 bytes | 12108 bytes | 12102 bytes | 6 µs | 6.2 MB |
| B2.3 Concurrently insert N words at random positions | 186 ms | 58828 bytes | 2 ms | 58730 bytes | 58736 bytes | 58731 bytes | 431 µs | 43.5 MB |
| B2.4 Concurrently insert & delete | 99 ms | 31080 bytes | 3 ms | 31052 bytes | 31058 bytes | 31053 bytes | 17 µs | 25.7 MB |

### B3

| Benchmark | time | encodeTime | docSize:json | docSize:bincode | docSize:postcard | parseTime | memUsed |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B3.1 20*sqrt(N) clients concurrently set number in Map | 400 ms | 474 µs | n/a (non-string keys) | 107843 bytes | 48926 bytes | 599 µs | 3.5 MB |
| B3.2 20*sqrt(N) clients concurrently set Object in Map | 1865 ms | 1 ms | n/a (non-string keys) | 488665 bytes | 257810 bytes | 2 ms | 7.5 MB |
| B3.3 20*sqrt(N) clients concurrently set String in Map | 638 ms | 1 ms | n/a (non-string keys) | 1656843 bytes | 1588632 bytes | 1 ms | 8.0 MB |
| B3.4 20*sqrt(N) clients concurrently insert text in Array | 538 ms | 2 ms | n/a (non-string keys) | 501582 bytes | 317198 bytes | 1 ms | 3.7 MB |
