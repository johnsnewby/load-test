# Tiny load tester

## Summary

This is a tiny load tester which simply reads a list of URLs from standard in, and executes them with some degree of parallelism, then reports back on shortest, longest and average request times, distribution of response status codes, total time taken and total downloaded. It outputs a JSON structure so that you can use it with jq in a command pipeline. It is intended that it be used in conjunction with other unix command line tools. 

## Building

`cargo build --release` (or debug if you want)

## Arguments

`-p` -- number of parallel workers to run making requests


## Usage

```
$ cat > urls.txt
https://jup.ag
https://github.com/johnsnewby/load-tester-there-is-nothing-here
https://github.com/johnsnewby/load-tester
$ cat urls.txt | ./target/debug/load-test -p 3 | jq .
{
  "average_request_duration": 353,
  "longest_request_duration": 608,
  "shortest_request_duration": 109,
  "status_codes": {
    "200": 1,
    "404": 0
  },
  "test_duration": 758,
  "total_downloaded": 836350,
  "total_requests": 3
}
```

All times are in milliseconds.

## Bugs

None known but surely there are some.
