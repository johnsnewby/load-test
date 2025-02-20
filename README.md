# Tiny load tester

## Summary

    This is a tiny load tester which simply reads a list of URLs from standard in, and executes them with some degree of concurrency, then reports back on shortest, longest and average request times, distribution of response status codes, total time taken and total downloaded. It outputs a JSON structure so that you can use it with jq in a command pipeline. It is intended that it be used in conjunction with other unix command line tools, and kubernetes (see section below).

## Building

- `cargo build --release` (or debug if you want)
- `cargo build --release --target=x86_64-unknown-linux-musl` for the most portable x86_64 binary


## Arguments

`-p` -- number of parallel workers to run making requests


## Usage

```
$ cat > urls.txt
https://jup.ag
https://github.com/johnsnewby/load-tester-there-is-nothing-here
https://github.com/johnsnewby/load-tester
$ cat urls.txt | ./target/release/load-test -p 100 | jq .
{
  "average_request_duration_ms": 124,
  "invalid_requests": 0,
  "longest_request_duration_ms": 211,
  "requests_per_second": 1284,
  "shortest_request_duration_ms": 22,
  "status_codes": {
    "200": 4980
  },
  "test_duration_ms": 6395,
  "total_downloaded_bytes": 8929140,
  "valid_requests": 4980
}

```

All times are in milliseconds.

## Kubernetes Usage

To use load-test an application on a k8s pod, from that pod, first build a binary which will work on your target system. For x86 systems the target `86_64-unknown-linux-musl` may help reduce any libc dependencies. To compile on Ubuntu for an x88 target:

```
$ sudo apt install musl-dev musl-tools
$ cargo build --release --target=x86_64-unknown-linux-musl
$ kubectl cp -n $NAMESPACE  target/x86_64-unknown-linux-musl/release/load-test $POD:/tmp/load-test
$ cat urls.txt | kubectl -n $NAMESPACE exec $POD -i -- /tmp/load-test -p 20 | jq .
{
  "average_request_duration_ms": 193,
  "invalid_requests": 0,
  "longest_request_duration_ms": 736,
  "requests_per_second": 201,
  "shortest_request_duration_ms": 13,
  "status_codes": {
    "200": 498
  },
  "test_duration_ms": 4967,
  "total_downloaded_bytes": 893412,
  "valid_requests": 498
}
```

## Bugs

None known but surely there are some.
