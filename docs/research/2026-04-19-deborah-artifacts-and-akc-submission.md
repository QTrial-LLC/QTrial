# Research capture: Deborah's artifacts and AKC submission mechanics

**Date:** 2026-04-19
**Participants:** Robare Pruyn, Deborah Pruyn (trial secretary, Glens Falls Kennel Club), Claude
**Purpose:** Capture the ground-truth findings from Deborah's artifact dump and the follow-up AKC regulatory research that resolves several PENDING items flagged across the design documents.

---

## Context

Over the course of texting on 2026-04-19, Deborah walked us through her trial-weekend workflow piece by piece and then sent a set of real artifacts from her November 2025 Glens Falls Kennel Club Rally trial (Event #2025103018, held 11/15/2025, with Saturday and Sunday each as their own AKC event numbers). She also provided screenshots of Obedience Solution (Lab Tested Databases) in use. This document preserves the actual content of what she sent and what it tells us, in her language where possible, so later contributors can see the evidence rather than just the conclusions.

---

## Artifacts received

**PDFs from the November 2025 Glens Falls Kennel Club Rally trial:**
- `Nov_2025_AKC_Rally_Trial_Judging_schedule.pdf` - the Saturday and Sunday judging schedules
- `Judges_Book_Cover_Sat.pdf` - the standard AKC cover page that precedes each class in the judges' book bundle, one per class
- `Judges_Book_Sat.pdf` - the interior judge's book pages with armband, breed, scoring fields, transfer section
- `Nov_2025_Sat_Marked_Catalog.pdf` - the marked catalog (Saturday) with full entry data, scores, placements, HC and High Triple calculations
- `Stewards_BOard_Sat.pdf` - the Steward's board layout showing ring, class, judge, dog counts, start times, with armband-numbered score lines
- `Trial_Summary_report.pdf` - the completed **JOVRY8** (Report of Rally Trial) form Deborah filled out and submitted

**Obedience Solution screenshots:**
- Main Menu, Trial Information Menu, Exhibitor/Dog Data menu, Armbands submenu
- Exhibitors/Dogs/Trial Entry screen
- Obedience Trial Entry screen (bottom and top halves)
- Class dropdown showing "Rally Novice A / Transfer to Rally Novice B / Rally Novice B / Transfer to Rally Intermediate..." pattern
- Judging schedule screen with "# of entries", "Time for class H:M", "Ring", "Start time", "Order" columns

---

## Key findings

### 1. JOVRY8 is the Rally submission form. JOVOB7 is the Obedience/Conformation equivalent.

The trial summary Deborah filled out is AKC form **JOVRY8 (10/25) v1.0 Edit** - the Report of Rally Trial form. This is the Rally analogue of JOVOB7, the Report of Dog Show or Obedience Trial. Both forms were updated in October 2025. Both are fillable PDFs.

Deborah uses JOVRY8. She had not heard of JOVOB7 by name; she called it "the trials summary report" and noted "AKC always has weird numbers on their docs." This matters because our documentation should refer to these forms by both their human name (Report of Rally Trial, Report of Dog Show or Obedience Trial) and their AKC form number for precision.

Key facts printed on JOVRY8 (10/25):
- Items required at AKC within 7 days of the event: marked catalog, judges books, this form with both pages completed, payment attached
- **Email submission is explicitly encouraged**, not just allowed: *"We encourage the club to scan this report, marked catalogs, and all required event records into PDF format and submit them as email attachments to RallyResults@akc.org."* This is in AKC's own voice on the form.
- Email address for Rally results: `RallyResults@akc.org`
- Mail address: American Kennel Club, Event Operations - Rally, PO Box 900051, Raleigh, NC 27675-9051
- Tel 919-816-3538, Fax 919-816-4220

Key facts printed on JOVOB7 (10/25):
- Email address for Conformation/Obedience results: `ConfObedResults@akc.org`
- Same 7-day submission window
- Same mail address template

Both forms direct dog-aggression incidents and Event Committee complaints to `eventrecords@akc.org` or fax 919-816-4220 within 72 hours of the event.

### 2. AKC's current fee schedules. 2026 brings fee increases.

**Rally (per JOVRY8 10/25):**
- Flat recording fee of $3.50 per entry for 2025 events
- **Flat recording fee of $4.50 per entry for 2026 events**
- No longer a split of "$3.50 first / $3.00 additional" (that language appears in older rulebooks but is superseded by the current form)
- Event Secretary Fee: free for first 12 Rally trials per year, then $10 per event
- Exclusions: no recording fee for Rally non-regular classes or Rally special attractions; entries withdrawn after closing (judge change, bitch in season) are not subject to recording fees and are not counted

**Conformation/Obedience (per JOVOB7 10/25):**
- Recording fee: $0.50 per first entry of each dog
- Service fee: $3.00 per first entry of each dog for 2025 events, **$4.00 for 2026 events**
- Service fee for each additional entry by the same dog: $3.50 for 2025 events, **$4.50 for 2026 events**
- Event Secretary Fee: free for first 8 Conformation/Obedience events per year, then $10 per event
- Exclusions: Junior Showmanship, Sweepstakes, Futurities, Maturities, multi-dog classes (brace/team), Obedience non-regular classes, Special Attractions

**Implication for OffLeash:** Fee calculation is not a constant; it's year-dependent and sport-dependent and has different thresholds per sport. The fee structure needs to be modeled as time-varying reference data, not hardcoded. Our MVP needs to generate a correct calculation for 2026 and accommodate the 2027+ values whenever AKC publishes them.

**The "glitchy" AKC online form.** Deborah's remark: *"the last time I tried to fill one out very very glitchy. It wouldn't do the totals plus there was a thing about dual classes that didn't have in it, so I don't trust the AKC."* The filled JOVRY8 she sent shows 93 entries calculated at $325.50 = 93 × $3.50 exactly, which is the correct Rally flat rate. So the "glitchy" behavior is likely in the form's field-level automation (totals not computing in certain browsers / PDF readers), not in the fee rules themselves. The "dual classes" comment remains unresolved - it may refer to Rally-plus-Obedience on the same day being handled awkwardly in the single form, or to some other edge case. Ask Deborah.

### 3. Rally Choice is a real, relatively recent AKC class.

Rally Choice is an official regular Rally class, effective **2023-06-29** per third-party sources and confirmed by AKC's 2024+ Rally Regulations. It appears in Deborah's Saturday catalog with 14 entries (armbands 801-814).

Key facts:
- **Title:** Rally Choice (RC), earned after 10 qualifying scores. RC2 for 20, RC3 for 30, etc.
- **Eligibility:** all dogs are eligible, may continue indefinitely (no class progression or A/B distinction)
- **No jumps** in Rally Choice
- All signs are judged **off leash**
- Verbal encouragement, multiple commands, inaudible signals with one or both arms/hands are allowed
- Penalties for clapping, patting legs, touching the dog, physical corrections
- Incorrectly performed sign on first attempt = 10-point deduction (no retry)
- 100-point max score, same as other Rally classes
- **Scheduling convention:** Rally Choice and Rally Master are interchangeable at the top of the day. The Rally Regulations specifically say the order may vary as "Master or Choice (Choice or Master), Excellent, Advanced, Intermediate, Novice" or the reverse.

**Implication for OffLeash:** Rally Choice is a 7th Rally class not currently in DOMAIN_GLOSSARY or REQUIREMENTS §2.5. It's a Regular Rally class with no A/B split, no jumps, and no title progression limit. This must be in the MVP canonical class catalog for Rally.

### 4. Rally High Triple Qualifying (RHTQ) is a real award.

The marked catalog ends with both a High Combined (HC) calculation and a **High Triple Qualifying** calculation:
- HC: 524 / 200 (armband 524 earned two 100s, combined 200)
- High Triple Qualifying: 524 / 300 (armband 524 earned three 100s across three classes, combined 300)

Per the Rally Regulations table of contents, this is formally "Highest Scoring Triple Qualifying Score" and is a defined award in Chapter 1 Section 32. It's the Rally analogue of High Combined but for three specified classes instead of two (typically Advanced B + Excellent B + Master).

**Implication for OffLeash:** Our DATA_MODEL `trial_awards.award_type` enum currently has `hit`, `hc`, `phit`, `phc`, `rhc`, `htq`. We need to verify what `htq` means (Honor Team Qualifier per DOMAIN_GLOSSARY) and add or rename to include **Rally High Triple Qualifying (RHTQ)** explicitly. Existing `rhc` is Rally High Combined.

### 5. A/B class eligibility differs between Obedience and Rally.

Per AKC's official guidance and per Deborah's explanation:

**Obedience A/B classes:** eligibility is **handler-based**.
- "A" class: handler has never put a novice title on a dog (or the class's corresponding title for higher levels)
- "B" class: handler has previously titled a dog at this level, or handler is anyone else

**Rally A/B classes (Novice and Advanced and Excellent levels):** eligibility is **dog-based**.
- Rally Novice A: the dog has never earned a Rally title; handler must own the dog or be a family member of the owner. Dogs may compete in Novice A until the Rally Novice title is earned.
- Rally Novice B: any dog. Handlers may be anyone.
- Rally Intermediate, Master, and Choice: **no A/B distinction at all.** Intermediate is a single class.
- Rally Advanced A: dog has earned Rally Novice (RN) but has NOT earned the Rally Advanced (RA) title, AND handler has never titled in Obedience and has never earned RA with any dog
- Rally Advanced B: any dog that has earned RN
- Rally Excellent A: dog has earned RA but has NOT earned RE, AND handler has never titled in Obedience and has never earned RE with any dog
- Rally Excellent B: any dog that has earned RA

**Promotion to B:** Once a dog earns the Novice title (in Rally), it's automatically promoted out of Novice A eligibility and into B-only for subsequent entries at that level.

**Implication for OffLeash:** Eligibility checks for A class entry are fundamentally different queries:
- Obedience: "has this handler ever earned title X with any dog?"
- Rally: "has this dog ever earned title X?" (and for Advanced/Excellent, "has this handler ever earned the same title with any dog, and has the handler ever titled in Obedience?")

Our DATA_MODEL needs title tracking at **both** the dog level (already present via `dog_titles`) and the handler level (which is the user's history across all dogs they've handled - not currently modeled as a first-class concept). Handler title history can be computed as an aggregate view over all entry results for entries where the handler was the entering user, but needs care because the "handler" field on an entry may be different from the entering user.

### 6. Transfers in AKC Obedience/Rally are a distinct concept from move-ups.

Per AKC's Obedience Transfer FAQ and the Obedience Transfer Form (January 2023):

**Transfer (formerly "Move-up"):**
- Allowed only if the host club opts in. Premium list must state whether transfers are allowed; default is to allow if not stated.
- Triggered when a dog earned a title between the trial's closing date and the trial's start, making the currently-entered class no longer appropriate
- Request must be in writing, presented to the trial secretary at least 30 minutes prior to the start of the relevant trial
- Subject to class availability and class not having reached its limit
- Covers: Obedience class → Obedience class at same sport; Rally class → Rally class at same sport
- A-to-B at the same level is also a transfer if the host club allows it

**Movement between A and B at the same level is not a transfer** if it's because the dog's eligibility naturally changed (e.g., earning the Novice title automatically promotes to B). That's handled in regular entry validation.

**Obedience Solution's UI insight:** The class dropdown in Obedience Solution explicitly lists "Rally Novice A / Transfer to Rally Novice B / Rally Novice B / Transfer to Rally Intermediate / ..." as separate selectable values. This suggests that in her current workflow, transfers are **pre-entered at entry time** as an intent flag ("I'm entering Rally Novice A but if my dog earns the RN before the trial, transfer me to Novice B"), rather than purely a post-entry state change. Need to ask Deborah whether transfers are sometimes pre-elected at entry time or whether the dropdown entries are used to record transfers retroactively after they happen.

**Implication for OffLeash:** Our current REQUIREMENTS §5.2 and DATA_MODEL treat transfers purely as a post-entry state transition. The Obedience Solution pattern suggests we may also need a pre-entry transfer-intent flag. This is a small addition but important for UX parity with what Deborah already knows.

### 7. The catalog format has a precise structure that we now have ground truth for.

Each entry in the marked catalog is formatted as:

```
REGISTERED NAME (in uppercase). REG_NUMBER. DOB. SEX. BREED.
Breeder: NAMES. By SIRE_REGISTERED_NAME - DAM_REGISTERED_NAME.
Owner(s): NAMES, FULL_ADDRESS. Handler: NAME. [Jumps XX inches]
[Also in CLASS1, CLASS2]
[free-text judge annotation, e.g. "left ring", "Lack of teamwork"]
```

Followed on its own line by: `PLACEMENT SCORE ARMBAND` (when scored) or just `ARMBAND` (pre-scoring).

Key formatting observations:
- Registered name is uppercase
- Date of birth format: `M/D/YYYY` (no leading zeros)
- Sex written as "Bitch" or "Dog"
- Breed spelled out in full ("All American Dog", "Pembroke Welsh Corgi", "German Shepherd Dog")
- PAL dogs show "PAL" prefix on the registration number and have birth year but no sire/dam data (shown as "Unknown" / "By - ")
- All American Dogs (mixed breed) also have birth date but "Breeder: Unknown. By Unknown - Unknown."
- Prefix titles (GCH, CH, OTCH, RACH, CD, etc.) precede the registered name: `GCH OSIRIS EVERYDAY I'M TRUFFLIN BCAT` has `GCH` as prefix then the full registered name with `BCAT` shown as a suffix title.
- Actually looking more carefully: `GCH OSIRIS EVERYDAY I'M TRUFFLIN BCAT` - here GCH is a prefix title and BCAT is a suffix. The catalog prints them as `PREFIXES NAME SUFFIXES` in all caps with spaces.
- Another example: `RACH RHUMBLINE'S GIMMIE SHELTER CD BN RM4 RAE4 ACT2 NAP NJP CGC TKN OAP NFP` - RACH is prefix, everything else after the name is suffixes
- Another: `MACH 2 COLLINSWOOD A LITTLE BIT LOUDER NOW` - MACH 2 is a prefix title, name follows, no suffixes. "MACH" titles use numeric increments (MACH, MACH2, MACH3...) spelled with a space before the number
- Address format: `Street, City, State Zip` (e.g., "47 Trull St, Covoes, NY 12047"). Note "Covoes" is a misspelling of "Cohoes" in the source data, preserved as-is.
- "Also in..." appears at the end of the entry on the same or adjacent line, listing other classes the dog is entered in at this trial
- Free-text judge annotations ("left ring", "not working", "no working", "unmanageable", "Lack of teamwork") appear as a separate line preceding the placement/score line, representing the judge's handwritten reason for non-qualifying or excusal

**Jump height subheadings** break up the class listing when jumps are involved:

```
8 inches
ARMBAND   DOG INFO...
12 inches
ARMBAND   DOG INFO...
16 inches
ARMBAND   DOG INFO...
```

**Class summary at end of each class:**
```
1st ________ 2nd ________ 3rd ________ 4th ________
Score _______ Score _______ Score _______ Score _______
Total Entries in [Class Name]: N
[ARMBAND1] [ARMBAND2] [ARMBAND3] [ARMBAND4]
[SCORE1]   [SCORE2]   [SCORE3]   [SCORE4]
Total Competing in [Class Name]: M
```

Where N = total entries including absentees, M = dogs that actually competed. Blank placement slots show as `----`.

**End-of-day summary** (appears in Rally Choice as the last class):
```
High Combined __________ [ARMBAND] [COMBINED_SCORE]
High Triple Qualifying __________ [ARMBAND] [COMBINED_SCORE]
Score __________
```

### 8. The judge's book has a standard AKC format.

Each class in the judges' book has:
- A **cover page** with AKC boilerplate: certification language, a 4-step "Procedure for Judges to follow" checklist (absentees, winners, time started/finished, signature), and "DISTRIBUTION OF COPIES: White: American Kennel Club, Yellow: Club, Pink: Judge, Gold: Post"
- A header: club name, AKC event number, judge name, date
- The class name and max score
- Entry listings broken down by jump height where applicable, each showing armband, breed, and three blank fields: **Time, Points Lost, Final Score**
- A **Transfers** section for dogs that transferred into this class after entries closed
- Placement summary: `1st ___ 2nd ___ 3rd ___ 4th ___` with corresponding score fields and armband fields
- **Time Started** and **Time Finished** fields

This is the exact form our Phase 3 judge's book generator must produce.

### 9. The steward's board is a distinct artifact.

What we received as `Stewards_BOard_Sat.pdf` is not a "board" in the physical sense but a printed running order with scoring lines. Format:
- `Ring X - Class Name / Day / Judge: Name / N dogs / Start time`
- Per-entry line: `ARMBAND BREED [JUMP_HEIGHT_IF_APPLICABLE]`
- Spans multiple columns or multiple pages when long

This is posted at ringside so exhibitors can see running order and stewards can track who has run.

**Implication for OffLeash:** The Steward's Board is a distinct report from the running order (REQUIREMENTS §7.1) and the judge's book (§9). We should name and scope it as its own deliverable.

### 10. Running schedule details from the Obedience Solution screenshot and Deborah's explanation.

Deborah: *"That's the only thing when setting up a running schedule there is a pro forma set amount time that the AKC says is allotted when you're doing the timing for a run which is three minutes which usually is something that doesn't work, but it's the way you have to do it and then you have to give time in between classes for a walk-through because in Rally everybody gets to walk the course before they bring their dogs in for their run and then after that class is finished, you have to give time for ribbons where people get their placement, ribbons, and any titles that they may have gotten."*

The Obedience Solution judging schedule screen shows, for each class:
- `# of entries`
- `Time for class H:M` (computed from entries × dogs-per-hour rate)
- `Ring`
- `Start time` (editable dropdown)
- `Order`

And a footnote: "Hours : Minutes (H : M) based on AKC recommended dogs per hour."

Per current Rally Regulations: **"The judging program will be based on the judging of up to 20 dogs per hour"** (Chapter 1 Section 23 says "up to 18 entries per hour" in the Rally news page; Chapter 2 Section 4 says "up to 20 dogs per hour" - there's some internal AKC inconsistency here).

The three-minute-per-dog figure Deborah cited is likely per-run, not per-judging-hour rate. A 20-dog-per-hour rate is 3 minutes per dog, which matches.

**Schedule components per Deborah:**
1. Per-dog judging time (derived from dogs-per-hour rate)
2. Walk-through time before each class (Rally requires a walk-through; see Rally Regulations Chapter 2 Section 25)
3. Ribbon/placement/title presentation time after each class
4. Inter-class transition / set-up time

**Implication for OffLeash:** REQUIREMENTS §7.3 has "class change time 30 sec default" and "event change time 45 sec default" - these are the 2003-era Obedience-specific defaults from the current Access schema, but they don't capture walk-through time or ribbon-presentation time, which are the real schedule components for Rally. We need to model schedule components explicitly:
- `per_dog_minutes` (derived from class's dogs-per-hour rate)
- `walkthrough_minutes` (class-specific; Rally needs one, some Obedience classes do, others don't)
- `ribbon_presentation_minutes` (default per-class estimate)
- `class_change_minutes` (set-up/tear-down between classes)

### 11. Trial day realities.

Deborah: *"That would be the other thing that would be set up on the trial screen. When you first set up the trial is the time the trial is actually going to start. I usually arrive at around 7 AM and the trial usually officially starts at around eight or 8:30 AM. Scoring starts happening as soon as the first dog completes its round."*

**Implication for OffLeash:** Our `trials.start_time` should be the **official start time** (the published time in the premium list and the judging schedule). Secretary arrival time is a separate concern and doesn't need to be modeled in the trial record itself (Deborah can put it in a personal note field if she wants; no one else needs to see it). Scoring is asynchronous from start-of-trial and begins whenever the first dog runs.

### 12. Armband numbering uses class-specific ranges.

From the judging schedule:
- Rally Novice A: 101-104 (4 dogs)
- Rally Novice B: 202-209, 212 (9 dogs, gaps because 210, 211, and 201 either don't exist or were cancelled)
- Rally Intermediate: 301-310
- Rally Advanced A: 401-404 + 103 (the 103 is a dog cross-entered from Rally Novice A whose armband stayed)
- Rally Advanced B: 502-505, 508-515, 518, 520-522, 524, 526
- Rally Excellent A: 701
- Rally Excellent B: 502-505, 508-513, 515, 518, 520-522, 524 (re-used from Advanced B for same dogs)
- Rally Master: 501-502, 506-507, 509-512, 515-517, 520, 524-525, 527-529
- Rally Choice: 801-814

Two important patterns:
1. **Class range convention:** 100s = Novice A, 200s = Novice B, 300s = Intermediate, 400s = Advanced A, 500s = Advanced B + Excellent B + Master (shared ranges because many dogs enter multiple of these), 700s = Excellent A, 800s = Choice
2. **Same armband across classes for same dog:** Armband 509 (Rhumbline's Gimmie Shelter, Labrador Retriever) appears in Advanced B, Excellent B, Master, and Choice all with the same number. The armband identifies the dog across all classes at this trial, not the dog-in-class.

This confirms that our DATA_MODEL `entry_lines.armband` should instead be (or be derived from) an armband assigned at the `entries` level per (dog × event), not per (dog × class). The legacy schema's `armband_scheme` enum with value `per_class` is misleading - even the "per_class" scheme at Deborah's club assigns per-dog-but-with-class-ranged-starting-points, not per-class-per-dog.

**Actually correcting:** Looking at this more carefully, armband 103 appears in both Novice A (as first entry) and Advanced A (listed at bottom). So this dog is cross-entered in both classes, with armband 103 assigned from the Novice A range because that was its first class entered. The dog keeps that armband in other classes at the same trial.

**Implication for DATA_MODEL:** `entries.armband` is a better home than `entry_lines.armband`. The armband assignment scheme determines the numeric range but the armband itself is per-dog-per-event, not per-dog-per-class.

### 13. Reverse running order on day 2.

From the Sunday judging schedule:
- Rally Choice Sunday: 814-807, 805-801 (descending, Saturday was 801-814 ascending)
- Rally Master Sunday: 502-501, 507-506, 529-527, 525-523, 520-519, 517-516, 512-509 (descending and grouped)
- Similar pattern for other classes

This is a fairness rotation: dogs that ran early Saturday run late Sunday, and vice versa.

**Implication for OffLeash:** REQUIREMENTS §7.1 lists "short_to_tall, tall_to_short, random, manual" running order strategies. Add: **`reverse_previous_day`** as a strategy. Or simpler: let the secretary choose a base strategy per class per day, and offer a "mirror yesterday" convenience option.

### 14. Obedience Solution feature surface reveals missing requirements.

The menu screenshots show features we don't have in REQUIREMENTS:

**Exhibitor/Dog Data menu:**
- Exhibitor / Dog Data and Trial Entry ✓ (covered)
- AKC Ineligible Dogs - a list of dogs that cannot be entered (for whatever AKC-disciplinary reason)
- Deposit Checks - a check-reconciliation workflow, explicitly tracking which physical checks have been deposited
- Exhibitor / Dog Lists - reporting
- Entering History - per-exhibitor trial history view
- Exhibitor Mailings - mass mailing (we have this as REQUIREMENTS §15, mailing list)
- Delete Exhibitors by Year - retention/purge workflow
- Secretary Return Address Labels - printable labels for paper mail returns
- Obedience Transfer Form - generates a pre-filled transfer form
- Rally Transfer Form - ditto

**Trial Information Menu:**
- Club Information
- Secretary Information
- Judge Information
- Trial Information
- List of Trials
- Available Classes ✓ (covered)

**Armbands submenu:**
- Assign Armband Numbers ✓
- View Armband Assignment ✓
- List of Dogs by Armband #'s ✓
- Dogs without Armband Numbers - validation report (any dog missing an armband)
- **Handler Conflicts** - detection of situations where the same handler is scheduled to handle two dogs in the same ring at the same time; needed because a handler can't be in two places at once
- Print Armbands ✓

**New features to add to REQUIREMENTS:**
- AKC Ineligible Dogs list (dogs that may not be entered due to AKC disciplinary action, separate from regular eligibility checks)
- Check deposit reconciliation as its own workflow (not just a field on payments)
- Pre-filled Obedience Transfer Form and Rally Transfer Form generation
- Return address labels for the secretary (small PDF output for mailing refund checks, etc.)
- Dogs-without-armband validation report
- Handler conflict detection

### 15. "Current Event #17" in Obedience Solution is a local-install counter.

The screenshots show "Current Event: #17" in the Obedience Solution header. Given that Glens Falls has been using this software for years and holds multiple trials per year, this is almost certainly a per-install sequence number local to Deborah's copy of Obedience Solution, not any AKC-assigned identifier. Migration from the `.mde` file should not assume this number has meaning beyond the local database.

### 16. "Ring 0" is a placeholder or bug in Obedience Solution.

The judging schedule, judges' books, and marked catalog all show "Ring 0" or "Ring #0" even though Glens Falls is clearly running a single ring (the schedule shows only one ring throughout both days). This is either (a) the default ring number Obedience Solution assigns when no ring data is entered, or (b) a deliberate convention Deborah's club uses. Ask Deborah, but I suspect (a).

**Implication for OffLeash:** Don't replicate this. Our `trial_class_offerings.ring_number` should require a value or default sensibly to 1.

---

## Summary of changes this drives in the design docs

This list is the input to the doc-update pass:

**DOMAIN_GLOSSARY.md**
- Correct A/B class definitions: handler-based for Obedience, dog-based for Rally, and note which levels have A/B distinction in Rally (Novice, Advanced, Excellent only)
- Add Rally Choice class
- Add JOVOB7 and JOVRY8 form identifiers
- Rename or disambiguate "HTQ" vs Rally High Triple Qualifying (RHTQ); the existing `htq` (Honor Team Qualifier) appears to be a Rally Team-specific award, distinct from RHTQ which is the Rally analog of HC
- Add definitions: Transfer (Obedience/Rally), Recording Fee, Event Service Fee, Event Secretary Fee, AEN999

**REQUIREMENTS.md**
- §2.3: note that an event's AKC event number is per-sport-per-day-per-trial (confirmed from Glens Falls having different event numbers for Saturday vs Sunday on the same physical trial)
- §2.5: add Rally Choice to Rally class list
- §3.3: revise entry eligibility validation to distinguish Obedience handler-based A/B from Rally dog-based A/B
- §5.2: add pre-entry transfer-intent concept alongside post-entry transfers
- §7.3: replace or augment the "class change / event change" time defaults with explicit schedule components: per-dog-time, walkthrough-time, ribbon-presentation-time, class-transition-time
- §8.2: add Rally High Triple Qualifying (RHTQ) to computed awards
- §10: add the concrete catalog format specification from finding #7
- §9: add the concrete judge's book specification from finding #8
- Add new §10a or similar: Steward's Board generation (separate from catalog and running order)
- §13: update fee calculation language to reflect year-dependent per-sport schedules, including the 2026 increases
- §14: major rewrite - the submission mechanism is confirmed as email + paper-attachment PDFs, no XML schema. Sport-specific email addresses (`RallyResults@akc.org`, `ConfObedResults@akc.org`, `eventrecords@akc.org` for incidents). JOVRY8 and JOVOB7 form generation is core. 7-day submission deadline. 5-day Event Committee incident submission. 72-hour dog-aggression incident submission.
- Add new section for features found in Obedience Solution but not yet in REQUIREMENTS: AKC Ineligible Dogs list, check deposit reconciliation, transfer form generation, return address labels, dogs-without-armband report, handler conflict detection
- Update §2.2 fee categories to match current AKC fee structure, including the 2026 flat Rally rate vs Obedience split rate

**DATA_MODEL.md**
- `trials.start_time` stays as the official start time (unchanged conceptually, but document the intent)
- `entries.armband` - move armband from `entry_lines` to `entries` (per-dog-per-event), update `armband_scheme` semantics accordingly
- `canonical_classes` - add Rally Choice; revise class_type and earning rules; add fields for the per-sport eligibility rule type (handler-based vs dog-based)
- `trial_class_offerings` - add `walkthrough_minutes`, `ribbon_presentation_minutes`, `class_transition_minutes`
- `trial_awards` - verify/clarify `htq` meaning; add `rhtq` (Rally High Triple Qualifying) explicitly
- Add `entry_line_results.judge_annotation_text` as free-text judge commentary (current `entry_line_results` has `reason` which might serve, but it's worth separating the "status reason" from the catalog-printed annotation)
- Add new table `akc_fee_schedules` for year-scoped per-sport fee reference data
- Add new concept/table for handler conflict detection (probably a computed view, not a table)
- Add `dogs.is_akc_ineligible` flag or related table for AKC-disciplinary ineligibility
- Document that the event `akc_event_number` is per-sport-per-day-per-trial

**WORKFLOWS.md**
- Fill in many of the PENDING items now confirmed:
  - §4: paper-entry workflow, what Deborah does with paper and check physically
  - §5.3: bitch-in-season refund - confirmed that withdrawn-after-closing entries are not subject to recording fees; this affects the report-generation math
  - §7.1: armband numbering practice at Glens Falls uses class-range-based starting points per dog-per-event
  - §7.2: catalog generation format now has concrete spec
  - §7.3: judge's book format now has concrete spec
  - §9.1-§9.2: AKC submission - the actual workflow is "scan marked catalog + judges' books + JOVRY8/JOVOB7 + payment info into a single PDF package, email to sport-specific address, or mail if preferred"
- Add a section for Steward's Board generation
- Add a section for transfer-form generation

**ARCHITECTURE.md**
- Update the "AKC" bullet in external integrations: the integration is email submission with PDF attachments, not an API or XML schema
- Note that sport-specific email addresses need to be modeled and configurable (they could change)

**ROADMAP.md**
- Reframe Phase 6 from "AKC electronic submission" to "AKC submission package generation" - producing JOVRY8/JOVOB7 PDFs, marked catalog PDFs, judges' book PDFs, and an email-compose flow that Deborah reviews before sending
- The XML-schema risk in the risk register is mitigated: there isn't one to target
- The "engagement with AKC" item in Phase 6 is lower priority than originally thought - AKC's documented workflow is already sufficient for MVP. Engagement may still be useful to confirm they receive the emails and process them correctly.

**PROJECT_CHARTER.md**
- "Working assumptions" section: resolve the "AKC still accepts the 2004-era XML schema" line - they never did, it was a vendor-internal artifact. Update to reflect that AKC's current process is email-attached PDF packages to sport-specific addresses.

---

## Things still unresolved, suitable for Deborah

See `docs/research/2026-04-19-questions-for-deborah.md` for the consolidated list.
