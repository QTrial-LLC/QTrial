# Sample real-world artifacts

Filled-out, real-world examples from past Glens Falls Kennel Club
(GFKC) trials. These are reference material for QTrial's
document-generation work: each PDF here is what QTrial should
eventually produce for the corresponding artifact type, or what the
secretary consumes from AKC during the workflow.

Provided by Deborah Pruyn (GFKC trial secretary). **DO NOT
redistribute publicly.** Dog call names, owner names, and addresses
from real entries appear in some of these artifacts; they are
here as internal QTrial design reference and must not leak to
external surfaces (GitHub public repos, blog posts, screenshots,
etc.). The repo is private today; assume that invariant holds and
avoid propagating these files into logs, caches, or CI artifacts.

Filename convention:

    gfkc_<sport>_<artifact_type>_<YYYY_MM[_DD][_day-marker]>.pdf

where the YYYY_MM prefix sorts trials chronologically and the
optional DD and day-marker disambiguate multi-day trials.

| File | Trial | Artifact type |
|---|---|---|
| `gfkc_rally_premium_2026_06.pdf` | GFKC Rally, Saturday-Sunday June 20-21, 2026 (events #2026103004 Sat, #2026103005 Sun) | Premium list PDF. First two pages carry club officers, show committee, trial chair (Chris Argento), trial secretary (Debbie Pruyn), entry-open and entry-close dates, fee table, and judging schedule. Reference for REQ-PREMIUM-* generation. Confirms Q5 (trial chair vs trial secretary) and Q6 (officers are club-level) from Deborah's 2026-04-23 email. |
| `gfkc_rally_judges_book_cover_2025_11_15_sat.pdf` | GFKC Rally, Saturday November 15, 2025 (event #2025103018), Rally Novice A class | Judge's Book cover page for one class. Shows the 4-item Procedure for Judges to Follow and the Judge's Certificate block. Header fields: Event Number, Class, Name of Club, Date, Judge, Ring, Scheduled Starting Time, Total Number of Dogs. Multiple design docs reference this file as the canonical judges-book-cover exemplar. Note: the "Ring: 0" value is an Obedience Solution default (flagged in the 2026-04-19 research note §16), not a real ring number. |
| `gfkc_rally_judges_book_2025_06_21.pdf` | GFKC Rally, Saturday June 21, 2025 (event #2025103007), Rally Novice A class, Judge Denise LaCroix | Judge's Book interior page, with six dogs listed (armbands 101, 102, 103, 105, 106, 107) by armband number and breed of dog. Confirms the per-Deborah-2026-04-23 column set: Armband #, Breed of Dog, Time, Points Lost, Final Score, with a placement block (1st-4th) at the bottom. No call names, handler names, or owner names appear on the scored pages. |
