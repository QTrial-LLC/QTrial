# AKC regulation rulebook PDFs

Frozen-in-repo copies of the current AKC regulation rulebooks for the
two MVP sports (Obedience and Rally). Migration headers and seed
comments cite specific Chapter / Section locations in these PDFs as
authoritative sources for combined-award definitions, title-progression
rules, and entry-fee mechanics.

The PDFs are committed (not URL-referenced) so that future readers can
cite the exact text the design was based on, even if AKC re-publishes
the rulebook at a new URL or with a different filename.

## Files

| File | Source URL (verified 2026-04-25) | Edition identifier |
|---|---|---|
| `akc_rally_regulations_1217.pdf` | `https://images.akc.org/pdf/AKC1193_ROR001_1217_WEB.pdf` | 1217 (December 2017 base; amended through January 8, 2024 with January 2026 / November 2025 / July 2024 inserts) |
| `akc_obedience_regulations_2025_03.pdf` | `https://images.akc.org/pdf/RO2999.pdf` | 2025-03 (March 1, 2025 amended edition) |

## Naming convention

`akc_<sport>_regulations_<edition>.pdf`. The `<edition>` segment is the
publication identifier from AKC's URL or PDF metadata: a four-digit
month-year code (e.g. `1217` for the 2017-12 Rally base) for stable
editions, or `YYYY_MM` for amendment-dated editions.

When AKC publishes a new edition, drop the new file alongside the old
one and update citations to the new edition. The old file stays for
historical reference; do not delete prior editions when the migration
or seed comments still cite them.

## Citations referenced from PR 2d migration headers and seeds

These section numbers were verified against the rulebooks as committed
in this directory on 2026-04-25.

**Rally Regulations (`akc_rally_regulations_1217.pdf`):**

- Chapter 1, Section 24: Limitation of Entries and Methods of Entry.
  Defines the RAE combined-entry mechanism (Advanced B + Excellent B)
  and the RACH combined-entry mechanism (Advanced B + Excellent B +
  Master). Both require the combined fee to be paid.
- Chapter 1, Section 31: Highest Combined Score in the Advanced B and
  Excellent B Classes. Per-trial RHC award.
- Chapter 1, Section 32: Highest Scoring Triple Qualifying Score
  (Advanced B + Excellent B + Master). Per-trial RHTQ award.
- Chapter 3, Section 15: Rally Advanced Excellent Title. RAE earned
  by qualifying scores in BOTH Advanced B and Excellent B at 10
  separate licensed or member rally trials.
- Chapter 4, Section 2: Championship Points (RACH). Points recorded
  for scores >= 91 in Advanced B, Excellent B, and Master; minimum
  150 of the required 300 points must come from Master; qualifying
  scores in all three classes on the same day at 20 separate trials.
- Chapter 4, Section 4: Rally Champion Title. Defines the RACH suffix
  and the numeric designation for repeat earners.

**Obedience Regulations (`akc_obedience_regulations_2025_03.pdf`):**

- Chapter 1, Section 22: Highest Scoring Dog in the Regular and
  Preferred Classes and Highest Combined Score in the Regular and
  Preferred Classes. Per-trial HC award computed from Open B + Utility
  combined scores.

## Investigation outcome: Master + Choice is NOT an AKC combined award

A 2026-04-25 investigation confirmed that the AKC Rally Regulations
do not define any "Master + Choice" combined award. Section 24 of
Chapter 1 enumerates exactly two combined-entry paths (RAE and RACH)
and Section 32 defines the per-trial Highest Scoring Triple Qualifying
award (Advanced B + Excellent B + Master). Choice never appears in any
combined-entry or combined-award definition in the rulebook.

The "Master + Choice" wording on the GFKC June 2026 Rally premium
list is a club-side fee discount GFKC chose to offer for dogs entered
in both classes, NOT an AKC-recognized combined award. The combined
award groups seed CSV (lands in CHECKPOINT 2) reflects only the
AKC-recognized paths.
