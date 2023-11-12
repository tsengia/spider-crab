# Spider Crab
Command link web crawler for checking links.

The purpose of spider crab is to provide a small, portable, and fast static website checker that can be used in CI pipelines to monitor for broken links.

If Spider Crab finds the following, then it will return a non-zero exit code:
- Broken link
- `<a>` element without an `href` attribute
- `<a>` element with an `href` attribute that is blank (`href=""`)

If Spider Crab does not find any issues, then it will return a `0` exit code.

```
Usage: spider-crab [OPTIONS] <url>

Arguments:
  <url>  URL of the webpage to check.

Options:
  -d, --depth <depth>  Depth of links to check. Default is -1 which is unlimited. [default: -1]
  -q, --quiet          Do not print to STDOUT or STDERR.
  -v, --verbose        Print more log messages.
  -h, --help           Print help
  -V, --version        Print version
```

Example:
```bash
spider-crab -v https://example.com
```

## Skipping Links
If you do not want Spider Crab to check a link on your webpage, add the `scrab-skip` CSS class to the link.

Example:
```html

<a href="https://non-existent-website.net" class="scrab-skip my-custom-class" >This link will not be checked by Spider Crab!</a>

```