# QTrial - Requirements

**Status:** Draft v0.1 - **provisional**; will be updated once additional artifacts (PDF examples, screen grabs, trial-weekend walkthrough) are received from Deborah.
**Last updated:** 2026-04-19
**Primary sources:** Deborah's `Outline_of_Online_Trial_software.pdf`; schema extract from `ObedienceData.mde`; competitive research.

---

## How to read this document

This document consolidates what QTrial must do, grouped by functional area. Requirements are stated as behaviors the system must support, not implementation. Implementation lives in `ARCHITECTURE.md` and the codebase.

Requirements are tagged:
- **[MVP]** - Required for first real-world use by Deborah.
- **[P2]** - Post-MVP, targeted for second release.
- **[P3]** - Further post-MVP.
- **[PENDING]** - Requirement known to exist but needs confirmation from additional artifacts.
- **[ASSUMPTION]** - Stated here as Claude's best guess; needs Deborah's confirmation.

---

## 1. Club and tenant management [MVP]

### 1.1 Club identity

The system must support multiple independent clubs, with each club's data strictly isolated from every other club's data.

Each club has:
- A display name (e.g., "Glens Falls Kennel Club")
- A short abbreviation (e.g., "GFKC") for internal and print use
- An AKC club number (4 characters)
- Member/Licensed status (the two categories of AKC-affiliated club)
- An optional logo file
- A UKC club number (reserved for future UKC support, nullable in MVP)
- An organizing address, contact information, and officer list

### 1.2 Users and roles

Each user of the system has a user account independent of any club. Users are then granted roles within specific clubs.

**Roles [MVP]:**
- **Platform Administrator** - QTrial staff. Can see all tenants. Used for support and diagnostics only, not day-to-day operations.
- **Club Administrator** - Can manage the club's profile, invite other users, configure default settings, view financial reports. There can be multiple club administrators per club.
- **Trial Secretary** - Can create and manage events, process entries, generate documents, enter scores, submit results. A secretary can be attached to multiple clubs (a common real-world pattern).
- **Judge** - Can view their own assignments and judge's books. Cannot see other judges' data or the full event dashboard.
- **Exhibitor** - Can enter dogs, pay for entries, view results. Has a self-service account and is not explicitly invited by any club (except waitlist management or special cases).

**Role assignment rules [MVP]:**
- A user can hold different roles at different clubs simultaneously (secretary at Club A, exhibitor at Club B, judge at Club C).
- A Club Administrator can invite and remove users at their club.
- The first user at a new club is the Club Administrator by default.

**Sign-up paths [MVP]:**
- Club creation is self-service via a sign-up flow (with some limits to prevent abuse).
- Exhibitors self-register to enter trials.
- Judges are typically invited by clubs but can also self-register and claim their AKC judge number (with club approval for specific assignments).

### 1.3 Club configuration [MVP]

Each club has default settings that apply across its events unless overridden at the event level:
- Default confirmation letter template
- Default waitlist letter template
- Default judge letter template
- Default premium list content blocks
- Default catalog formatting preferences
- Default payment policies (which payment methods the club accepts, whether card fees are passed to exhibitors, refund policies)
- Default armband numbering scheme
- Default running-order sort (short-to-tall, tall-to-short, random, manual)

## 2. Event setup [MVP]

Matches and expands on Deborah's outline section 1.

### 2.1 Event creation

A secretary can create a new event with:
- Event type (Obedience, Rally, or both; Agility, Scent Work etc. deferred per sport-by-sport roadmap)
- AKC event number(s), one per sport per day per trial (see 2.3 below - the system must accommodate the event-day-trial hierarchy)
- Human-readable event name
- Host club (must be a club the user is authorized for)
- Venue / location (free text with structured address fields)
- Event dates (start and end; system derives the list of days)
- Entry opening date and time
- Entry closing date and time
- Move-up submission deadline (date and time)
- Trial entry limit per day per trial (may be set globally or per-trial)
- Breeds allowed (all breeds, specific group, specialty breed, mixed-breed inclusion) [ASSUMPTION: most events are all-breed; breed restrictions exist per the schema's BreedRestriction, GroupRestriction, BreedExclusion, GroupExclusion fields]

### 2.2 Entry fees

Per-trial fee structure:
- First-entry fee (per dog)
- Additional-entry fee (per dog, each subsequent class in the same trial)
- Nonregular class fee (often different from regular classes)
- Nonregular second-class fee
- Brace fee
- Team fee
- Rally Pairs fee, Rally Team fee
- Junior Handler fees (reduced, for each of the above) - per 2013+ AKC regulations
- Catalog fee (for purchasers who buy extra printed catalogs)

All fees are tracked in the event and visible on confirmation emails and invoices.

### 2.3 Event-day-trial hierarchy

An event has one or more days. Each day can have one or more trials (typically one or two per sport per day). Each trial has:
- Its own AKC event number
- Its own start time
- Its own entry limits
- Its own trial chair
- Its own HIT / HC / PHIT / PHC / RHC awards
- Its own judge-to-class assignments
- Independent fee structure (though usually identical across trials within an event)

This hierarchy is a first-class concept in the data model. The system must not conflate days with trials.

### 2.4 Judges and class assignment

Judges are managed as a cross-event directory:
- Full name, AKC judge number, contact info, provisional status
- History of assignments across events in the system

For each event, the secretary:
- Creates judge assignments by matching a judge to one or more classes
- One judge can be assigned to multiple classes
- Two judges can share a class (co-judging)
- Assignments can be changed before the trial (with appropriate notifications)

### 2.5 Classes offered

For each (day × trial) combination, the secretary selects which classes are offered. The system presents:
- Regular Obedience classes (Novice A, Novice B, Open A, Open B, Utility A, Utility B)
- Optional Titling Obedience (Beginner Novice A/B, Graduate Novice, Graduate Open, Versatility, Pre-Novice, Pre-Open, Pre-Utility)
- Preferred Obedience (Preferred Novice, Preferred Open, Preferred Utility)
- Nonregular Obedience (Wildcard Novice/Open/Utility, Brace, Veterans, Team, Sub-Novice, International)
- Regular Rally (Rally Novice A/B, Rally Intermediate, Rally Advanced A/B, Rally Excellent A/B, Rally Master)
- Nonregular Rally (Rally Pairs Novice/Advanced/Excellent, Rally T Challenge, Rally T Challenge Team, Rally Team Novice/Advanced/Excellent, Rally Plus, Rally Intro)

The canonical class definitions (including whether the class has jumps, whether it allows multiple entries per dog, default dogs-per-hour, and related metadata) are seeded into QTrial's reference data, derived from the `tblkAKCObedClassInfo` data in Deborah's current system.

### 2.6 Jump heights

For classes that involve jumps, jump heights must be selectable per dog per class. Jump heights available for Obedience: 4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36 inches. For Rally: 4, 8, 12, 16 inches. The running order can be sorted short-to-tall or tall-to-short; this is configurable per event (or even per class).

## 3. Entry submission [MVP]

Matches and expands on Deborah's outline section 2.

### 3.1 Exhibitor account

An exhibitor creates an account with:
- Name, address, phone, email
- Optional mailing list opt-ins (per club, per sport)
- Optional junior-handler flag with JH number if applicable
- Saved payment methods (stored via Stripe, never in our database)

### 3.2 Dog records

An exhibitor can add dogs to their account:
- Call name (the nickname used day-to-day)
- Breed (with variety and division where applicable - Poodles have varieties; some breeds have divisions)
- Sex
- AKC registered name
- AKC registration number (or PAL, Canine Partners, or FSS equivalent)
- Country of registration (default US, but Canadian, UK, etc. supported)
- Prefix titles (editable list, selected from canonical AKC prefix title set)
- Suffix titles (editable list, selected from canonical AKC suffix title set)
- Breeder name
- Sire: prefix titles, registered name, suffix titles
- Dam: prefix titles, registered name, suffix titles
- Co-owners (free text for catalog rendering)
- Owner (the exhibitor's account, or a named co-owner)
- Birthdate
- AKC jump height measurement (for dogs with a height card)
- Sport participation flags (Obedience, Rally, Agility, etc. - used for mailing preferences)

Dogs are shared across events - an exhibitor enters the same dog record in multiple trials without re-typing.

### 3.3 Entry flow

For a given event:
1. Exhibitor selects the event from their dashboard or a public event listing.
2. Exhibitor selects which dog to enter.
3. Exhibitor selects one or more classes, specifying:
   - Class (from those offered at the trial)
   - Jump height (if the class has jumps)
   - Day / trial (one entry per class per trial)
4. System calculates fees (first-entry vs. additional, junior-handler rate if applicable, team/brace/pairs surcharges).
5. System validates entry eligibility:
   - The class is offered on that day/trial
   - The dog is eligible for the class (per AKC rules - dog too advanced for "A" class, etc. - basic checks only in MVP)
   - The entry is before the closing date
   - Neither the class nor the trial's limit is exceeded (otherwise the entry goes on waitlist)
6. Exhibitor pays via credit card (Stripe).
7. System generates armband assignment (either at entry-time or deferred until after closing - configurable per club).
8. System sends confirmation email with entry details.
9. If the class or trial is full, entry is placed on waitlist and a waitlist email is sent.

### 3.4 Paper entry processing [MVP]

Some clubs accept paper entries mailed in. The secretary must be able to:
- Manually enter an exhibitor's paper entry (bypassing the online payment flow)
- Record payment by check, money order, cash, or coupon
- Trigger the same confirmation email flow (or print a mail-in confirmation instead)

### 3.5 Entry state and lifecycle [MVP]

An entry has a state:
- `Pending` - submitted but awaiting payment or validation
- `Active` - accepted into the trial
- `Waitlist` - awaiting an opening
- `Scratched` - removed at exhibitor request before closing
- `Withdrawn` - removed after closing
- `Transferred` - moved to a different class (A↔B or class-to-class within the same level, via written request)
- `MovedUp` - moved to the next level (because the dog earned a title after closing)
- `Absent` - dog did not appear
- `Excused` - judge excused the dog
- `DQ` - judge disqualified the dog

Each state transition must be timestamped and (for transfers, move-ups, withdrawals after closing) accompanied by the required documentation per AKC rules. **[PENDING confirmation of specific documentation requirements from Deborah's workflow.]**

### 3.6 Special entry types [MVP]

- **Brace** - two dogs from the same or different exhibitors paired for Brace Obedience. Brace number assigned.
- **Team** - four dogs for Team Obedience. Team name assigned.
- **Rally Pairs** - two dogs, two handlers, paired for Rally Pairs classes.
- **Rally Team** - variable-sized Rally team.
- **Rally T Challenge Team** - Rally-specific team format.
- **Veteran** - entries in Veteran-class variants (marked on the entry).
- **Junior Handler** - entry handled by a registered junior; affects fee calculation.

## 4. Waitlist management [MVP]

A waitlisted entry:
- Is associated with the class and trial it attempted to enter.
- Is ordered by submission timestamp (first-come-first-served per AKC convention).
- Receives the waitlist email immediately upon being placed on the waitlist.

When a space opens (another entry withdraws or scratches before closing):
- The next waitlisted entry for that class is automatically promoted to `Active`.
- Payment is processed (if not already collected).
- A new confirmation email is sent.

If the entry is still on the waitlist when closing occurs:
- The entry is either refunded (if payment was collected conditionally) or released with no charge.
- An "unable to enter" notification is sent.

The secretary can manually adjust waitlist order in exceptional cases (with an audit entry).

## 5. Move-ups, transfers, and late changes [MVP]

### 5.1 Move-ups

An exhibitor can submit a written move-up request for a dog that earned a title after the closing date but before the trial begins. The system:
- Accepts the request up to the stated move-up deadline
- Validates that the target class is offered at the trial and has capacity
- Updates the entry's class assignment
- Updates catalog, judge's books, running order, and any downstream documents
- Notifies the exhibitor and the relevant judge (if applicable)

### 5.2 Transfers (A↔B, special cases)

Per AKC regulations, certain transfers are allowed with specific timing and documentation:
- Entry of ineligible dog → transfer to eligible class
- A-class to B-class transfer at the same level (if handler qualifies)
- Obedience class to Obedience class or Rally class to Rally class within the same sport

System enforces:
- Deadline: written request received by specified time (typically 30 min before trial starts, configurable)
- Target class must be offered and have capacity
- The judge and class availability check

### 5.3 Bitches in season

When an exhibitor reports a bitch in season (written or text notification per AKC), the system must:
- Allow the secretary to mark the entry as "withdrawn - bitch in season"
- Calculate refund per club policy (typically full entry fee less a $5 processing fee per class)
- Generate a refund through Stripe
- Update all downstream documents to remove the dog

### 5.4 Day-of operations

At the trial, the secretary may need to handle:
- Late scratches (dog arrives, doesn't run) → mark `Absent`
- Excusals (judge excuses dog) → enter `Excused` with reason
- DQs (judge disqualifies) → enter `DQ` with reason
- Run-offs (ties for placement or HIT/HC) → record outcome

## 6. Armband generation and numbering [MVP]

Matches Deborah's outline section 3.

### 6.1 Numbering schemes

The secretary selects the numbering scheme at event setup:
- Sequential per-trial (armband 1 in each trial)
- Sequential across the full event (armband 1 starts once)
- By class (each class gets its own range)
- By day (each day resets)
- With configurable intervals and starting numbers

### 6.2 Output

The system produces:
- An armband assignment sheet for the secretary (spreadsheet-like view)
- Printable armband cards (typically 4 or 6 per letter-size page) with dog call name, owner, breed, and class
- Re-generation as entries change (move-ups, scratches, waitlist promotions)

## 7. Running order and schedule generation [MVP]

Matches Deborah's outline sections 4 and 5.

### 7.1 Class running order

For each class at each trial, the system produces an ordered list of dogs:
- Sorted by jump height (short-to-tall or tall-to-short, configurable)
- Within the same height, by armband or random
- Reflecting all current move-ups, scratches, and transfers

### 7.2 Judging schedule

Based on estimated dogs-per-hour per class (seeded from canonical reference data), the system produces a per-ring, per-day schedule:
- Ring assignments
- Start times per class
- Lunch breaks and other gaps (configurable)
- Updated automatically as entry counts change

### 7.3 Trial-time calculation

System provides an estimate of total trial time based on:
- Entries per class
- Class-specific run time (e.g., Standard Agility 1.5 min/dog, Rally 20 dogs/hr)
- Class change time (30 sec default)
- Event change time (45 sec default)

These values are taken from Deborah's `tblTrialTimeCalculation` data and are configurable per event.

## 8. Scoring [MVP]

Matches Deborah's outline section 6.

### 8.1 Score entry

For each entry, the secretary (or in Phase 2, the judge) records:
- Score (numeric, per class's scoring scale - Obedience 200, Rally 100, etc.)
- Time (for timed classes like Rally Master, in MM:SS format or seconds)
- Q or NQ
- Abs (Absent), Exc (Excused), DQ (Disqualified)
- Placement (1st, 2nd, 3rd, 4th if qualified; nullable otherwise)
- OTCH points (computed per class per entry - auto-calculated from rank and number of entries)
- OM points (for Obedience Master - auto-calculated)
- Reason for excusal (free text)
- Withdrawn flag (post-trial)
- Alternate flag (for Rally team alternates)

### 8.2 Computed awards

For each trial, the system computes:
- High in Trial (HIT) - highest Regular Obedience score
- High Combined (HC) - highest combined Open B + Utility B scores for dogs that qualified in both
- PHIT, PHC, RHIT, RHC, HTQ - analogous awards for Preferred, Rally

Ties are flagged for the secretary to resolve via run-off.

### 8.3 Score validation

- Score must be within the valid range for the class
- A dog marked Absent, Excused, or DQ cannot also have a score
- Total score cannot exceed class maximum
- Placements can only be assigned to qualifying dogs (except in specific nonregular classes)

## 9. Judges' books [MVP]

Matches Deborah's outline section 7.

For each judge, for each class, generate a printable judge's book containing:
- Class name, level, day, trial
- Judge name
- Entry list with armband, call name, breed, handler, jump height
- Blank fields for scoring (or pre-populated if scores are already entered)
- Signature line for the judge
- Page numbering and catalog cross-reference

Format: PDF, one file per judge with all their classes, or one file per class. Secretary chooses.

## 10. Catalog generation [MVP]

Matches Deborah's outline section 8.

### 10.1 Content

The catalog contains:
- Event cover page (club logo, event name, dates, location, judges)
- Premium list reprint (classes offered, fees, policies)
- Judges' list with photos (optional) and AKC numbers
- For each class:
  - Class header (name, level, judge, ring, start time)
  - Numbered entries (armband, full dog info: prefix titles + registered name + suffix titles, breed + variety, sire info, dam info, owner(s), handler)
  - Move-up entries (optionally shown separately at end)
  - Absent/withdrawn entries (optionally marked)
- After the trial: scored catalog with results (placements, Q/NQ, scores, HIT/HC designations)

### 10.2 Output

- PDF generation
- Re-generation as entries change
- Support for the pre-trial catalog and the post-trial "marked" catalog
- AKC compliance - catalog format must meet AKC's requirements for the official record

## 11. Premium list generation [MVP]

Though not explicitly in Deborah's outline, the premium list is a required pre-trial deliverable. The system must generate a PDF premium list containing:
- Event name, dates, location
- Entry opening and closing dates
- Classes offered, jump heights, judges
- Entry fees (structured per class and per-addition)
- Payment instructions
- Refund policy
- Entry submission method (online, mail)
- Club officers and trial chair
- AKC required boilerplate (the Agreement and Rules)
- Mailing address for paper entries
- Online entry URL

Seeded from event setup data; the secretary can add club-specific content via a template editor.

## 12. Confirmation emails [MVP]

Matches Deborah's outline section 2d.

For each entry, an email is generated from a template with variable substitution:
- Exhibitor name and address
- Dog call name and registered name
- List of classes entered with jump heights and fees
- Total fees owed and paid
- Entry status (active or waitlist)
- Trial dates, location, and relevant venue info
- Club contact information
- Signature block from the secretary

Templates are editable by the club administrator and per-event by the secretary.

## 13. Financial tracking [MVP]

For each event:
- Total entries, dogs, runs
- Total fees collected (by payment method)
- AKC fees owed to AKC
- Coupons and discounts applied
- Refunds issued
- Net revenue to the club

For each exhibitor:
- Amount owed (calculated from entries)
- Amount paid (sum of payments)
- Balance (owed minus paid) - non-zero balances flagged for follow-up

For each payment:
- Method (check, money order, credit card, cash, PayPal, coupon, discount)
- Payment reference (check number, Stripe transaction ID, etc.)
- Amount
- Date
- Whether deposited (for checks)

Reports:
- Per-event financial summary (for the club)
- Per-day breakdown
- Payment-method breakdown (for bank reconciliation)
- AKC fees calculation (for filing)

## 14. AKC results submission [MVP]

This moved from Phase 2 to MVP based on the discovery of the AKC XML structure in Deborah's current schema.

### 14.1 Electronic submission

The system produces an XML document conforming to the AKC schema (the 2004 schema `xmlschema_12032004.xsd` is the known starting point; QTrial must validate against the current AKC schema as of the MVP release date - **[PENDING: obtain current schema from AKC or from Deborah's workstation].**

The XML document contains:
- Sender information (club, club number, secretary identity, schema version)
- Event information (AKC event number, event date, club name)
- For each class:
  - Class identifier (primaryClass code like `AGNOVA`, `AGOPEN`, etc. - Obedience and Rally codes to be confirmed)
  - Number of entries, number of starters
  - Secondary class type (e.g., jump height code)
- For each result:
  - Dog name, AKC reg number
  - Owner name and address
  - Result code (Q, NQ, Abs, Exc, DQ, Withdrawn)
  - Score, time
  - Placement
  - Amateur handler indicator, second-entry indicator
  - Disqualification reason code if applicable
  - Junior handler info if applicable

### 14.2 CSV fallback

If XML submission encounters schema issues, the system also exports a CSV of results suitable for AKC's manual upload process or for the secretary to email to AKC.

### 14.3 Report of Trial

Regardless of electronic submission, the system generates the club-facing Report of Trial document (AKC Form JOVOB7 or equivalent) showing:
- Complaints or issues during the event
- Judges who did not officiate and reasons
- Summary of any regulatory violations or incidents

## 15. Mailing list management [MVP]

Per `tblPremiumMailingList` in the current schema and typical club workflow.

- Club administrators can maintain a mailing list of past exhibitors and interested parties
- Mailing list is segmented by sport interest (Obedience, Rally, Agility, etc.)
- Club can send mass emails (premium list announcements, trial reminders, results notifications) with template-driven content
- Exhibitors control their own opt-in/opt-out per sport per club
- List is auto-populated from past entries (with opt-out opportunity at entry time)

## 16. Data migration from Access [MVP]

A tool that imports data from a Lab Tested Databases–style Access `.mde`/`.mdb` file into an QTrial club:
- Dogs (`tblDogData`)
- Owners (`tblOwnerData`)
- Judges (`tblJudges`)
- Events (`tblEventData`, `tblEventDayData`, `tblAKCObedienceClassInfo`)
- Historical entries (`tblAKCObedienceEntryInfo`, `tblAKCObedienceEntries`) - as read-only historical records
- Secretary info (`tblSecretaryInfo`)
- Club info (`tblClubInfo`)

Deduplication logic (especially for dogs and owners that span years) is critical. The migration tool produces a pre-migration report for the secretary to review before committing.

## 17. Exhibitor-facing results and history [P2]

Post-MVP: exhibitors can view:
- Their dogs' full trial history across all events entered via QTrial
- Accumulated Q's toward titles (with explicit "this dog has earned 2 of 3 legs for CDX" indicators)
- OTCH and OM points accumulated
- Event photos (if the club uploads them)
- Catalog entries for past trials

## 18. Queue management (run order for trial-day) [P3]

Post-MVP: real-time run order management on trial day:
- Exhibitors see their position in line via a phone-responsive page
- Ring stewards check off dogs as they run
- Notifications sent "2 dogs out" and "you're up"
- Optional scoring by judge or scribe on a tablet

## 19. Billing and revenue [MVP for basic, P2 for sophisticated]

QTrial as a business:

**MVP:**
- Each event's entry-processing fees are calculated per the pricing model (free under a threshold, per-class above)
- Stripe Connect for routing payments to the club's bank account
- QTrial's fees are automatically withheld from the exhibitor's payment at checkout
- Simple invoicing for clubs that choose alternative arrangements

**P2:**
- Club dashboards with usage and spend
- Promotional pricing and discount codes
- Enterprise / superintendent tier pricing
- Revenue reporting for QTrial's operators

## 20. Out-of-scope for MVP (stated explicitly)

- Native mobile apps (iOS/Android)
- Video streaming
- Judging app for in-ring scoring
- Pedigree management beyond the sire/dam name fields
- Training log / class management
- Intra-club messaging or forums
- Sweepstakes and Futurities (Conformation-specific)
- Group-specialty shows
- Junior Showmanship class management (beyond junior handler fee tracking)

## Artifacts still needed

Marking explicit dependencies on forthcoming materials from Deborah:

1. **PDF examples of outputs:** premium list, catalog, confirmation email, waitlist email, judge's book, scribe sheet, armband sheet, AKC Report of Trial. Needed to validate exact formatting requirements for sections 9, 10, 11, 12, 14.
2. **Screen grabs of the current software:** especially the main entry screen, scoring screen, and the AKC XML generation screen. Needed to validate sections 3, 8, 14.
3. **The current AKC XML schema file** (or equivalent current format). Critical for section 14.1.
4. **Walkthrough of a trial weekend** (narrative or voice memo). Needed for `WORKFLOWS.md`, but will surface gaps here.
5. **Other clubs' data** (if Deborah has `.mde` files for other clubs she's worked with). Would reveal configuration variety relevant to sections 1.3 and 2.
