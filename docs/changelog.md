# Changelog

## 0.1.0 (17 October 2026)

The first release of Standing.

### Personal Standing Index

- The Personal Standing Index, computed from composite standing and momentum per the [Methodology](methodology.md).
- The Self-Assessment Adjustment and the conservative peer estimate.
- Market hours: the market opens at the first launch of each day and closes at midnight, with a daily summary.
- Daily candles, recorded at every recomputation.

### Dashboard

- Ticker strip of every index on the market.
- Personal Standing Index with its change, composite standing, momentum, Enough Threshold and shortfall.
- The Spotlight Monitor, sampling once per second.
- Index history as daily candles, with the Enough Threshold in effect on each day; 1M, 3M, 1Y and All.
- Positions Held, with volatility ratings.
- Comparables summary and recent notices.

### Comparables and Ratings

- Comparables with tickers, relationships, coverage and spreads.
- Removal from the peer set, with history retained, and permanent deletion.
- Automatic Peer Set Rebalancing, with introduced Comparables that may be claimed.
- Ratings from 0 to 10 for yourself and each Comparable, by sector and industry, with provisional estimates marked.

### Daily Filing and Notices

- Ten event types with documented weights, including observations of Comparables.
- Momentum with a seven-day half-life; filings may be withdrawn.
- Notices for market closes, threshold revisions, peer set changes, discoveries and weight recalibrations.

### Dimension Catalog

- The Standard Classification of Comparison Dimensions (SCCD), version 4.2: 480 dimensions in 12 sectors and 63 industries.
- Dimension Discovery, every three days.
- Weight Drift, with recalibration at each market close; weights may be set by hand.

### Enough Threshold

- The Enough Threshold, set during onboarding and revisable at any time, with a full Threshold History.

### Onboarding

- Profile, Dimension Review, Comparables and Enough Threshold, resumable at any step.

### Reports

- Quarterly Statements, exported as Markdown or HTML.

### Platform

- Linux AppImage. The profile is kept in a `Standing Data` folder beside the application.
- Light and dark themes.
