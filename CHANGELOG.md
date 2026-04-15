# V 1.0.0

*TBD* <!--TODO: Release date-->

The first release of `floodr` after converting it from `drill`. The focus for this release was improving documentation and structure of the project, while familiarizing myself with rust and the codebase more broadly. 

## Notes

- There were no detected performance regressions, every run I made vs the versions I tested were within a few % of each other
- **A LOT** of code was changed by LOC (lines of code), but not necessarily SLOC (source lines of code). Most of that was adding documentation, and moving existing files (~%80 of the LOC or more). Very little of the actual internal code was changed, but many dependencies required overhauls of how things were done. I don't write rust, so I did my best to make the updated versions equivalent
- Trying to document every function and struct of a language you don't write is hard. There's a lot of existing state that makes creating minimal examples to help understand how things work really hard. If you notice any documentation issues (particularly on the library side), please open a PR.
- Most of the modules were made public. As time goes on this might change, but if it does I will bump the major version of the package. 

## Features

- **Breaking change** Changed name of project from drill -> floodr after a few people mentioned it conflicts with several common tools
- **Breaking change** Removed `--benchmark` flag, and made it an optional positional argument that defaults to `benchmark.yml`
- **Breaking change** Refactored project structure
    - Split from just one `main.rs` to a `main.rs` with the primary CLI entrypoint, and a `lib.rs` for use as a library
    - Split files in the root of the `src` folder to two modules
        - `engine`: Files that drive and coreograph the execution of a benchmark
        - `parsing`: I/O and various parsing utilities to help parse YAML, csv's and handle interpolation
- **Breaking change** Removed `-o` short flag
- **Breaking Change** Changed `--timeout` to `--request-timeout`
- **Breaking change** Removed `--nanosec` and `-n` flags for "nanosecond" precision. Rust doesn't actually guarentee this (since many OS's don't), it's usually off by a few hundred even though it lets you scope to that resolution. So, just removing it since it can give inaccurate info
- **Breaking change** Updated compare system
    - Removed `--compare` and `--threshold` flags converting them to a subcommand. So `floodr --compare report.yml --threshold 2` becomes `floodr compare report.yml 2`
    - Report files now require a `base` field to know which domain to use
- **Breaking change** updated `--report` 
    - File now includes a `base` value
    - File now includes a `plan` which is a copy of the plan used to generate the report
    - File now includes a `baseline` which is the measured value to compare against
- **Breaking change** Removed `--relaxed-interpolations`. It seems like the only situations it would be useful in are better handled in other ways, and I frequently found hard edge cases especially around headers. It's still available in the API if people need it, but exposing it in the CLI is not a good idea IMO
- Added ability to `assert` against the request URL and Http version
- Removed OpenSSL system-level dependency
- Made exec acitons cross platform
    - Actions now run with `bash` by default on \*nix, `zsh` on macOS and `powershell` on windows. Each will use `-c` with the passed in command as a string, and fail when command returns error status
    - Added new `--exec-terminal` flag to allow running commands with specified terminal (note `-c "<exec command>"` is passed, make sure your terminal/language supports this)
- Delay actions now take `milliseconds` by default instead of `seconds`. If you specify `seconds` it's converted to `miliseconds`. If both are specified `milliseconds` takes precendence
- Added a debug log when failing to access context that includes the full context

## Docs

- Created documentation website
- Added docstrings to all functions and structs
- Overhauled `README.md`
- Removed `SYNTAX.md`
- Updated CLI documentation for `--compare`,`--report` and `--threshold` to be more clear
- Added `CHANGELOG.md` to see changes
- Added issue and pull request templates

## Bugs

- Fixed panic when any request exceeds 3.6 seconds (now panics if request exceeds 1 hour)
- Fixed bug when accessing headers in assertions
- Fixed bug when using an exec action on machine without `bash` present

## Chores

- Bumped versions of all crates to latest
- Fixed `cargo clippy -D warnings` statements
