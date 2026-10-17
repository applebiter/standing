<p align="center">
  <img src="assets/icons/standing.png" width="128" height="128" alt="Standing emblem">
</p>

<h1 align="center">Standing</h1>

A professional terminal for monitoring your social standing relative to the people you compare yourself to.

Standing maintains a Personal Standing Index across 480 dimensions of the Standard Classification of Comparison Dimensions, keeps a watchlist of Comparables, records the events of each day, and holds the level of standing at which you will feel secure.

It runs entirely offline. Nothing you record leaves your computer.

![The Dashboard](docs/images/dashboard.png)

## Sample record

Extract from the demonstration profile.

**Enough Threshold History**

| Date | Prior | New | Reason |
|---|---:|---:|---|
| 2 Oct 2026 | 92.00 | 111.00 | Threshold adjusted to reflect current market conditions. |
| 2 Oct 2026 | 137.00 | 92.00 | Set by user. |
| 16 Sep 2026 | 114.00 | 137.00 | Threshold adjusted to reflect current market conditions. |
| 19 Aug 2026 | 95.00 | 114.00 | Threshold adjusted to reflect current market conditions. |
| 19 Aug 2026 | — | 95.00 | Initial threshold. |

**Comparables**

| Ticker | Name | Relationship | Index | Your spread |
|---|---|---|---:|---:|
| DANA | Dana | college roommate | 96.36 | +6.32 |
| MB | Marcus Bell | brother-in-law | 100.23 | +2.40 |
| PRIY | Priya | former colleague | 97.85 | +9.19 |
| YMSM | Gordon Pratt | your manager's manager | 124.73 | −24.73 |

| Personal Standing Index | Enough Threshold | People currently thinking about you |
|---:|---:|---:|
| 86.82 | 111.00 | 0 (±0) |

## Features

- **Dashboard.** The Personal Standing Index with its change, composite standing, momentum and shortfall; a ticker strip of every index on the market; daily candles with the Enough Threshold in effect on each day; Positions Held; and the Spotlight Monitor, sampling once per second.
- **Comparables.** A watchlist of the people you measure yourself against, each with a ticker, an index and your spread. When you lead every Comparable, the peer set is rebalanced from your extended social graph.
- **Ratings.** Assess yourself and each Comparable from 0 to 10 on every applicable dimension, adjusted for known reporting bias.
- **Daily Filing.** Record compliments, invitations, exclusions, unanswered messages, slights and achievements, and observations of your Comparables. Each carries a documented weight and decays with a seven-day half-life.
- **The Dimension Catalog.** The Standard Classification of Comparison Dimensions, version 4.2: 480 dimensions in 12 sectors, with further dimensions classified every three days. Weights follow your attention.
- **The Enough Threshold.** The index level at which you will feel secure, revised to reflect current market conditions, with a complete history.
- **Reports.** A Quarterly Statement for every quarter, exported as Markdown or HTML.

## Documentation

- [User Guide](docs/user-guide.md)
- [Frequently Asked Questions](docs/faq.md)
- [Methodology](docs/methodology.md)
- [The SCCD Reference](docs/sccd-reference.md)
- [Changelog](docs/changelog.md)

## Running Standing

Standing is a Linux application.

### Download

Download `Standing-0.1.0-x86_64.AppImage` from the [Releases](../../releases) page, make it executable, and run it:

```sh
chmod +x Standing-0.1.0-x86_64.AppImage
./Standing-0.1.0-x86_64.AppImage
```

It requires no installation. See the [User Guide](docs/user-guide.md#1-getting-started).

### From source

Install Rust from [rustup.rs](https://rustup.rs), then the system libraries the window toolkit needs (Debian, Ubuntu, Mint):

```sh
sudo apt install libxkbcommon-dev libgl1-mesa-dev libegl1-mesa-dev libwayland-dev \
  libxcursor-dev libxrandr-dev libxi-dev libgtk-3-dev
```

Then:

```sh
cargo run --release
```

### As an AppImage

```sh
packaging/linux/build-appimage.sh
```

This produces `dist/Standing-<version>-x86_64.AppImage`. The AppImage is portable: it keeps your profile in a `Standing Data` folder beside itself, so the two can be moved or copied together. It runs on systems with glibc at least as new as the build machine's.

### Where the profile is kept

| How it runs | Location |
|---|---|
| As an AppImage | `Standing Data/` beside the `.AppImage` file |
| Otherwise | `~/.local/share/standing/` (profile) and `~/.config/standing/` (settings) |
| With `STANDING_HOME` set | That directory |

Settings in the application shows the location in use.

### Demonstration profile

To run against a separate, pre-filled profile without touching your own:

```sh
cargo run --release --example demo -- demo
STANDING_HOME=demo cargo run --release
```

The demonstration replays two months of use, ending on the day it is run, through the same methodology the application applies.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Acknowledgements

Typeset in IBM Plex Serif, Sans and Mono, used under the SIL Open Font License 1.1 (`assets/fonts/OFL-IBMPlex.txt`).
