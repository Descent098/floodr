# floodr

*Floodr is a HTTP load testing system designed to be lightweight, fast, and highly configurable*

## Demo

[![asciicast](https://asciinema.org/a/NfDR2EDUYkyH57aO.svg)](https://asciinema.org/a/NfDR2EDUYkyH57aO)

## Features

This is the list of all features supported by the current version of `floodr`:

- **Concurrency:** run your benchmarks choosing the number of concurrent iterations.
- **Multi iterations:** specify the number of iterations you want to run the benchmark.
- **Ramp-up:** specify the amount of time, in seconds, that it will take `floodr` to start all iterations.
- [**Delay:** introduce controlled delay between requests.](https://kieranwood.ca/floodr/benchmark-reference/actions/delay)
- [**Variables:** execute requests with dynamic interpolations in the url, like `/api/users/{{ item }}`](https://kieranwood.ca/floodr/benchmark-reference/actions/assign)
- [**Dynamic headers:** execute requests with dynamic headers.](https://kieranwood.ca/floodr/benchmark-reference/actions/requests#custom-headers)
- [**Executions:** execute commands within tests.](https://kieranwood.ca/floodr/benchmark-reference/actions/exec)
- [**Assertions:** assert values during the test plan, failing when they are not met.](https://kieranwood.ca/floodr/benchmark-reference/actions/asserts)
- [**Split files:** organize your benchmarks in multiple files and include them.](https://kieranwood.ca/floodr/benchmark-reference/include)
- [**CSV support:** read CSV files and build N requests fill dynamic interpolations with CSV data.](https://kieranwood.ca/floodr/benchmark-reference/expandables)
- [**HTTP methods:** build request with different http methods like GET, POST, PUT, PATCH, HEAD or DELETE.](https://kieranwood.ca/floodr/benchmark-reference/actions/requests#setting-the-method)
- [**Cookie support:** create benchmarks with sessions because cookies are propagates between requests.](https://kieranwood.ca/floodr/benchmark-reference/actions/requests#cookies)
- [**Stats:** get nice statistics about all the requests.](https://kieranwood.ca/floodr/getting-started/basic-usage)
- [**Thresholds:** compare the current benchmark performance against a stored one session and fail if a threshold is exceeded.](https://kieranwood.ca/floodr/cli)

## Installation

You have 3 options for install, as a binary, or via `cargo` from [crates.io](https://crates.io), or from source.

### Binary Install

The easiest way to get `floodr` is to download a pre-built binary from the [latest release](https://github.com/descent098/floodr/releases/latest) page.


### Cargo Install Options

If you have [Rust](https://rustup.rs/) available on your machine you can install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) directly from `crates.io` using:

```bash 
cargo install floodr
```

or you can build it from source:

```bash
git clone https://github.com/descent098/floodr
cd floodr
cargo install --path . 
```

## Basic Usage

Floodr uses benchmark files (YAML files) to plan runs. The simplest plan is a single request to a set URL:

`benchmark.yml`

```yaml
base: http://localhost:4896 # The domain to run queries against

plan: # The actions to run
- name: Fetch
  request:
    url: /
```

You can then run using `floodr` or with more details by running `floodr --stats`:

```bash
$> floodr --stats
Concurrency 1
Iterations 1
Rampup 0
Base URL http://localhost:4896

Fetch                     http://localhost:4896/ 200 OK 309ms

Fetch                     Total requests            1
Fetch                     Successful requests       1
Fetch                     Failed requests           0
Fetch                     Median time per request   309ms
Fetch                     Average time per request  308ms
Fetch                     Sample standard deviation 0ms
Fetch                     99.0'th percentile        309ms
Fetch                     99.5'th percentile        309ms
Fetch                     99.9'th percentile        309ms

Time taken for tests      0.3 seconds
Total requests            1
Successful requests       1
Failed requests           0
Requests per second       3.23 [#/sec]
Median time per request   309ms
Average time per request  308ms
Sample standard deviation 0ms
99.0'th percentile        309ms
99.5'th percentile        309ms
99.9'th percentile        309ms
```

We can then begin increasing the load by adding more concurrency an iterations:

`benchmark.yml`

```yaml
base: http://localhost:4896 # The domain to run queries against
concurrency: 10 # How many threads to run
iterations: 2000 # Total number of times to run the plan

plan: # The actions to run
- name: Fetch
  request:
    url: /
```

We can run with `--quiet` and `--stats` to just see the overall results:

```bash
$>floodr --quiet --stats
Concurrency 10
Iterations 2000
Rampup 0
Base URL http://localhost:4896


Fetch                     Total requests            2000
Fetch                     Successful requests       2000
Fetch                     Failed requests           0
Fetch                     Median time per request   0ms
Fetch                     Average time per request  2ms
Fetch                     Sample standard deviation 23ms
Fetch                     99.0'th percentile        0ms
Fetch                     99.5'th percentile        1ms
Fetch                     99.9'th percentile        324ms

Time taken for tests      0.4 seconds
Total requests            2000
Successful requests       2000
Failed requests           0
Requests per second       5281.91 [#/sec]
Median time per request   0ms
Average time per request  2ms
Sample standard deviation 23ms
99.0'th percentile        0ms
99.5'th percentile        1ms
99.9'th percentile        324ms
```

## Roadmap

- Be able to run benchmarks for a set amount of time instead of iterations
- Cli options for quick tests without a dedicated benchmark plan
- TUI for interactive sessions
- Allow reports for PDF's with charts
- Web interface for interactive sessions
- improve the API for easier library usage

## FAQ

<details><summary>Can I use this without any extra files?</summary>
No, currently you need a benchmark plan (YAML file), but in the future there are plans for a CLI where you will just be able to specify a route and hit it as hard as you can
</details>
<details><summary>Can I run a test for a set amount of time?</summary>

No, calculating how long a plan runs for currently is also quite complicated, so there are plans in the future to make it possible to just run a test with a set `concurrency` and route to hit
</details>
<details><summary>Why the name change? </summary>

This project is a fork of [drill](https://github.com/fcsonline/drill), which was a fantastic base to build from. Unfortunately the name `drill` is already taken both on crates.io, and conflicts with a few other tools as mentioned in other issues like [this one](https://github.com/fcsonline/drill/issues/160), and [this](https://github.com/fcsonline/drill/issues/181) in the original project. No one seemed to be using `floodr` so I grabbed it!

</details>
<details><summary>What about Drill? </summary>

This project is a fork of [drill](https://github.com/fcsonline/drill), which was a fantastic base to build from. I originally was hoping of creating a compatible version and just contributing to the original project. 

Unfortunately, due to questions around [maintainer status](https://github.com/fcsonline/drill/issues/200) I tentatively started a fork, and was intending to contribute upstream. As time went on I wanted more breaking changes. At this point while the core is largely the same, I changed **a lot**, and intend to change a lot more. I have limited time to work on side projects these days, so I just do work when I can, and while I have motivation. Likewise a lot of [pretty major bugs were left unsolved in the original](https://github.com/fcsonline/drill/pull/223) (no shade to the original dev, everyone gets busy), and I need something like this for work, so hard fork it is.

The original project is still great, and worth a look. If you're feeling generous I would give the original dev a donation(I did), they did great work:

[!["Buy Them A Coffee"](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/fcsonline)

</details>

