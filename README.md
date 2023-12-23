# Spider Crab
![Spider Crab Logo](spider_crab_logo.png)  
[![CI Build](https://github.com/tsengia/spider-crab/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/tsengia/spider-crab/actions/workflows/rust.yml)  

Web crawler for checking links.

The purpose of spider crab is to provide a small, portable, and fast static website checker that can be used in CI pipelines to monitor for broken links.

If Spider Crab finds the following, then it will return a non-zero exit code:
- A referenced URL/page returns an unsuccessful HTTP status code
- An `<a>` or `<link>` element without an `href` attribute, or an `href` attribute that is blank (`href=""`)
- An `<img>` element without a `src` attribute, or a `src` attribute that is empty
- A `<script>` element without a `src` attribute and no content between the tags

If Spider Crab does not find any issues, then it will return a `0` exit code.

```
Usage: spider-crab [OPTIONS] <url>

Arguments:
  <url>  URL of the webpage to check.

Options:
  -d, --depth <depth>  Depth of links to check. Default is -1 which is unlimited. [default: -1]
  -q, --quiet          Do not print to STDOUT or STDERR.
  -v, --verbose        Print more log messages. Append additional 'v' characters to increase verbosity.
  -h, --help           Print help
  -V, --version        Print version
```

Example:
```bash
spider-crab -v https://example.com
```

## Skipping Links
If you do not want Spider Crab to check a link/element on your webpage, add the `scrab-skip` CSS class to the link.

Example:
```html

<a href="https://non-existent-website.net" class="scrab-skip my-custom-class" >This link will not be checked by Spider Crab!</a>

```

## Development
`spider-crab` uses the default `cargo fmt` formatter and `cargo clippy` linter.

To run the integration tests, run: `cargo test`.

## Code Coverage
To generate source based code coverage reports, use the following commands:

1. Install `llvm-tools-preview` and `grcov`
```bash
rustup component add llvm-tools-preview

cargo install grcov
```
2. Clean the build
```bash
cargo clean
```
3. Run the tests with `RUSTFLAGS` set to create profile files
```bash
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
```

**For Windows users:**
```batch
CARGO_INCREMENTAL=0 
RUSTFLAGS=-C instrument-coverage
LLVM_PROFILE_FILE=cargo-test-%p-%m.profraw
cargo test
```

4. Generate an HTML report file with grcov:
```bash
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" --ignore 'target/*/build/*5ever' -o target/coverage/html
```