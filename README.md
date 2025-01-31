# Tiny load tester

## Summary

This is a tiny load tester which simply reads a list of URLs from standard in, and executes them with some degree of parallelism, then reports back on shortest, longest and average request times, distribution of response status codes, total time taken and total downloaded. It outputs a JSON structure so that you can use it with jq in a command pipeline. It is intended that it be used in conjunction with other unix command line tools, and kubernetes (see section below).

## Building

`cargo build --release` (or debug if you want)
`cargo build --release --target=x86_64-unknown-linux-musl` for the most portable x86_64 binary


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
  "average_request_duration": 487,
  "invalid_requests": 0,
  "longest_request_duration": 821,
  "shortest_request_duration": 266,
  "status_codes": {
    "200": 2,
    "404": 1
  },
  "test_duration": 984,
  "total_downloaded": 848074,
  "valid_requests": 3
}

```

All times are in milliseconds.

## Kubernetes Usage

To use load-test an applicatiopn on a k8s pod, from that pod, first build a binary which will work on your target system. For x86 systems the target `86_64-unknown-linux-musl` may help reduce any libc dependencies. To compile on Ubuntu for an x88 target:

```
$ sudo apt install musl-dev musl-tools
$ cargo build --release --target=x86_64-unknown-linux-musl
$ kubectl cp -n $NAMESPACE  target/x86_64-unknown-linux-musl/release/load-test $POD:/tmp/load-test
$ cat urls.txt | kubectl -n $NAMESPACE exec $POD -i -- /tmp/load-test -p 20 | jq .
{
  "average_request_duration": 261,
  "invalid_requests": 0,
  "longest_request_duration": 701,
  "shortest_request_duration": 24,
  "status_codes": {
    "200": 498
  },
  "test_duration": 6770,
  "total_downloaded": 893910,
  "valid_requests": 498
}
```

## Bugs

None known but surely there are some.
