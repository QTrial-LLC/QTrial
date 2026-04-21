# QTrial - Requirements

**Status:** Draft v0.2 - integrates Deborah's Q&A (2026-04-19 and 2026-04-20) and confirmation-letter artifact review.
**Last updated:** 2026-04-20
**Primary sources:** Deborah's `Outline_of_Online_Trial_software.pdf`; schema extract from `ObedienceData.mde`; Nov 2025 Glens Falls Rally trial artifacts (judging schedule, marked catalog, judges book cover, steward board, Trial Summary report, confirmation letters); Deborah Q&A answers; competitive research.

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

Jump height is an attribute of a dog at a given trial, not per individual entry line. A dog running multiple jumping classes on the same trial jumps the same height in all of them. Heights available for Obedience: 4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36 inches. For Rally: 4, 8, 12, 16 inches. Rally Choice does not use a jump; dogs entered in Rally Choice have no jump height for that class.

The handler elects the dog's jump height at entry time. A judge who doubts the submitted height may measure the dog in-ring and override it for the rest of the trial. Running order can be sorted short-to-tall or tall-to-short; this is configurable per event (or per class).

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
- Registered name (verbatim as submitted; may contain embedded title tokens that the name parser extracts on save)
- Registration type (one of: `AKC_PUREBRED`, `PAL`, `CANINE_PARTNERS`, `FSS`, `MISC`)
- Registration number (TEXT, preserving verbatim formatting including leading zeros; formats vary by type, e.g., `DN71750607`, `SS22371305`, `SR95697401`, `PAL282370`, `MB11524001`)
- Country of registration (default US, but Canadian, UK, etc. supported)
- Prefix titles (editable list, selected from canonical prefix title set; auto-populated from the name parser)
- Suffix titles (editable list, selected from canonical suffix title set; auto-populated from the name parser)
- Unparsed title tokens (preserved verbatim for trial secretary review; see REQ-NAME-001)
- Breeder name
- Sire: registered name with embedded titles (free text, parsed at display time)
- Dam: registered name with embedded titles (free text, parsed at display time)
- Owners (one or more contacts, with exactly one designated as primary; see REQ-ENTRY-016)
- Birthdate
- Sport participation flags (Obedience, Rally, Agility, etc. - used for mailing preferences)

Dogs are shared across events - an exhibitor enters the same dog record in multiple trials without re-typing. Jump height is recorded per-(dog, trial), not on the dog record itself (see REQ-ENTRY-013).

### 3.3 Entry flow

For a given event:
1. Exhibitor selects the event from their dashboard or a public event listing.
2. Exhibitor selects which dog to enter.
3. Exhibitor selects one or more classes, specifying:
   - Class (from those offered at the trial)
   - Day / trial (one entry per class per trial)
   - Handler (defaults to the dog's primary owner; may differ; junior handler AKC number where applicable)
4. For jumping classes, exhibitor selects a single jump height that applies to the dog across all its entries at each trial (4, 8, 12, 16, or 20 inches for Rally; see §2.6 for Obedience heights). Rally Choice entries do not prompt for jump height.
5. System calculates fees, grouping by `(dog, trial)` to apply first-entry vs. additional-entry rates per AKC convention (see REQ-SUB-005). Junior-handler rates and team/brace/pairs surcharges are added as applicable.
6. System validates entry eligibility:
   - The class is offered on that day/trial
   - The dog is eligible for the class (per AKC rules - dog too advanced for "A" class, etc. - basic checks only in MVP)
   - The entry is before the closing date
   - Neither the class nor the trial's limit is exceeded (otherwise the entry goes on waitlist)
7. Exhibitor pays via credit card (Stripe).
8. System assigns armband numbers by series (see REQ-ENTRY-012): a single dog running multiple classes in the same armband series (e.g., Advanced B + Excellent B + Master in the 500 series) shares one armband; a class in a different series (e.g., Rally Choice in the 800 series) gets a separate armband. Assignment may happen at entry-time or be deferred until after closing, configurable per club.
9. System generates a per-dog entry confirmation PDF (see REQ-ENTRY-010) and sends a confirmation email with entry details using the club's `entry_confirmation` email template.
10. If the class or trial is full, entry is placed on waitlist and a waitlist email is sent.

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

### 3.7 Armband assignment by series [MVP]

- **REQ-ENTRY-012:** QTrial shall assign armband numbers by series. A single dog running multiple classes in the same series (e.g., Advanced B + Excellent B + Master in the 500 series) shall share one armband across those classes. A dog running a class in a different series (e.g., Rally Choice in the 800 series) shall receive a separate armband.

The conventional AKC Rally series mapping is documented in `DOMAIN_GLOSSARY.md` under "Armband series." The series-to-class mapping is configurable per club but defaults to the AKC convention.

### 3.8 Jump height model [MVP]

- **REQ-ENTRY-013:** QTrial shall store each dog's jump height as a per-(dog, trial) attribute (4, 8, 12, 16, or 20 inches for Rally; see §2.6 for Obedience heights). The same height applies across all of that dog's entries at the trial. Rally Choice does not jump.
- **REQ-ENTRY-015:** QTrial shall support a judge-measurement override flow in the judge-facing app that allows a judge to update a dog's jump height in-ring when they doubt the submitted height is accurate. The updated height shall propagate to all of the dog's remaining entries at the current trial. This override is rare (Deborah: approximately once per trial-secretary career) but must be supported.

### 3.9 Handler and junior handler identity [MVP]

- **REQ-ENTRY-016:** QTrial shall allow an entry's handler to differ from the dog's owner(s), and shall support marking an entry as a junior-handler entry with an associated AKC junior handler number. In performance events, handler and owner(s) are the same person for ~99% of entries; junior handlers are the common exception. Professional handlers do not exist in AKC performance events; QTrial shall not expose a professional-handler flag. Junior handler paperwork is sent by AKC directly to the kennel club; QTrial does not generate it.

### 3.10 Registered name parsing [MVP]

- **REQ-NAME-001:** QTrial shall parse registered dog names to extract recognized prefix and suffix titles against the title catalog (currently 49 prefix titles + 259 suffix titles including the 244 AKC core suffixes, 5 legacy compound suffixes, and 10 Barn Hunt titles). Unrecognized tokens shall be preserved verbatim in an `unparsed_title_tokens` field for trial secretary review. QTrial shall NOT auto-create title catalog entries from unrecognized tokens. Real data contains typos (e.g., `UCGC` for `CGCU`, `WCCC?` with a literal `?`, garbled concatenations like `CGUWCX`); the parser must handle these gracefully without rejecting the name.

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

## 12. Confirmation emails and entry confirmations [MVP]

Matches Deborah's outline section 2d, extended by confirmation-letter artifact review (2026-04-20).

QTrial produces two distinct confirmation artifacts at two points in the entry lifecycle:

### 12.1 Entry confirmation (per-dog PDF, sent when entry is processed)

- **REQ-ENTRY-010:** QTrial shall generate an entry confirmation PDF for each dog upon entry processing, showing: registered name with titles rendered, registration number, DOB, sex, breed, breeder, sire, dam, owner(s), and per-class per-day entry status with armband numbers and jump heights. One PDF per dog.

Reference format: the two `Confirmation_Letter*.pdf` artifacts Deborah provided.

### 12.2 Post-closing confirmation email (per-owner consolidated, ~1 week pre-trial)

- **REQ-ENTRY-014:** QTrial shall send a post-closing confirmation email to each exhibitor approximately 1 week before the trial, including all the owner's dogs' entries and the day-by-day running schedule.
- **REQ-ENTRY-011:** QTrial shall support multi-dog-per-owner batch entry and generate consolidated per-owner confirmation emails listing all their dogs' entries. Subject lines shall include the involved registration numbers for email threading and search.

### 12.3 Email templates

- **REQ-EMAIL-001:** QTrial shall support per-kennel-club email templates for the following template keys: `entry_confirmation`, `post_closing_reminder`, `cancellation_notice`, `refund_confirmation`. Templates shall support simple `{{variable_name}}` substitution. Default templates are seeded at club creation; clubs may override via the settings UI. Template variables available per template key are documented in `WORKFLOWS.md` §10.

Templates are editable by the club administrator and per-event by the secretary.

### 12.4 Email variables provided to every template

Standard variables include: exhibitor name and address, dog call name and registered name, list of classes entered with jump heights and fees, total fees owed and paid, entry status (active or waitlist), trial dates, location, venue info, club contact info, and a signature block from the secretary.

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

For MVP (Obedience and Rally), AKC submission is PDF-based. XML-based electronic submission is used today only for Agility and is deferred until QTrial adds Agility support post-MVP.

### 14.1 PDF submission package

After the trial, the secretary assembles a submission package consisting of three artifacts:

1. **Marked catalog PDF** (REQ-SUB-001) - the trial catalog with judge scores annotated on each entry. Reference format: `Nov_2025_Sat_Marked_Catalog.pdf`.
2. **Judges books** - one per class, with armband + registered name + breed + score + time columns and a cover sheet with judge certification checkboxes. The pink carbon copy goes to AKC per distribution rules (see REQ-SUB-002). Reference: `Judges_Book_Cover_Sat.pdf`.
3. **AKC Report of Rally Trial (Form JOVRY8 (03/23) v1.0 Edit)** or Obedience equivalent (REQ-SUB-003), populated with event number, date, dog counts, fee calculations, and secretary contact info. Reference: `Trial_Summary_report.pdf`.

Submission requirements:

- **REQ-SUB-001:** QTrial shall generate a marked catalog PDF with judge scores annotated on each entry, in the AKC-accepted format.
- **REQ-SUB-002:** QTrial shall generate pre-printed judges books per class for judge use during trials, with armband + registered name + breed + score + time columns, and a cover sheet with judge certification checkboxes.
- **REQ-SUB-003:** QTrial shall populate a PDF version of AKC Report of Rally Trial (Form JOVRY8) with event number, date, dog counts, fee calculations, and secretary contact info. An Obedience equivalent form shall be supported the same way.
- **REQ-SUB-004:** QTrial shall support emailing the submission package to AKC. The destination email is configurable, defaulting to `rallyresults@akc.org` for Rally and the Obedience equivalent for Obedience. The secretary may alternatively mail a physical package to AKC Event Operations (PO Box 900051, Raleigh NC 27675-9051).
- **REQ-SUB-005:** QTrial shall calculate AKC recording fees per the current rate schedule ($3.50 first entry per dog per trial, $3.00 each additional entry per dog per trial, plus a $10 event secretary fee after 12 trials per year). Fee calculations shall group by `(dog_id, trial_id)` and charge first-entry vs. additional-entry rates accordingly.

### 14.2 Deferred: XML-based electronic submission [P2]

XML-based electronic submission to AKC (conforming to AKC's Agility schema or any later successor) is deferred post-MVP. It becomes relevant when QTrial adds Agility support, since Agility submission today is XML-based.

### 14.3 Deferred: AKC email integration [P2]

Automated API or SMTP integration with `rallyresults@akc.org` is deferred. For MVP, QTrial produces the submission package as downloadable PDFs and the secretary attaches them to an email she sends herself. A templated "send to AKC" draft-email workflow may be added at the secretary's request.

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

## Artifacts received and still needed

### Received (2026-04-19 / 2026-04-20)

Reference artifacts now in the project for downstream development:

- `Nov_2025_AKC_Rally_Trial_Judging_schedule.pdf` - reference judging-schedule format
- `Nov_2025_Sat_Marked_Catalog.pdf` - reference marked-catalog format (critical for REQ-SUB-001 PDF generation)
- `Judges_Book_Cover_Sat.pdf` - reference judges-book cover format
- `Stewards_BOard_Sat.pdf` - reference steward-board format
- `Trial_Summary_report.pdf` - reference AKC Form JOVRY8 population (critical for REQ-SUB-003 PDF form-fill)
- `Confirmation_Letter.pdf`, `Confirmation_Letter2.pdf` - reference entry-confirmation PDF format (REQ-ENTRY-010)
- Susan Brownell post-closing email - reference consolidated per-owner confirmation email (REQ-ENTRY-014)

### Still needed

1. **`Judges_Book_Sat.pdf` body pages** (Deborah has the cover; the body pages with filled-in score and time rows were listed in the project file manifest but not uploaded). Needed to validate judges-book body layout for REQ-SUB-002.
2. **Screen grabs of the current software:** especially the main entry screen and the scoring screen. Useful for validating UX in sections 3 and 8; not blocking schema.
3. **Trial-weekend walkthrough** (narrative or voice memo). Needed for `WORKFLOWS.md`; will surface edge cases.
4. **Other clubs' data** (`.mde` files for other clubs Deborah has worked with). Would reveal configuration variety relevant to sections 1.3 and 2.
5. **AKC Agility XML schema** (current version). Not blocking MVP; needed when QTrial adds Agility support and §14.2 becomes live.
