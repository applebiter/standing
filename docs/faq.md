# Frequently Asked Questions

### General

**Does Standing connect to the internet?**

No. Standing has no network functions of any kind. It does not check for updates or send usage information. Everything you record remains on your computer.

**Who can see my Comparables?**

No one. The names, relationships and ratings you enter are stored in your profile on your computer and are never transmitted. They leave your computer only if you export a Quarterly Statement and share the file.

**Can Standing import Comparables or events from social platforms?**

No. Each Comparable, rating and event is entered by hand, which ensures that every entry in the record has been reviewed by the person best placed to assess it.

**How do I move my profile to another computer?**

Copy the AppImage and the `Standing Data` folder beside it to the other computer, keeping them in the same folder. See the User Guide, [Your profile and its location](user-guide.md#11-your-profile-and-its-location).

**How do I back up my profile?**

Close Standing and copy the `Standing Data` folder.

### The index

**Why is my index below 100 when I rated myself the same as my Comparables?**

Ratings are adjusted before use, correcting for known reporting bias: your own by a factor of 0.85 and your Comparables' by 1.10. Equal ratings are therefore not equal after adjustment. See the [Methodology](methodology.md#2-the-self-assessment-adjustment).

**A Comparable rated 10 shows an adjusted rating of 11. Is that an error?**

No. The adjusted rating of a Comparable is not capped.

**I have not rated a Comparable on some dimensions. Are those dimensions ignored?**

No. Where you have rated yourself and a Comparable is not yet rated, the Comparable is taken at your adjusted rating plus 0.5, the conservative peer estimate. Dimensions you have not rated yourself on are excluded for everyone.

**My index changed, but I have not done anything.**

Momentum declines as events age: each event's contribution halves every seven days. The index is also recomputed at each market open, when a dimension may be discovered, and whenever your Comparables or weights change.

**Why did my index fall when a new Comparable was added?**

Your index measures your standing against the mean of your peer set. A new Comparable whose standing exceeds the mean raises it.

### Comparables

**Where did a Comparable I did not add come from?**

When your composite standing exceeds that of every Comparable in your peer set, a new Comparable is identified from your extended social graph and added. It is described by its relationship to you until you claim it. See the User Guide, [Introduced Comparables](user-guide.md#introduced-comparables).

**Can I remove an introduced Comparable?**

Yes. Any Comparable may be removed from the peer set or deleted permanently. If your standing again exceeds that of every remaining Comparable, a further Comparable will be introduced, at most once a day.

**Can I change a Comparable's ticker?**

No. Tickers are issued once and never changed, including when a Comparable is renamed or claimed.

### The Enough Threshold

**Why did my threshold change?**

Your index reached it. When the Personal Standing Index reaches the Enough Threshold, the threshold is revised to reflect current market conditions: raised by 20%, rounded up to the next whole point. Each revision is recorded in the Threshold History and posted as a notice.

**I lowered my threshold, and it changed again immediately.**

A threshold entered at or below your current index is revised when it is saved, in the same way.

**Can this be turned off?**

The threshold is applied as described in the [Methodology](methodology.md#8-the-enough-threshold). There is no setting that alters it.

**What is the highest threshold I can set?**

10,000. Thresholds revised by Standing may exceed it.

### Dimensions and weights

**A dimension appeared that I did not see during onboarding.**

Further dimensions are classified from time to time. Every three days, at the market open, one is added to the catalog and applies to you from the start. To exclude it, clear **Applies** in the Dimension Catalog.

**Why does one of my Comparables already lead in a dimension I have just seen for the first time?**

When a dimension is discovered, one Comparable in your peer set is given a provisional estimate in it, ahead of your own rating or, if you have not rated yourself, at 7.0. Record a rating of your own for them to replace it.

**Why have my weights changed?**

Each rating you record adds 0.02 to the weight of its dimension, and weights are recalibrated to a mean of 1.00 at each market close. Dimensions you attend to come to count for more. See [Weight Drift](methodology.md#9-weight-drift).

**Can I add my own dimensions?**

No. The classification is maintained as a standard, so that standing is measured consistently.

### The Spotlight Monitor

**What does the Spotlight Monitor measure?**

The number of people currently thinking about you, with its margin of error. It samples once per second while the Dashboard is displayed.

**Is the reading accurate?**

Each reading is reported with its margin of error.

**Why does it say *None recorded* under Last non-zero reading?**

No reading with a count above zero has been taken.
