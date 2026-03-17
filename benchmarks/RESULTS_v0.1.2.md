# MDCS Benchmark Results

> N = 6000 (default). Generated from raw benchmark log.

Comparable to [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks) output.

**Notes**

> The tests are run on a Windows Laptop (WSL UBUNTU 22.04) with Ryzen 7 5800HS, 16GB 3200mhz SO-DIMM.

> **N=6000**
> MDCS-SDK `V0.1.2`

## Benchmarks

#### B1: No conflicts

Simulate two clients. One client modifies a text object and sends update
messages to the other client. We measure the time to perform the task (`time`),
the amount of data exchanged (`avgUpdateSize`), the size of the encoded document
after the task is performed (`docSize`), the time to parse the encoded document
(`parseTime`), and the memory used to hold the decoded document (`memUsed`).

#### B2: Two users producing conflicts

Simulate two clients. Both start with a synced text object containing 100
characters. Both clients modify the text object in a single transaction and then
send their changes to the other client. We measure the time to sync concurrent
changes into a single client (`time`), the size of the update messages
(`updateSize`), the size of the encoded document after the task is performed
(`docSize`), the time to parse the encoded document (`parseTime`), and the
memory used to hold the decoded document (`memUsed`).

#### B3: Many conflicts

Simulate `√N` concurrent actions. We measure the time to perform the task
and sync all clients (`time`), the size of the update messages (`updateSize`),
the size of the encoded document after the task is performed (`docSize`),
the time to parse the encoded document (`parseTime`), and the memory used to hold the decoded document (`memUsed`).
The logarithm of `N` was
chosen because `√N` concurrent actions may result in up to `√N^2 - 1`
conflicts (apply action 1: 0 conlict; apply action2: 1 conflict, apply action 2: 2 conflicts, ..).


## Only MDCS Benchmark

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


## Comparison (fetched from [CRDT-Benchmarks](https://github.com/dmonad/crdt-benchmarks) )

> **†** MDCS docSize for array/map types uses **bincode** encoding — serde\_json rejects non-string map keys present in `RGAList`/`JsonCrdt` internals. All other MDCS docSize values use JSON.
>
> MDCS encodeTime and parseTime cover all three encoders (json + bincode + postcard) summed, so they are not directly comparable to single-encoder JS timings.
>
> JS benchmarks run in Node.js on an unspecified Linux machine; MDCS runs compiled Rust (debug build) on WSL Ubuntu 22.04, Ryzen 7 5800HS, 16 GB.

| N = 6000 | [yjs](https://github.com/yjs/yjs) | [ywasm](https://github.com/y-crdt/y-crdt/tree/main/ywasm) | [loro](https://github.com/loro-dev/loro) | [automerge](https://github.com/automerge/automerge/) | **mdcs-sdk** |
| :- | -: | -: | -: | -: | -: |
| Version | 13.6.11 | 0.9.3 | 0.10.1 | 2.1.10 | **0.1.2** |
| Bundle size | 69,124 bytes | 677,667 bytes | 1,052,250 bytes | 1,737,571 bytes | **N/A (native)** |
| Bundle size (gzipped) | 20,100 bytes | 213,833 bytes | 399,276 bytes | 604,118 bytes | **N/A (native)** |
| [B1.1] Append N characters (time) | 188 ms | 154 ms | 120 ms | 365 ms | **11,531 ms** |
| [B1.1] Append N characters (avgUpdateSize) | 27 bytes | 27 bytes | 109 bytes | 121 bytes | **1 bytes** |
| [B1.1] Append N characters (encodeTime) | 1 ms | 1 ms | 1 ms | 7 ms | **< 1 ms** |
| [B1.1] Append N characters (docSize) | 6,031 bytes | 6,031 bytes | 6,162 bytes | 3,992 bytes | **6,002 bytes** |
| [B1.1] Append N characters (memUsed) | 0 B | 0 B | 0 B | 0 B | **2.8 MB** |
| [B1.1] Append N characters (parseTime) | 32 ms | 23 ms | 26 ms | 80 ms | **< 1 ms** |
| [B1.2] Insert string of length N (time) | 0 ms | 0 ms | 0 ms | 9 ms | **9 ms** |
| [B1.2] Insert string of length N (avgUpdateSize) | 6,031 bytes | 6,031 bytes | 6,107 bytes | 6,201 bytes | **6,000 bytes** |
| [B1.2] Insert string of length N (encodeTime) | 0 ms | 0 ms | 0 ms | 3 ms | **< 1 ms** |
| [B1.2] Insert string of length N (docSize) | 6,031 bytes | 6,031 bytes | 6,117 bytes | 3,974 bytes | **6,002 bytes** |
| [B1.2] Insert string of length N (memUsed) | 17.4 kB | 0 B | 0 B | 8.8 kB | **2.8 MB** |
| [B1.2] Insert string of length N (parseTime) | 27 ms | 34 ms | 29 ms | 47 ms | **< 1 ms** |
| [B1.3] Prepend N characters (time) | 119 ms | 23 ms | 81 ms | 307 ms | **2,126 ms** |
| [B1.3] Prepend N characters (avgUpdateSize) | 27 bytes | 27 bytes | 108 bytes | 116 bytes | **1 bytes** |
| [B1.3] Prepend N characters (encodeTime) | 3 ms | 0 ms | 10 ms | 5 ms | **< 1 ms** |
| [B1.3] Prepend N characters (docSize) | 6,041 bytes | 6,041 bytes | 12,125 bytes | 3,988 bytes | **6,002 bytes** |
| [B1.3] Prepend N characters (memUsed) | 919.9 kB | 8.3 kB | 26.3 kB | 0 B | **2.3 MB** |
| [B1.3] Prepend N characters (parseTime) | 93 ms | 31 ms | 26 ms | 63 ms | **< 1 ms** |
| [B1.4] Insert N characters at random positions (time) | 131 ms | 128 ms | 79 ms | 310 ms | **5,706 ms** |
| [B1.4] Insert N characters at random positions (avgUpdateSize) | 29 bytes | 29 bytes | 109 bytes | 121 bytes | **1 bytes** |
| [B1.4] Insert N characters at random positions (encodeTime) | 1 ms | 1 ms | 35 ms | 8 ms | **< 1 ms** |
| [B1.4] Insert N characters at random positions (docSize) | 29,554 bytes | 29,554 bytes | 35,401 bytes | 24,743 bytes | **6,002 bytes** |
| [B1.4] Insert N characters at random positions (memUsed) | 883.6 kB | 0 B | 0 B | 9 kB | **2.4 MB** |
| [B1.4] Insert N characters at random positions (parseTime) | 76 ms | 29 ms | 31 ms | 79 ms | **< 1 ms** |
| [B1.5] Insert N words at random positions (time) | 154 ms | 449 ms | 82 ms | 449 ms | **37,657 ms** |
| [B1.5] Insert N words at random positions (avgUpdateSize) | 36 bytes | 36 bytes | 117 bytes | 131 bytes | **4 bytes** |
| [B1.5] Insert N words at random positions (encodeTime) | 5 ms | 1 ms | 69 ms | 21 ms | **< 1 ms** |
| [B1.5] Insert N words at random positions (docSize) | 87,924 bytes | 87,924 bytes | 94,524 bytes | 96,203 bytes | **29,006 bytes** |
| [B1.5] Insert N words at random positions (memUsed) | 2.3 MB | 872 B | 2.1 kB | 0 B | **16.3 MB** |
| [B1.5] Insert N words at random positions (parseTime) | 92 ms | 34 ms | 31 ms | 143 ms | **< 1 ms** |
| [B1.6] Insert string, then delete it (time) | 1 ms | 1 ms | 2 ms | 22 ms | **28 ms** |
| [B1.6] Insert string, then delete it (avgUpdateSize) | 6,053 bytes | 6,053 bytes | 6,217 bytes | 6,338 bytes | **6,000 bytes** |
| [B1.6] Insert string, then delete it (encodeTime) | 0 ms | 0 ms | 0 ms | 3 ms | **< 1 ms** |
| [B1.6] Insert string, then delete it (docSize) | 38 bytes | 38 bytes | 6,120 bytes | 3,993 bytes | **2 bytes** |
| [B1.6] Insert string, then delete it (memUsed) | 0 B | 0 B | 0 B | 2 kB | **3.1 MB** |
| [B1.6] Insert string, then delete it (parseTime) | 44 ms | 28 ms | 27 ms | 37 ms | **< 1 ms** |
| [B1.7] Insert/Delete strings at random positions (time) | 158 ms | 141 ms | 98 ms | 389 ms | **24,580 ms** |
| [B1.7] Insert/Delete strings at random positions (avgUpdateSize) | 31 bytes | 31 bytes | 113 bytes | 135 bytes | **4 bytes** |
| [B1.7] Insert/Delete strings at random positions (encodeTime) | 8 ms | 1 ms | 17 ms | 19 ms | **< 1 ms** |
| [B1.7] Insert/Delete strings at random positions (docSize) | 28,377 bytes | 28,377 bytes | 50,836 bytes | 59,281 bytes | **15,611 bytes** |
| [B1.7] Insert/Delete strings at random positions (memUsed) | 1.4 MB | 632 B | 1.8 kB | 6 kB | **10.4 MB** |
| [B1.7] Insert/Delete strings at random positions (parseTime) | 117 ms | 31 ms | 25 ms | 111 ms | **< 1 ms** |
| [B1.8] Append N numbers (time) | 148 ms | 29 ms | 81 ms | 480 ms | **14,846 ms** |
| [B1.8] Append N numbers (avgUpdateSize) | 32 bytes | 32 bytes | 114 bytes | 125 bytes | **8 bytes** |
| [B1.8] Append N numbers (encodeTime) | 0 ms | 0 ms | 1 ms | 8 ms | **6 ms** |
| [B1.8] Append N numbers (docSize) | 35,634 bytes | 35,634 bytes | 35,719 bytes | 26,985 bytes | **1,758,090 bytes †** |
| [B1.8] Append N numbers (memUsed) | 0 B | 0 B | 0 B | 61.3 kB | **4.3 MB** |
| [B1.8] Append N numbers (parseTime) | 36 ms | 31 ms | 27 ms | 80 ms | **7 ms** |
| [B1.9] Insert Array of N numbers (time) | 1 ms | 2 ms | 9 ms | 38 ms | **14,761 ms** |
| [B1.9] Insert Array of N numbers (avgUpdateSize) | 35,657 bytes | 35,657 bytes | 35,735 bytes | 31,199 bytes | **—** |
| [B1.9] Insert Array of N numbers (encodeTime) | 1 ms | 0 ms | 1 ms | 5 ms | **6 ms** |
| [B1.9] Insert Array of N numbers (docSize) | 35,657 bytes | 35,657 bytes | 35,742 bytes | 26,953 bytes | **1,758,090 bytes †** |
| [B1.9] Insert Array of N numbers (memUsed) | 39.3 kB | 608 B | 2.4 kB | 61.6 kB | **4.3 MB** |
| [B1.9] Insert Array of N numbers (parseTime) | 33 ms | 26 ms | 22 ms | 53 ms | **7 ms** |
| [B1.10] Prepend N numbers (time) | 122 ms | 28 ms | 78 ms | 461 ms | **2,984 ms** |
| [B1.10] Prepend N numbers (avgUpdateSize) | 32 bytes | 36 bytes | 113 bytes | 120 bytes | **8 bytes** |
| [B1.10] Prepend N numbers (encodeTime) | 3 ms | 1 ms | 10 ms | 7 ms | **6 ms** |
| [B1.10] Prepend N numbers (docSize) | 35,665 bytes | 65,658 bytes | 41,748 bytes | 26,987 bytes | **1,758,090 bytes †** |
| [B1.10] Prepend N numbers (memUsed) | 1.8 MB | 168 kB | 119.5 kB | 61.5 kB | **3.6 MB** |
| [B1.10] Prepend N numbers (parseTime) | 96 ms | 31 ms | 32 ms | 77 ms | **7 ms** |
| [B1.11] Insert N numbers at random positions (time) | 134 ms | 144 ms | 78 ms | 433 ms | **7,214 ms** |
| [B1.11] Insert N numbers at random positions (avgUpdateSize) | 33 bytes | 34 bytes | 114 bytes | 125 bytes | **8 bytes** |
| [B1.11] Insert N numbers at random positions (encodeTime) | 1 ms | 1 ms | 37 ms | 9 ms | **9 ms** |
| [B1.11] Insert N numbers at random positions (docSize) | 59,136 bytes | 59,152 bytes | 65,016 bytes | 47,746 bytes | **1,758,090 bytes †** |
| [B1.11] Insert N numbers at random positions (memUsed) | 1.8 MB | 0 B | 0 B | 61.7 kB | **3.8 MB** |
| [B1.11] Insert N numbers at random positions (parseTime) | 80 ms | 34 ms | 36 ms | 93 ms | **7 ms** |
| [B2.1] Concurrently insert string of length N at index 0 (time) | 1 ms | 0 ms | 2 ms | 62 ms | **14 ms** |
| [B2.1] Concurrently insert string of length N at index 0 (updateSize) | 6,094 bytes | 6,094 bytes | 9,276 bytes | 9,499 bytes | **12,200 bytes** |
| [B2.1] Concurrently insert string of length N at index 0 (encodeTime) | 0 ms | 0 ms | 0 ms | 5 ms | **< 1 ms** |
| [B2.1] Concurrently insert string of length N at index 0 (docSize) | 12,152 bytes | 12,151 bytes | 12,248 bytes | 8,011 bytes | **12,102 bytes** |
| [B2.1] Concurrently insert string of length N at index 0 (memUsed) | 0 B | 592 B | 6.4 kB | 14.5 kB | **6.9 MB** |
| [B2.1] Concurrently insert string of length N at index 0 (parseTime) | 43 ms | 27 ms | 25 ms | 47 ms | **< 1 ms** |
| [B2.2] Concurrently insert N characters at random positions (time) | 65 ms | 365 ms | 83 ms | 287 ms | **14 ms** |
| [B2.2] Concurrently insert N characters at random positions (updateSize) | 33,444 bytes | 177,007 bytes | 35,554 bytes | 27,476 bytes | **12,200 bytes** |
| [B2.2] Concurrently insert N characters at random positions (encodeTime) | 2 ms | 1 ms | 82 ms | 9 ms | **< 1 ms** |
| [B2.2] Concurrently insert N characters at random positions (docSize) | 66,852 bytes | 66,860 bytes | 71,858 bytes | 50,683 bytes | **12,102 bytes** |
| [B2.2] Concurrently insert N characters at random positions (memUsed) | 2.4 MB | 392 B | 1.8 kB | 0 B | **6.2 MB** |
| [B2.2] Concurrently insert N characters at random positions (parseTime) | 101 ms | 34 ms | 30 ms | 53 ms | **< 1 ms** |
| [B2.3] Concurrently insert N words at random positions (time) | 85 ms | 1,014 ms | 112 ms | 663 ms | **186 ms** |
| [B2.3] Concurrently insert N words at random positions (updateSize) | 88,994 bytes | 215,213 bytes | 93,132 bytes | 122,485 bytes | **58,828 bytes** |
| [B2.3] Concurrently insert N words at random positions (encodeTime) | 4 ms | 4 ms | 145 ms | 38 ms | **2 ms** |
| [B2.3] Concurrently insert N words at random positions (docSize) | 178,137 bytes | 178,130 bytes | 188,458 bytes | 185,019 bytes | **58,730 bytes** |
| [B2.3] Concurrently insert N words at random positions (memUsed) | 5.5 MB | 0 B | 1.5 kB | 0 B | **43.5 MB** |
| [B2.3] Concurrently insert N words at random positions (parseTime) | 85 ms | 71 ms | 52 ms | 168 ms | **< 1 ms** |
| [B2.4] Concurrently insert & delete (time) | 178 ms | 2,786 ms | 208 ms | 1,066 ms | **99 ms** |
| [B2.4] Concurrently insert & delete (updateSize) | 139,517 bytes | 398,881 bytes | 163,564 bytes | 298,810 bytes | **31,080 bytes** |
| [B2.4] Concurrently insert & delete (encodeTime) | 12 ms | 6 ms | 233 ms | 62 ms | **3 ms** |
| [B2.4] Concurrently insert & delete (docSize) | 279,172 bytes | 279,166 bytes | 289,590 bytes | 293,828 bytes | **31,052 bytes** |
| [B2.4] Concurrently insert & delete (memUsed) | 8.2 MB | 0 B | 1.8 kB | 0 B | **25.7 MB** |
| [B2.4] Concurrently insert & delete (parseTime) | 121 ms | 78 ms | 50 ms | 255 ms | **< 1 ms** |
| [B3.1] 20√N clients concurrently set number in Map (time) | 75 ms | 290 ms | 56 ms | 1,632 ms | **400 ms** |
| [B3.1] 20√N clients concurrently set number in Map (updateSize) | 49,169 bytes | 49,169 bytes | 161,636 bytes | 283,296 bytes | **—** |
| [B3.1] 20√N clients concurrently set number in Map (encodeTime) | 2 ms | 1 ms | 2 ms | 11 ms | **< 1 ms** |
| [B3.1] 20√N clients concurrently set number in Map (docSize) | 32,225 bytes | 32,209 bytes | 21,506 bytes | 86,167 bytes | **107,843 bytes †** |
| [B3.1] 20√N clients concurrently set number in Map (memUsed) | 0 B | 176 B | 824 B | 344 B | **3.5 MB** |
| [B3.1] 20√N clients concurrently set number in Map (parseTime) | 104 ms | 70 ms | 40 ms | 37 ms | **< 1 ms** |
| [B3.2] 20√N clients concurrently set Object in Map (time) | 84 ms | 278 ms | 67 ms | 1,726 ms | **1,865 ms** |
| [B3.2] 20√N clients concurrently set Object in Map (updateSize) | 85,082 bytes | 85,085 bytes | 200,630 bytes | 398,090 bytes | **—** |
| [B3.2] 20√N clients concurrently set Object in Map (encodeTime) | 3 ms | 2 ms | 2 ms | 30 ms | **1 ms** |
| [B3.2] 20√N clients concurrently set Object in Map (docSize) | 32,235 bytes | 32,249 bytes | 40,494 bytes | 112,570 bytes | **488,665 bytes †** |
| [B3.2] 20√N clients concurrently set Object in Map (memUsed) | 0 B | 0 B | 136 B | 0 B | **7.5 MB** |
| [B3.2] 20√N clients concurrently set Object in Map (parseTime) | 102 ms | 70 ms | 45 ms | 86 ms | **2 ms** |
| [B3.3] 20√N clients concurrently set String in Map (time) | 86 ms | 299 ms | 116 ms | 2,335 ms | **638 ms** |
| [B3.3] 20√N clients concurrently set String in Map (updateSize) | 7,826,222 bytes | 7,826,231 bytes | 7,940,240 bytes | 8,063,440 bytes | **—** |
| [B3.3] 20√N clients concurrently set String in Map (encodeTime) | 2 ms | 1 ms | 46 ms | 91 ms | **1 ms** |
| [B3.3] 20√N clients concurrently set String in Map (docSize) | 38,357 bytes | 38,376 bytes | 7,798,572 bytes | 98,047 bytes | **1,656,843 bytes †** |
| [B3.3] 20√N clients concurrently set String in Map (memUsed) | 243 kB | 0 B | 696 B | 0 B | **8.0 MB** |
| [B3.3] 20√N clients concurrently set String in Map (parseTime) | 97 ms | 52 ms | 55 ms | 118 ms | **1 ms** |
| [B3.4] 20√N clients concurrently insert text in Array (time) | 72 ms | 283 ms | 227 ms | 2,780 ms | **538 ms** |
| [B3.4] 20√N clients concurrently insert text in Array (updateSize) | 52,738 bytes | 52,751 bytes | 166,750 bytes | 311,830 bytes | **—** |
| [B3.4] 20√N clients concurrently insert text in Array (encodeTime) | 2 ms | 1 ms | 8 ms | 17 ms | **2 ms** |
| [B3.4] 20√N clients concurrently insert text in Array (docSize) | 26,583 bytes | 26,596 bytes | 31,119 bytes | 96,463 bytes | **501,582 bytes †** |
| [B3.4] 20√N clients concurrently insert text in Array (memUsed) | 588.8 kB | 0 B | 480 B | 0 B | **3.7 MB** |
| [B3.4] 20√N clients concurrently insert text in Array (parseTime) | 84 ms | 60 ms | 29 ms | 42 ms | **1 ms** |
