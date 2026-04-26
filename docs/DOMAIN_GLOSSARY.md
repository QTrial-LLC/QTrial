# QTrial - Domain Glossary

**Status:** Draft v0.2
**Last updated:** 2026-04-20
**Purpose:** Precise definitions of dog-sport and AKC terminology used throughout QTrial. LLMs and new contributors consistently guess these terms wrong if they are not explicitly defined. This document is authoritative; if code, comments, or other docs use a term differently, they are wrong and must be corrected.

---

## The event hierarchy

**Event**
A single approved AKC gathering at a specific location over one or more consecutive days, hosted by one club. An event has one AKC event number *per sport per day per trial* (see Trial below). "Glens Falls Kennel Club Obedience and Rally Trials, November 14-15, 2025" is one event.

**Day**
A calendar date on which the event is being held. An event can be one day or span multiple days (typically 1-3).

**Trial**
A single competition unit within a day for a single sport. A day can contain multiple trials of the same sport, most commonly two: a morning (AM) trial and an afternoon (PM) trial. Each trial has its own AKC event number, its own judge assignments, its own entry limits, its own entry fees, and its own awards (HIT, HC, etc.). This three-level hierarchy (event → day → trial) is a critical data modeling concern.

**Sport** (also called "Event type" in some AKC documents)
The overall discipline. Examples: Obedience, Rally, Agility, Scent Work, Conformation, FastCAT, Barn Hunt, Lure Coursing, Herding, Tracking, Field Trial. A single event can host trials for multiple sports concurrently (e.g., Obedience trial and Rally trial on the same day, same location, different rings).

**Cluster**
An informal term for a series of events held at the same site over several consecutive days, often by different clubs. "The St. Louis cluster" might run for a full week with different clubs hosting different days. QTrial does not need first-class cluster modeling in MVP; clusters are just sequential events from the system's perspective.

## Classes, levels, and divisions

**Class**
A specific competition category within a sport at a specific level. Examples in Obedience: "Novice A", "Novice B", "Open A", "Open B", "Utility A", "Utility B", "Beginner Novice", "Graduate Novice", "Versatility", "Veterans Obedience", "Brace Obedience", "Team Obedience". In Rally: "Rally Novice A", "Rally Novice B", "Rally Intermediate", "Rally Advanced A/B", "Rally Excellent A/B", "Rally Master", "Rally Pairs".

**Level**
The regulatory category of a class. AKC categorizes classes as Regular, Optional Titling, Preferred, or Nonregular. Each has different eligibility rules and titling implications.

**A vs B classes**
Within many classes (Novice, Open, Utility, Rally Novice, Rally Advanced, Rally Excellent), the "A" version is for dogs and/or handlers who have not yet earned the title at that level, and the "B" version is for everyone else. The distinction affects eligibility. A dog cannot be entered in both A and B at the same level on the same day; moving between A and B mid-event is a "transfer" (see below).

**Regular class**
A class at which an AKC title can be earned through normal competition. Novice, Open, Utility are Regular Obedience classes.

**Optional Titling class**
A class that awards an AKC title but is not one of the main progression classes. Beginner Novice, Graduate Novice, Versatility are Optional Titling classes in Obedience.

**Preferred class**
A class that mirrors a Regular class but offers modifications (typically jump height reductions) for dogs that benefit from them. Earns a different title than the Regular class.

**Nonregular class**
A class that does not award AKC titles but is offered for variety, team competition, or fun. Veterans, Sub-Novice, Wildcard, Brace, Team, Rally Pairs, Random Reward are nonregular. Modeled in `canonical_classes` with `class_type = 'Nonregular'`; some nonregular classes also have `is_sanctioned = false` (e.g., Random Reward - a novelty class that doesn't earn AKC titles).

**Canonical class catalog**
The seeded 75-row master list of Obedience + Rally classes in `canonical_classes`. Spans:

- Regular Obedience: Novice A/B, Open A/B, Utility A/B
- Optional Titling Obedience: Beginner Novice, Graduate Novice, Graduate Open, Versatility, Preferred variants
- Nonregular Obedience: Brace, Team, Sub-Novice, Veterans, Wildcard, Random Reward (Novice/Open/Utility)
- Regular Rally: Novice A/B, Intermediate, Advanced A/B, Excellent A/B, Master, Choice
- Nonregular Rally: Pairs, Team variants, T Challenge, Plus, Intro

Additional sports (Agility, Scent Work, etc.) extend this catalog post-MVP.

## Qualifying, scoring, and titles

**Q (Qualifying score)**
A score that meets the minimum standard for the class. In Obedience, a Q requires at least 170 out of 200 with no exercise scored below 50% of its value. In Rally, a Q requires at least 70 out of 100. A dog earning a Q "qualified" at that class in that trial.

**NQ (Non-qualifying score)**
A score that fails the qualifying threshold. An NQ dog ran the class but did not qualify.

**Leg**
One qualifying score toward a title. Most AKC Obedience titles require three legs under three different judges (or different judges for at least two of the three, depending on the title). Each Q in a relevant class earns one leg.

**Title**
A credential earned by completing a defined sequence of qualifying performances. Obedience titles include CD (Companion Dog), CDX (Companion Dog Excellent), UD (Utility Dog), UDX (Utility Dog Excellent), OM (Obedience Master), OGM (Obedience Grand Master), BN (Beginner Novice), GN (Graduate Novice), GO (Graduate Open), VER (Versatility). Rally titles include RN (Rally Novice), RA (Rally Advanced), RE (Rally Excellent), RAE (Rally Advanced Excellent), RM (Rally Master), RAE2/RAE3/... (repeated Rally Advanced Excellent).

**Championship titles**
OTCH (Obedience Trial Champion) requires advanced qualifying scores and accumulated OTCH points. RCH (Rally Champion) has similar structure. These are prefix titles - they appear before the dog's registered name.

**Prefix title vs suffix title**
AKC titles are rendered either as prefixes before the registered name (CH, GCH, OTCH, MACH, CT, FC, AFC, etc.) or as suffixes after the registered name (CD, CDX, UD, UDX, BN, RN, RA, RE, etc.). A fully titled dog's name is rendered as "PrefixTitles RegisteredName, SuffixTitles" - e.g., "GCH OTCH Kensington's Moonlight Sonata UDX RAE". This formatting matters for catalogs.

**HIT (High in Trial)**
The top overall Obedience score across all regular classes at a trial. One HIT is awarded per trial.

**HC (Highest Combined)**
The Obedience per-trial award for the highest combined Open B + Utility B score among dogs that qualified in both classes. Defined in AKC Obedience Regulations Chapter 1, Section 22 (`db/seed/akc/regulations/akc_obedience_regulations_2025_03.pdf`). Modeled in QTrial as a `combined_award_groups` row with `code = 'akc_obedience_hc'` and `award_type = 'hc'`; the junction lists Open B and Utility B with `is_required_for_award = TRUE`.

**PHIT / PHC (Preferred HIT / Preferred HC)**
The Preferred-class equivalents of HIT and HC.

**RHIT / RHC (Rally HIT / Rally Highest Combined)**
Rally-specific honors. RHIT is the highest qualifying Rally score across all regular classes at a trial. RHC is the per-trial award for the highest combined Advanced B + Excellent B score among dogs that qualified in both, defined in AKC Rally Regulations Chapter 1, Section 31 (`db/seed/akc/regulations/akc_rally_regulations_1217.pdf`). Modeled as a `combined_award_groups` row with `code = 'akc_rally_rhc'` and `award_type = 'rhc'`.

**HTQ / RHTQ (Highest Triple Qualifying / Rally HTQ)**
The Rally per-trial award for the highest combined triple score across Advanced B + Excellent B + Master among dogs that qualified in all three at the same trial. Defined in AKC Rally Regulations Chapter 1, Section 32. Modeled as a `combined_award_groups` row with `code = 'akc_rally_rhtq'` and `award_type = 'rhtq'`. The schema's `award_type` ENUM has both `htq` and `rhtq` values; only the Rally form is currently seeded. (Earlier glossary editions translated HTQ as "Honor Team Qualifier"; the rulebook makes clear it is "Highest Triple Qualifying", a per-trial combined-score award, not a team-eligibility marker.)

**RAE (Rally Advanced Excellent)**
A Rally title earned by qualifying scores in BOTH Advanced B and Excellent B at 10 separate licensed or member rally trials. Defined in AKC Rally Regulations Chapter 3, Section 15. The same trial's combined entry fee for both classes is paid as one bundled entry under the combined-entry mechanism in Chapter 1, Section 24. Numeric variants (RAE2, RAE3, ...) accumulate as the dog meets the requirements again. Modeled as a `combined_award_groups` row with `code = 'akc_rally_rae'` and `award_type = NULL` (title-progression path; no per-trial ribbon).

**RACH (Rally Champion)**
A Rally championship title earned by qualifying scores in Advanced B, Excellent B, AND Master on the same day at the same trial across 20 separate licensed or member rally trials, plus 300 RACH points (minimum 150 from Master). Defined in AKC Rally Regulations Chapter 4, Sections 2 and 4. The combined-entry mechanism in Chapter 1, Section 24 lets exhibitors pay one bundled fee for the three-class entry. Numeric variants (RACH2 = 40 triple-qualifying scores + 600 points, RACH3 = 60 + 900, ...) accumulate. RACH is a prefix title; it appears before the dog's registered name. Modeled as a `combined_award_groups` row with `code = 'akc_rally_rach'` and `award_type = NULL` (title-progression path).

**Combined award**
An AKC-recognized award computed across multiple classes at the same trial, requiring the dog to qualify in every contributing class. Examples: Obedience HC (Open B + Utility B), Rally RHC (Adv B + Ex B), Rally RHTQ (Adv B + Ex B + Master). Distinct from a *combined-entry mechanism* (RAE / RACH), which is the entry-fee bundling AKC Rally Regulations Chapter 1, Section 24 defines for paying once for two or three classes that contribute to a title-progression path; the title itself accumulates across multiple trials. QTrial models both shapes through `combined_award_groups`: per-trial awards have a non-NULL `award_type`; title-progression paths have `award_type = NULL`. The fee engine applies the additional-entry discount to any dog entered in two or more classes from the same `is_discount_eligible` group at the same trial, regardless of whether the group produces a per-trial ribbon.

**Run-off**
When two or more dogs tie for HIT, HC, or a class placement, they may be required to repeat an exercise to determine the winner.

**Placement**
The finishing order within a class for qualifying dogs: 1st, 2nd, 3rd, 4th. Most clubs award ribbons for the first four placements. Placement is typically only meaningful for qualifying dogs.

## Entries, exhibitors, and handling

**Entry**
A single dog's registration in a single class at a single trial. A dog entered in Novice B and Open A on Saturday AM has two entries.

**Exhibitor**
The person entering the dog. Not always the legal owner.

**Owner**
The person or people on the dog's AKC registration. May not be the exhibitor.

**Co-owner**
A person listed as an owner of a dog alongside one or more other owners. Recorded on the dog record (not the entry) via `dog_ownerships`. Only one owner is designated as primary; others are co-owners. Co-ownership is common in real AKC trial data.

**Handler**
The person actually taking the dog into the ring on trial day. In AKC performance events (Obedience, Rally), handler and owner(s) are the same person for approximately 99% of entries. The common exception is a junior handler (typically a family member of the owner). Professional handlers do NOT exist in AKC performance events (they exist in conformation, which is out of QTrial's scope). QTrial does not expose a professional-handler flag.

**Junior Handler**
A minor (typically a family member of the owner) handling a dog in competition. Marked with a special checkbox and an AKC-issued junior handler number on the entry; that number must be submitted to AKC with the trial results. Junior handler paperwork is sent by AKC directly to the kennel club; QTrial does not generate it. Rare in performance events.

**Senior Handler**
In some contexts, a handler over a specified age; some clubs offer Senior Handler classes.

**Catalog order**
The order in which dogs are printed in the event catalog. Typically alphabetical by owner or by breed/class/armband.

**Armband**
The identification number a dog wears during its class run, written on a fabric band worn on the handler's left arm. Assigned at entry processing; remains with the dog for the trial. Numbered in series by class grouping (100s = Novice A, 200s = Novice B, 300s = Intermediate, 400s = Advanced A, 500s = Advanced B + Excellent B + Master shared series, 700s = Excellent A, 800s = Rally Choice - conventional AKC Rally scheme). Armband numbering schemes are configurable per club but default to this convention.

**Armband series**
A block of armband numbers shared across classes that are judged together. When a dog runs multiple classes in the same series, it uses one armband for all of them; if it runs classes in a different series, it gets a separate armband. Modeled as `armband_assignments` keyed by `(dog, trial, armband_series)` in QTrial. See REQ-ENTRY-012.

**Jump height**
The height of the jump the dog must clear. In Obedience, heights are typically 4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36 inches. In Rally, heights are 4, 8, 12, or 16 inches. Rally Choice does not use a jump.

Jump height in QTrial is a per-(dog, trial) election (the `dog_trial_jump_heights` table), not a per-entry-line attribute. The same height applies to every class the dog runs at that trial. Chosen by the handler at entry time based on the dog's height and condition; may be lower than the breed default for older or injured dogs. A judge who doubts the submitted height can measure the dog in-ring and override the recorded value for the rest of the trial (see Judge-measurement override). The dog's AKC jump-height card, when present, provides a default at entry time but does not constrain what the handler may elect.

**Judge-measurement override**
Rare in-ring action where a judge measures a dog whose submitted jump height seems inaccurate and updates the recorded height. The updated height applies to all of that dog's remaining classes at the current trial. Happens approximately once per trial-secretary career (Deborah's reported frequency). QTrial models this on `dog_trial_jump_heights` via `was_judge_measured`, `judge_measured_at`, and `judge_measured_by_contact_id`.

## Trial-day operations

**Judge's book**
The paper (or electronic) record the judge uses to track scores during the trial. Contains one page per class with pre-printed dog entries (armband, call name, handler, placement placeholders).

**Scribe sheet**
The sheet used by a scorekeeper (scribe) to record exercise-by-exercise scoring during Obedience. Separate from the judge's book.

**Catalog**
The master printed document for the trial listing every entry with full information: armband, breed, registered name with titles, sire, dam, owner, handler. Distributed to exhibitors and judges, archived for AKC records.

**Premium list**
The pre-trial document advertising the trial: dates, location, judges, classes offered, entry fees, closing date, mailing address. Mailed and posted before entries open.

**Judging schedule / Running order**
The sequence in which classes will be judged and the order in which dogs will run within each class.

**Running order within a class**
The sequence of dogs within one class. Can be sorted short-to-tall by jump height (common in Obedience), tall-to-short, randomized, or manually ordered by the secretary.

**Ring**
The physical space where a class is judged. A trial may use multiple rings concurrently (especially Rally).

**Ring steward / gate steward**
Volunteers who manage the flow of dogs at the ring gate - calling dogs, managing the queue, assisting the judge.

## Entry lifecycle events

**Opening date**
The date and time entries begin being accepted.

**Closing date**
The date and time entries stop being accepted (except for day-of move-ups and transfers, which have their own rules).

**Waitlist**
When a class or trial is full, additional entries go on a waitlist. If an entered dog withdraws before the closing date (or sometimes after, depending on club rules), the next waitlisted dog takes its place.

**Move-up**
An exhibitor's written request to move an entered dog up to the next class because the dog earned the title for its current class after the trial's closing date but before the trial begins. For example: a dog earned CD yesterday, and the exhibitor requests move-up from Novice B to Open A for this weekend. Move-ups follow strict AKC rules about timing and documentation.

**Transfer**
A change of a dog's class entry, typically within the same sport at the same level (e.g., Novice A to Novice B). Allowed under specific conditions per AKC Obedience/Rally regulations. Written request required, must be received before a specified deadline (commonly 30 minutes before the start of the trial in question).

**Bitches in season**
AKC allows a full refund (less a small processing fee, typically $5 per class) for a bitch in season, provided written notice is given before the start of the trial. This is a special-case refund category.

**Scratch**
An exhibitor's decision not to run a dog that was entered. A scratched dog may still appear in the catalog but does not compete. Refund policies vary.

**Withdraw**
Similar to scratch but typically indicates the exhibitor removed the entry before the trial began, often during waitlist processing or before closing.

**Absent**
Dog was entered but did not appear at ringside when called.

**Excused**
Dog appeared but the judge excused it from the ring without a qualifying performance (for safety, handler behavior, dog behavior, etc.).

**DQ (Disqualified)**
A more severe version of excusal. The judge disqualifies the dog, typically for aggression or a specific rules violation.

## Teams, braces, and pairs

**Brace**
Two dogs entered and handled together by one handler. Common in Obedience Brace class.

**Team (Obedience Team)**
Four dogs entered as a team, typically judged with modified exercises. Team competition has its own scoring system.

**Rally Pairs**
Two dogs and two handlers completing a Rally course together.

**Rally Team**
Multiple dogs and handlers completing a Rally course together.

**Rally T Challenge / Rally T Challenge Team**
Specialty Rally variants with team and individual versions.

## People and administration

**Trial Secretary**
The volunteer (or paid individual) responsible for the trial's on-the-day administrative side: paperwork, scores, entries, money handling, AKC reporting. Per Deborah's 2026-04-23 Q5, the secretary's responsibilities are operational and run from check-in through awards and post-trial submission. Modeled in QTrial as `events.event_secretary_user_id` (an FK to the user who holds the role for the event); the role is event-level, shared across the event's trials. QTrial's primary user.

**Trial Chair / Trial Chairperson**
The club's designated lead for pre-trial arrangements: acquires judges, gets AKC approval, arranges judge accommodations, recruits stewards / hospitality crew / timekeeper / course builder, and handles judge and secretary expense payments. Distinct from the Trial Secretary; usually a different person. Per Deborah's 2026-04-23 Q5, the chair "can be off-site during the trial in a pinch" but the expectation is they are present. Modeled as `events.trial_chair_user_id`; role is event-level. The free-text `trials.trial_chairperson` column from the Phase 0 scaffold was dropped in PR 2d in favor of this typed FK.

**Club**
The AKC-affiliated or AKC-licensed dog club hosting the event. Clubs have member status, license status, or an AKC club number (4-digit).

**Event Chair**
Sometimes synonymous with Trial Chair.

**Superintendent**
A paid professional who runs large events for clubs. Large conformation shows use superintendents. Smaller trials use volunteer trial secretaries. QTrial's primary market is the trial secretary segment, not the superintendent segment.

## AKC-specific identifiers and codes

**AKC event number**
A unique number assigned by AKC to each approved event (technically each trial-day combination). Required for results submission.

**AKC club number**
A 4-digit identifier for the hosting club.

**AKC registration number / AKC #**
The dog's individual AKC identifier. Formatted as two letters followed by up to 8 digits (e.g., `DN12345678`, `SS22371305`, `SR95697401`). The letter prefix relates to the breed group - see `tblAKCGroups` in Deborah's current schema for the prefix-to-group mapping. Registration numbers in QTrial are stored as TEXT (not INT); leading zeros matter and the prefix may vary.

**PAL (Purebred Alternative Listing)**
AKC registration category for spayed or neutered purebred dogs whose owners lack registration papers. PAL-registered dogs can compete in AKC performance events (Obedience, Rally) but not in conformation. Formerly called ILP (Indefinite Listing Privilege). Registration-number prefix is "PAL" (e.g., `PAL282370`).

**Canine Partners**
AKC's registration program for mixed-breed dogs, allowing them to compete in companion events. Listed as breed "All American Dog" in catalogs. Registration-number prefix "MA" or "MB" (e.g., `MB11524001`, `MA92798101`).

**All-American Dog (AAD)**
The breed name AKC's canonical breed catalog uses for mixed-breed dogs registered through the Canine Partners program. Treated as a regular breed for entry, scoring, and submission purposes in AKC Obedience and Rally trials. The `events.mixed_breeds_allowed BOOL` (default TRUE) controls whether All-American Dogs may enter a given event. Most Obedience and Rally trials accept AAD; conformation events and certain Specialty events typically do not. Per Deborah's 2026-04-23 Q3, the BOOL flag is the right shape for the AAD-exclusion case because the alternative (enumerating all 288 recognized breeds in an exclude-list to opt AAD out) is unwieldy and the alternative-alternative (treating AAD as a special case in the renderer) is brittle. The broader breed-list / breed-group / breed-variety allow-list / deny-list model is post-MVP; see `docs/PROJECT_STATUS.md` Known-gaps and `docs/ROADMAP.md` Phase 2+.

**FSS (Foundation Stock Service)**
AKC's registration program for breeds working toward full AKC recognition. FSS breeds compete in the Miscellaneous group.

**Judge number**
AKC's identifier for each approved judge. Required on AKC reports.

**Junior Handler number (JHN)**
Identifier for a registered Junior Handler under 18.

## Money

**Entry fee (first)**
The fee for the first class a given dog is entered in, within a trial. Typically the larger of the two fees (covers AKC recording fee and processing overhead).

**Entry fee (additional)**
The fee for each subsequent class the same dog is entered in, within the same trial. Typically lower than the first-entry fee.

**AKC recording fee**
The per-entry fee AKC charges to the club for each entry (typically $3.50 first entry, $3 additional - verify current figures). Included in the advertised entry fee.

**AKC event service fee**
A per-event fee the club pays to AKC.

**HIT, HC, and placement awards**
Ribbons, trophies, or small cash awards for placement. QTrial tracks whether they are awarded, not the physical distribution.

**Coupon / discount**
Club-issued credits against entry fees, often for members or to rectify prior refund issues.

## File and document formats QTrial must produce

This is a quick index. Details live in `REQUIREMENTS.md` and `WORKFLOWS.md`.

- **Premium list** - PDF, mailed/posted pre-trial
- **Entry confirmation** - PDF, one per dog per trial, sent after entry is processed (REQ-ENTRY-010). Shows registered name with titles, registration number, DOB, sex, breed, breeder, sire, dam, owner, and per-class per-day entry status with armband numbers and jump heights.
- **Post-closing confirmation email** - Email sent by the trial secretary approximately 1 week before the trial, consolidated per owner, listing all the owner's dogs' entries along with the running schedule (REQ-ENTRY-014).
- **Confirmation / waitlist / cancellation / refund email** - template-driven per club (REQ-EMAIL-001)
- **Catalog** - PDF, printed for distribution on trial day
- **Marked catalog** - PDF, the trial catalog with final scores annotated on each entry. One of the three required AKC submission artifacts for Obedience/Rally (REQ-SUB-001). Reference: `Nov_2025_Sat_Marked_Catalog.pdf`.
- **Judges book** - Per-class scoring book with armband, dog info, score, and Time Started / Time Finished fields. Carbonless four-part form (White: AKC, Yellow: Club, Pink: Judge, Gold: Post). One of the three required AKC submission artifacts (REQ-SUB-002). Reference: `gfkc_rally_judges_book_cover_2025_11_15_sat.pdf`.
- **Steward board** - Large-print ringside posting showing class armband order and scores for spectator and handler reference during the trial.
- **Scribe sheet** - PDF, for Obedience exercise-by-exercise scoring
- **Running order / judging schedule** - per class per trial, updated as move-ups and scratches occur
- **Armband assignment sheet** - internal document for secretary
- **Armband cards** - printed, distributed at check-in
- **Form JOVRY8** - AKC's official Report of Rally Trial form (current version: 03/23 v1.0 Edit). One of the three required AKC submission artifacts (REQ-SUB-003). An Obedience equivalent form exists. Reference: `Trial_Summary_report.pdf`.
- **Financial report** - per event, for club accounting

## AKC submission (Obedience and Rally, MVP)

**rallyresults@akc.org**
AKC's electronic submission destination for Rally trial results packages. Obedience has a parallel address. Submission package is three PDFs (marked catalog + judges books + Form JOVRY8) plus payment. Alternative: mail to AKC Event Operations, PO Box 900051, Raleigh NC 27675-9051. See WORKFLOWS.md §9.

**AKC recording fee**
The per-entry fee AKC charges to the club. Current schedule: $3.50 for the first entry per dog per trial, $3.00 for each additional entry per dog per trial, plus a $10 event-secretary fee after 12 trials in a calendar year. Always verify current rates via AKC Rally Regulations Chapter 1 Section 4 before using (see CLAUDE.md). QTrial calculates this per REQ-SUB-005 by grouping entries by `(dog_id, trial_id)`.

## Entry and name parsing

**Entry confirmation**
PDF document sent by the trial secretary to each exhibitor after their entry is processed, showing what's been entered for each of their dogs at this trial, with armband numbers and jump heights. One PDF per dog.

**Post-closing confirmation email**
Email sent by the trial secretary approximately 1 week before the trial, consolidated per-owner, listing all the owner's dogs' entries along with the running schedule.

**Unparsed title token**
A string appearing in a registered dog name that looks like a title but is not recognized in QTrial's title catalog (49 prefix + 244 AKC suffix + 5 legacy compound + 10 Barn Hunt). Preserved verbatim in the dog record's `unparsed_title_tokens` array for trial-secretary review; rendered verbatim in catalog output. QTrial does NOT auto-create catalog entries from unparsed tokens; the trial secretary decides whether they are typos (to be corrected) or non-AKC titles (to be ignored). Real-data examples of unparsed tokens: `UCGC` (likely a typo for `CGCU`), `WCCC?` (has a literal question mark in the source), `CGUWCX` (likely a concatenation error).

**Email template**
A per-kennel-club, per-purpose message body with `{{variable_name}}` substitution. Each club configures their own `entry_confirmation`, `post_closing_reminder`, `cancellation_notice`, and `refund_confirmation` templates. Stored in the `email_templates` table; default templates are seeded when a club is created.

## Things that are easy to get wrong

- **"Trial" vs "event"**: these are not interchangeable. An event can contain multiple trials.
- **"Class" vs "level"**: Novice is a level; Novice A and Novice B are classes within that level.
- **"Entry" vs "entrant"**: an entry is a specific registration (dog in class). An entrant could be a dog or a person depending on context. Prefer "entry" (unambiguous).
- **"Exhibitor" vs "handler" vs "owner"**: not always the same person.
- **"Q" vs "placement"**: A dog can Q and still not place. A dog can place and not Q (rare, only in specific nonregular classes).
- **"Move-up" vs "transfer"**: a move-up advances a dog to the next level (because it earned the title); a transfer changes class within the same level (A to B).
- **"Preferred" classes in Obedience** are unrelated to Preferred in Agility. Different regulatory categories.
- **"HIT" is not per-class**: it is awarded once per trial across all Regular Obedience classes.

## Pending glossary additions (needed from artifacts)

- Rally-specific course terminology (signs, station numbering, course map conventions)
- Scent Work, FastCAT, Agility, Barn Hunt terminology (once those sports are in scope)
- Regional and National Championship event-type distinctions
- UKC equivalents of all the above (for when we extend beyond AKC)

Deborah's screenshots, PDFs, and a walkthrough of one full trial will help fill in anything subtle that Robare and Claude got wrong here.
