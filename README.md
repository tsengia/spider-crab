# Spider Crab
![Spider Crab Logo](spider_crab_logo.png)  
[![CI Build](https://github.com/tsengia/spider-crab/actions/workflows/lint_fmt_test.yml/badge.svg)](https://github.com/tsengia/spider-crab/actions/workflows/lint_fmt_test.yml)

Web crawler for checking links.

The purpose of spider crab is to provide a small, portable, and fast static website checker that can be used in CI pipelines to monitor for broken links.

If Spider Crab finds the following, then it will return a non-zero exit code:
- A referenced URL/page returns an unsuccessful HTTP status code
- An `<a>` or `<link>` element without an `href` attribute, or an `href` attribute that is blank (`href=""`)
- An `<img>` element without a `src` attribute, or a `src` attribute that is empty
- A `<script>` element without a `src` attribute and no content between the tags

If Spider Crab does not find any issues, then it will return a `0` exit code.

```
Usage: spider-crab.exe [OPTIONS] <url>

Arguments:
  <url>  URL of the webpage to check.

Options:
  -d, --depth <depth>  Depth of links to check. Default is -1 which is unlimited. [default: -1]
  -q                   Silence logging output.
  -v...                Print more log messages.
  -o, --dot <dot>      Save output to file in graphiz Dot format.
  -h, --help           Print help
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

## Suppressing Errors
If you want to ignore specific errors on specific pages, then you can write a `.spidercrab-ignore` file and place it in your working directory. 
When spider-crab launches, it will read the file line by line for a `ignore-rule target-url` pairing, separated by any amount whitespace. 
Lines starting with a `#` are comments and will be ignored.  

The names of rules to ignore are printed between the parenthesis `()` of an error report when you run spider crab.  
For example, to ignore this error:
```
ERROR - SpiderError (missing-title): Page at "https://example-page.com/somewhere/something.html" does not have a title!
```
We would need to add this line to our `.spidercrab-ignore` file:
```
missing-title   https://example-page.com/somewhere/something.html
```


Here is a more complete example of an `.spidercrab-ignore` file:
```
# This line is a comment
# Ignore that this page doesn't have a title. It's an archived page that we won't fix due to historic reasons
missing-title   https://old-website.com/somewhere/something.html
# Ignore the 400 HTTP status code this website returns. It's an external website that blocks spider crab
http-error      https://another-website-somewhere.org/
```

## Development
Since version 1.0.0, `spider-crab` uses the [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) standard for commit messages.
However, if you make contributions that do not follow the Conventional Commits standard, then a maintainer will squash your commits and make a merge commit that follows the Conventional Commits standard.

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

## Release Process
This repo uses two workflows in `.github/workflows`:
- `lint_fmt_test.yml` - Checks Pull Requests, runs formatter, clippy, `cargo check`, `cargo test`, and ensures that the version number in `Cargo.toml` was bumped.
- `release.yml` - Performs a `cargo build --release` and uploads the compiled binary to GitHub

To make a new release, perform the following:
- Make a tag on the `main` branch that matches the version number in `Cargo.toml`. Update `Cargo.toml` if necessary.
- Push the tag.
- Watch for the GitHub workflow to finish and upload the build artifacts.