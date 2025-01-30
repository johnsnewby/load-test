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
$ cat urls.txt | load-test -p 3
{"average_request_duration":580,"longest_request_duration":1013,"shortest_request_duration":130,"status_codes":{"200":1,"404":0},"test_duration":1991,"total_downloaded":832292,"total_requests":3}
```

## Bugs

None known but surely there are some.
