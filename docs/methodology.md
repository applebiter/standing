# Methodology

Version 0.1.0

This document defines every figure Standing computes, and the conventions by which it is computed and displayed. Each figure can be reproduced from the record and the definitions below.

**Contents**

1. [Definitions](#1-definitions)
2. [The Self-Assessment Adjustment](#2-the-self-assessment-adjustment)
3. [Composite Standing](#3-composite-standing)
4. [Momentum](#4-momentum)
5. [The Personal Standing Index](#5-the-personal-standing-index)
6. [Comparable tickers and spreads](#6-comparable-tickers-and-spreads)
7. [Market hours](#7-market-hours)
8. [The Enough Threshold](#8-the-enough-threshold)
9. [Weight Drift](#9-weight-drift)
10. [Peer Set Rebalancing](#10-peer-set-rebalancing)
11. [Dimension Discovery](#11-dimension-discovery)
12. [Positions Held](#12-positions-held)
13. [The Spotlight Monitor](#13-the-spotlight-monitor)
14. [Reproducibility](#14-reproducibility)
15. [Presentation](#15-presentation)
16. [Summary of constants](#16-summary-of-constants)

---

## 1. Definitions

A **dimension** is an entry in the Standard Classification of Comparison Dimensions (SCCD). A dimension is **applicable** if it is marked *Applies to me* and has been classified into the catalog (that is, it is not held in reserve for [Dimension Discovery](#11-dimension-discovery)). Let *D* be the set of applicable dimensions.

Each dimension *d* carries a **weight** *w*<sub>*d*</sub> > 0. See [Weight Drift](#9-weight-drift).

A **subject** is either the user or a Comparable. A **rating** *r* is a value from 0 to 10, in steps of 0.5. Ratings are never overwritten: each new rating is recorded alongside its predecessors, and the **current rating** of a subject on a dimension is the one recorded most recently.

An **active** Comparable is one in the peer set. Comparables removed from the peer set are retained in the record and take no part in the computations below.

A **day** is a calendar day in the computer's time zone.

## 2. The Self-Assessment Adjustment

Ratings are adjusted before use, correcting for known reporting bias.

| Subject | Adjusted rating |
|---|---|
| The user | *a* = 0.85 × *r* |
| A Comparable | *a* = 1.10 × *r* |

The adjusted rating of a Comparable is not capped. A Comparable rated 10 has an adjusted rating of 11.0.

A user and a Comparable with equal ratings on a dimension do not have equal adjusted ratings.

## 3. Composite Standing

Let *R* ⊆ *D* be the applicable dimensions on which the user has a current rating.

The user's **composite standing** *S* is the weighted mean of the user's adjusted ratings over *R*:

> *S* = Σ<sub>*d* ∈ *R*</sub> *w*<sub>*d*</sub> · *a*<sub>*d*</sub> ÷ Σ<sub>*d* ∈ *R*</sub> *w*<sub>*d*</sub>

The composite standing *C* of a Comparable is the weighted mean over the same set *R*. Where the Comparable has no current rating on a dimension in *R*, the **conservative peer estimate** is used in its place: the user's adjusted rating on that dimension plus 0.5.

| Case | Contribution of dimension *d* to a Comparable's composite |
|---|---|
| Comparable rated on *d* | 1.10 × the Comparable's rating |
| Comparable not rated on *d* | 0.85 × the user's rating + 0.5 |
| User not rated on *d* | None: *d* is not in *R* |

Dimensions the user has not rated take no part in any subject's composite. If *R* is empty, no composite is defined.

## 4. Momentum

Each event recorded in the Daily Filing carries a fixed weight, in index points.

| Event | Weight | Applies to |
|---|---:|---|
| Compliment received | +2.00 | The user |
| Invitation received | +3.00 | The user |
| Not invited (learned of after the fact) | −4.00 | The user |
| Text left unanswered | −1.50 | The user |
| Achievement | +5.00 | The user |
| Public recognition | +4.00 | The user |
| Slight (minor) | −2.00 | The user |
| Slight (major) | −5.00 | The user |
| Observed Comparable achievement | +5.00 | The Comparable observed |
| Comparable seen to be thriving | +3.00 | The Comparable observed |

An event's **contribution** halves every seven days. For an event of weight *v* that occurred *t* days ago (fractional days included):

> contribution = *v* × 0.5<sup>*t* ÷ 7</sup>

An event dated later than the time of computation contributes its full weight.

A subject's **momentum** *M* is the sum of the contributions of the events that apply to it.

## 5. The Personal Standing Index

Let *C*<sub>1</sub> … *C*<sub>*n*</sub> be the composites of the active Comparables, and *C̄* their mean.

> **PSI** = 100 + 10 × (*S* − *C̄*) + *M*

| Case | PSI |
|---|---|
| *S* defined and *n* ≥ 1 | 100 + 10 × (*S* − *C̄*) + *M* |
| No active Comparables | 100 + *M* |
| The user has rated no applicable dimension | 100 + *M* |

At parity, with no momentum, the index stands at 100.

The index is recomputed whenever the record changes (a rating, a filing, a change to the peer set, the catalog, a weight, or the Enough Threshold) and at each market open. Every recomputation is recorded with its time.

## 6. Comparable tickers and spreads

Each Comparable is issued a **ticker** when it enters the record. The ticker is the first four letters of a one-word name, or otherwise the initials of up to four words, in capitals. If the ticker is taken, a numeral is appended. **PSI** is reserved for the user. A ticker is never changed, including when a Comparable is renamed or claimed.

The value of a Comparable's ticker is:

> **Ticker** = 100 + 10 × (*C* − *S*) + *M*<sub>Comparable</sub>

where the spread term is zero if either composite is undefined.

The user's **spread** against a Comparable, in index points, is:

> **Spread** = 10 × (*S* − *C*)

It is positive when the user leads, and negative when the Comparable leads. Momentum is excluded from the spread.

## 7. Market hours

The market **opens** on the first launch of Standing on each day. At the open, any earlier day on which the market opened but has not closed is closed, [Dimension Discovery](#11-dimension-discovery) is applied, and the index is recomputed.

The market **closes** at midnight. If Standing is running at midnight, the close is applied then. Otherwise it is applied at the next open, for each day that requires it.

At each close:

1. Weights are recalibrated ([Weight Drift](#9-weight-drift)).
2. A summary of the day is posted, giving the closing value, the change from the previous close, and the session range. The range is omitted when the index did not move during the session.

A day's **candle** is formed from the index values recorded during the day:

| Field | Value |
|---|---|
| Open | The first value recorded |
| High | The highest value recorded |
| Low | The lowest value recorded |
| Close | The last value recorded |

A day with no recorded values carries the previous close as its open, high, low and close. Days before the first recorded value have no candle.

The **change** shown on the Dashboard and the ticker strip is measured from the previous close. On the first day of record, it is measured from the day's open.

## 8. The Enough Threshold

The **Enough Threshold** *T* is the index value at which the user will feel secure. It is set during onboarding and may be revised at any time. A threshold entered by the user must be from 1 to 10,000, and is rounded to two decimal places.

Whenever the index is recomputed:

> while PSI ≥ *T*: *T* ← ⌈1.20 × *T*⌉

where ⌈ ⌉ rounds up to the next whole point. The product is first rounded to six decimal places, so that a threshold of 110 is revised to 132.

| Case | Result |
|---|---|
| PSI < *T* | No change |
| *T* ≤ PSI < ⌈1.20 × *T*⌉ | One revision |
| PSI ≥ ⌈1.20 × *T*⌉ | One revision for each step, each recorded separately |

Each revision is recorded in the Threshold History with its date, prior value and new value, and posted as a notice, with the reason: *Threshold adjusted to reflect current market conditions.*

A threshold entered at or below the current index is revised when it is saved.

## 9. Weight Drift

Every dimension begins with a weight of 1.00.

Each time the user records a rating on a dimension, for themselves or for a Comparable, that dimension's weight increases by 0.02. Provisional estimates (sections [10](#10-peer-set-rebalancing) and [11](#11-dimension-discovery)) do not affect weights.

At each market close, the weights of applicable dimensions are rescaled so that their mean is exactly 1.00. The weights of dimensions that do not apply are left unchanged.

After rescaling, each weight is compared with its value at the last recalibration (or with 1.00, before the first). If any differs from it by more than 5%:

1. The weights are recorded as a recalibration, and become the basis for the next comparison.
2. A notice is posted: *Weights recalibrated to reflect observed priorities.*

Rescaling raises or lowers every applicable weight, including those of dimensions not recently rated. A weight may therefore be revised by more than 5% without having been rated.

Weights may be set by hand in the Dimension Catalog, from 0.10 to 5.00. Weight Drift continues from the value set.

## 10. Peer Set Rebalancing

After each recomputation, a new Comparable is introduced to the peer set if all of the following hold:

- the user's composite standing is defined;
- there is at least one active Comparable;
- the user's composite standing exceeds that of every active Comparable; and
- no Comparable has been introduced on the same day.

The new Comparable is identified from the user's extended social graph by relational archetype, for example *Your manager's manager*. Archetypes may refer to an existing Comparable by name, or to the user's line of work; those that cannot be completed are not used. An archetype is not used twice while others remain.

The new Comparable receives a **provisional estimate** on each applicable dimension the user has rated:

| Probability | Provisional estimate |
|---:|---|
| 70% | The user's rating plus between 0.3 and 1.5 |
| 30% | The user's rating less between 0 and 1.0 |

Estimates are limited to the range 0 to 10 and rounded to the nearest 0.5. If the resulting composite does not exceed the user's, the estimates are drawn again; on the eighth draw, every estimate is placed above the user's rating.

A notice is posted: *A new Comparable has been added to your peer set.*

**Claiming.** Once the user has identified the person concerned, the Comparable may be claimed: renamed, with its relationship recorded. Its record and ticker are kept, and its ratings cease to be provisional.

## 11. Dimension Discovery

The catalog holds dimensions in reserve. At each market open, if at least three days have passed since the last discovery (or, before the first, since the profile was created), one reserved dimension is classified into the catalog.

A discovered dimension:

- applies to the user from the start (this may be changed in the Dimension Catalog);
- has a weight of 1.00; and
- is led by one active Comparable, which receives a provisional estimate on it.

| Case | Leading Comparable's provisional estimate |
|---|---|
| The user has rated the dimension | The user's rating plus 1.0, to a maximum of 10 |
| The user has not rated the dimension | 7.0 |

A notice is posted, naming the dimension and the Comparable that currently leads.

At most one dimension is discovered at each open. When no reserved dimensions remain, discovery ends.

## 12. Positions Held

A **position** is held on an applicable dimension the user has rated if the user's adjusted rating exceeds the adjusted rating of every active Comparable. A Comparable not rated on the dimension is taken at the conservative peer estimate ([section 3](#3-composite-standing)), and so precludes a position. With no active Comparables, no position is held.

The **margin** of a position is the user's adjusted rating less the highest Comparable's. Its **volatility** is rated by margin:

| Margin | Volatility |
|---|---|
| 3.0 or more | Low |
| 1.5 to under 3.0 | Moderate |
| 0.5 to under 1.5 | High |
| Under 0.5 | Extreme |

Positions are listed widest margin first.

## 13. The Spotlight Monitor

The Spotlight Monitor reports the number of people currently thinking about the user, with its margin of error. It samples once per second while the Dashboard is displayed.

Each reading comprises:

| Field | Description |
|---|---|
| Count | People currently thinking about the user |
| Margin | The margin of error of the count, in people |
| Time of reading | The time the sample was taken |
| Last non-zero reading | The time of the most recent reading with a count above zero |

Where no non-zero reading has been taken, the last non-zero reading is reported as *None recorded*.

## 14. Reproducibility

Every figure defined in this document is computed from the record. Choices made on the user's behalf (the archetype and estimates of a new Comparable, and the dimension discovered and the Comparable that leads it) are drawn from a generator seeded with a value fixed when the profile is created, together with the number of such choices already made. The same record therefore always produces the same results.

## 15. Presentation

- Index values, spreads and changes are shown to two decimal places. Negative values use a true minus sign (−).
- Ratings are shown to one decimal place.
- Weights are shown to two decimal places.
- Figures are shown with comma thousands separators.
- Green and red are used for index movement only.

## 16. Summary of constants

| Constant | Value | Section |
|---|---:|---|
| Self-assessment factor | 0.85 | [2](#2-the-self-assessment-adjustment) |
| Peer reporting factor | 1.10 | [2](#2-the-self-assessment-adjustment) |
| Conservative peer margin | 0.5 | [3](#3-composite-standing) |
| Index baseline | 100 | [5](#5-the-personal-standing-index) |
| Spread multiplier | 10 | [5](#5-the-personal-standing-index) |
| Momentum half-life | 7 days | [4](#4-momentum) |
| Threshold adjustment factor | 1.20 | [8](#8-the-enough-threshold) |
| Threshold range | 1 to 10,000 | [8](#8-the-enough-threshold) |
| Weight drift per rating | 0.02 | [9](#9-weight-drift) |
| Recalibration threshold | 5% | [9](#9-weight-drift) |
| Weight range (set by hand) | 0.10 to 5.00 | [9](#9-weight-drift) |
| Provisional estimate above the user | 70%, +0.3 to +1.5 | [10](#10-peer-set-rebalancing) |
| Provisional estimate below the user | 30%, −0 to −1.0 | [10](#10-peer-set-rebalancing) |
| Discovery interval | 3 days | [11](#11-dimension-discovery) |
| Discovery lead | +1.0, or 7.0 if unrated | [11](#11-dimension-discovery) |
| Volatility bands | 3.0 / 1.5 / 0.5 | [12](#12-positions-held) |
| Spotlight sampling interval | 1 second | [13](#13-the-spotlight-monitor) |
