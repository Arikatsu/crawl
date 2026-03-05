# Overview

A logo scraper in Rust. 
Scrape mostly high quality logo sources to keep precision as high as possible.
Logs metrics at the end of execution.

# Usage

Run the nix environment:
```bash
nix-shell
```

Then run the scraper:
```bash
cargo run --release < websites.csv > logos.csv
```

### CLI options:

Providing max concurrent tasks at a time:
```bash
cargo run --release -- -c 50 < websites.csv > logos.csv
```

Muting info logs:
```bash
cargo run --release -- -q < websites.csv > logos.csv
```

# Metrics

At the end of execution, the scraper logs the following metrics:
- Total number of websites processed
- Total number of HTTP errors
- Total number of network errors
- Number of successfully reached websites
- Total number of logos successfully scraped

e.g.:
```
 --- INTERNAL CRAWL METRICS ---
Total Domains Processed: 1000
- HTTP Errors:       90
- Network Errors:    204
- Reachable Domains: 706
--------------------------------
Logos Found:         506 (True Hit Rate: 71.7% of reachable HTML)
--------------------------------
```

# Notes

- I ended up updating the nix-pkgs pin to include the latest Rust version due to some dependencies requiring it.
- The scraper is designed to be time and memory efficient, using asynchronous tasks to handle multiple websites concurrently and also streaming the HTML the content at the same time to not have to build a full DOM tree in memory.
- The number of concurrent tasks are kept in check with a semaphore, helps in backpressuring the logos from stdin and not overwhelming the system with too many tasks at once.

