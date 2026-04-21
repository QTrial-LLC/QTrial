# QTrial - Workflows

**Status:** Draft v0.2 - integrates Deborah's Q&A (2026-04-19 / 2026-04-20) and confirmation-letter artifact review.
**Last updated:** 2026-04-20

---

## How to read this document

This document describes user-facing workflows, step by step. It is written as narrative: what a user is trying to accomplish, what they see, and what happens. It complements `REQUIREMENTS.md` (which is structural) and is the primary artifact Deborah (and later other secretaries) should review and annotate.

Each workflow notes the primary actor, triggering context, and the system's expected behavior. Things marked **[PENDING]** need confirmation from Deborah's walkthrough.

---

## 1. Club and account setup

### 1.1 A new club signs up

**Actor:** First user of a club (typically the club's trial secretary or treasurer).

1. User visits `qtrial.app` and clicks "Sign up your club."
2. User provides their own name, email, and password (or signs up via Google/Apple).
3. Keycloak creates the account; QTrial creates a `users` row.
4. User is prompted: "Create a new club or join an existing one?" User selects "Create new."
5. User enters the club's name, abbreviation, AKC club number, license status, and address.
6. QTrial creates the `clubs` row, the `user_club_roles` row (user as `club_admin`), and prompts the user to upload a logo (optional).
7. User lands on the empty club dashboard with a prompt to create their first event.

**Edge cases:**
- AKC club number already registered under a different user → QTrial flags this and asks for resolution. For MVP, Robare or a platform admin resolves manually; P2 automates via a claim-verification workflow.
- User wants to belong to multiple clubs → they complete sign-up for one, then repeat the flow to be invited (by someone else) or create another.

### 1.2 An exhibitor signs up

**Actor:** Dog owner wanting to enter a trial.

1. Exhibitor clicks an online-entry link from a club's premium list (or finds the trial via QTrial's public event directory).
2. Exhibitor is prompted to sign in or create an account.
3. On creation: name, email, password, home address, phone.
4. QTrial creates the `users` row. No club role is created - an exhibitor is a user without any club-role grants.
5. Exhibitor is redirected to the entry flow for the specific trial they were trying to enter.

### 1.3 A club admin invites a trial secretary

**Actor:** Club administrator.

1. From the club dashboard, admin clicks "Team" → "Invite user."
2. Admin enters the invitee's email and selects the role (Trial Secretary).
3. QTrial sends an invitation email.
4. Invitee clicks the link, is prompted to sign up or sign in, and upon completion their `user_club_roles` row is created.
5. The invitee now sees the club in their dashboard.

---

## 2. Setting up an event

**Actor:** Trial secretary.

This is the workflow that most closely maps to Deborah's outline section 1.

### 2.1 Create the event

1. From the club dashboard, secretary clicks "New event."
2. Secretary fills in:
   - Event name (free text)
   - Sport(s) offered (checkboxes: Obedience, Rally - more sports post-MVP)
   - Registry (AKC for now)
   - Venue name and address
   - Start date, end date (derives the list of days)
   - Entry opening date/time
   - Entry closing date/time
   - Move-up deadline
3. Secretary saves. QTrial creates:
   - One `events` row
   - N `event_days` rows (one per day)
   - Empty stubs for `trials` (configured in the next step)

### 2.2 Configure each trial

For each day, the secretary configures the trials for that day.

1. Secretary clicks into a day and clicks "Add trial."
2. Secretary fills in:
   - Trial number (1 for single-trial days, 1 and 2 for AM/PM)
   - Sport for this trial
   - AKC event number
   - Start time
   - Trial chairperson
   - Entry limit (if any)
   - Fee structure (first entry, additional entry, nonregular, brace, team, pairs, JR rates)
3. Secretary saves. QTrial creates a `trials` row.
4. Secretary clicks into the trial and selects which classes are offered:
   - A sport-filtered list of canonical classes is shown with checkboxes
   - Secretary checks the classes to be offered; can override dogs-per-hour, ring number, class limit
5. Secretary saves. QTrial creates `trial_class_offerings` rows.

### 2.3 Assign judges

1. Secretary clicks "Judges" on the event dashboard.
2. Secretary adds judges (select from club's judge directory or add new).
3. Secretary assigns each judge to one or more classes across the event.
4. Secretary saves. QTrial creates `judge_assignments` rows.
5. **[PENDING]** Should QTrial auto-email judges with their assignments and a draft schedule? Probably yes, but only when the secretary explicitly triggers it.

### 2.4 Generate the premium list

1. Once judges, classes, and fees are configured, secretary clicks "Generate premium list."
2. QTrial renders an HTML preview with all event details.
3. Secretary can edit club-specific content (welcome text, refund policy, trophy descriptions) via inline editors.
4. Secretary clicks "Export PDF" → QTrial generates the premium list PDF and makes it downloadable.
5. Secretary can also publish a public landing page for the event at `qtrial.app/e/<slug>` that serves as the online premium list.

### 2.5 Open entries

1. When the entry opening date/time arrives, QTrial automatically transitions the event status to `open`.
2. The online entry page becomes active.
3. Optionally, QTrial sends a mailing-list notification to the club's mailing list.

---

## 3. Submitting an entry (exhibitor side)

**Actor:** Exhibitor.

### 3.1 Find the event

1. Exhibitor arrives at the online entry page (via email link, club website, or QTrial's public event directory).
2. Exhibitor signs in or creates an account.

### 3.2 Select dog

1. Exhibitor's dashboard shows dogs they've added previously; they select one or click "Add new dog."
2. For a new dog: exhibitor enters the dog's data (breed, call name, AKC registered name, reg number, titles, sire, dam, co-owners, jump height).
3. Titles are selected from canonical prefix/suffix lists, not typed free-form, to ensure catalog-quality consistency.

### 3.3 Choose classes

1. Exhibitor sees the classes offered per day per trial.
2. Exhibitor checks the classes to enter. For each class with jumps, exhibitor selects a jump height (defaulting to the dog's registered jump height).
3. For special entry types (brace, team, pairs), exhibitor indicates the partner(s) and provides the other dog's armband or entry ID if already entered; otherwise creates a combined entry.

### 3.4 Review and pay

1. QTrial shows the entry summary with fees broken down:
   - First entry fee
   - Additional class fees (at the additional rate)
   - Special surcharges (brace, team)
   - Junior handler discount if applicable
   - Subtotal
   - QTrial service fee (if the club passes it to exhibitors)
   - Card processing fee (if passed to exhibitors)
   - Total
2. Exhibitor enters (or selects a saved) credit card via Stripe Elements.
3. Exhibitor clicks "Submit entry."

### 3.5 Confirmation

1. QTrial validates:
   - Payment succeeds
   - Class still has space (atomic check against the trial's entry limit)
2. If everything validates:
   - `entries`, `entry_lines`, `payments` rows are created
   - `dog_trial_jump_heights` row is created or updated from the handler's elected height
   - `armband_assignments` rows are created per (dog, trial, series) the dog is entered in; entry lines are linked via `entry_lines.armband_assignment_id`. A dog running Advanced B + Excellent B + Master (all in the 500 series) gets ONE armband; the same dog also running Rally Choice (800 series) gets a SECOND armband.
   - Entry confirmation PDF is generated (REQ-ENTRY-010) and archived
   - `entry_confirmation` email is queued to `qtrial-workers` using the club's template (REQ-EMAIL-001)
   - Worker sends the email and marks `confirmation_email_sent_at`
3. If the class is full, the entry is placed on waitlist:
   - Payment is captured or authorized per club policy **[PENDING]**
   - Waitlist email is sent
4. Exhibitor sees a confirmation page with entry details and an option to add another dog or another entry.

### 3.6 Post-closing confirmation email (~1 week pre-trial)

Approximately one week before the trial, QTrial sends a `post_closing_reminder` email consolidated per owner (REQ-ENTRY-011, REQ-ENTRY-014). Unlike the per-dog per-entry confirmation PDF in §3.5, this email batches all of an owner's dogs and entries for the trial and includes the day-by-day running schedule.

Example (from Susan Brownell artifact): one email listing three dogs × multiple classes × two days, with the subject line including all three registration numbers for email threading.

1. Scheduler detects that a trial is 7 days out and all entries have been processed (the closing date has passed).
2. For each owner with one or more active entries at the trial, QTrial renders the `post_closing_reminder` template with `{{dogs}}`, `{{entries_by_day}}`, `{{schedule}}`, and `{{registration_numbers}}` variables.
3. QTrial sends the email and marks `entries.post_closing_email_sent_at` for every included entry.

---

## 4. Managing paper entries

**Actor:** Trial secretary.

Some clubs still accept paper entries by mail.

1. Secretary receives a paper entry in the mail.
2. Secretary logs into QTrial and clicks "Enter paper entry" on the event.
3. Secretary enters:
   - Exhibitor contact info (or selects an existing exhibitor)
   - Dog info (or selects an existing dog in the directory)
   - Classes being entered
4. Secretary records payment:
   - Method: Check, money order, cash, or coupon
   - Reference number (check #)
   - Date received
5. Secretary clicks "Submit." QTrial creates the same rows as for online entries but with payment already recorded.
6. Secretary can trigger a confirmation email (or indicate the exhibitor prefers mail).
7. **[PENDING]** How does Deborah typically handle a mailed entry form today? Does she write on the paper? Staple the check? Keep the paper? QTrial should mirror whatever physical-artifact workflow she needs.

---

## 5. Entry changes before closing

**Actor:** Exhibitor (typically) or secretary (on exhibitor's behalf).

### 5.1 Cancel an entry

1. Exhibitor opens their entry and clicks "Cancel."
2. QTrial warns about refund policy.
3. Exhibitor confirms. QTrial:
   - Marks the `entry_lines.status = 'scratched'`
   - Initiates refund via Stripe (if before closing and club policy allows full refund)
   - Promotes the next waitlisted entry, if any
   - Sends refund confirmation and promotion emails

### 5.2 Add a class to an existing entry

1. Exhibitor adds an entry in an additional class for the same dog at the same trial.
2. QTrial calculates the additional-entry fee (not first-entry fee).
3. Payment is processed as usual.

### 5.3 Change jump height

1. Exhibitor updates the dog's jump height on an existing entry line.
2. QTrial records the change. No fee change.
3. If the catalog has already been generated, the secretary is notified.

---

## 6. After closing: move-ups, transfers, and day-of changes

**Actor:** Exhibitor (initiates) and secretary (approves and acts).

### 6.1 Move-up request

A dog earned its CD title yesterday and is currently entered in Novice B at a trial starting in 3 days. The exhibitor wants to move up to Open A.

1. Exhibitor clicks "Request move-up" on the entry.
2. Exhibitor provides: current class, target class, date the dog earned the qualifying title, supporting documentation (optional file upload).
3. QTrial validates:
   - The target class is offered at the same trial
   - The target class has capacity
   - The move-up deadline has not passed
4. The request is submitted to the secretary as a notification.
5. Secretary reviews and approves or rejects.
6. On approval:
   - `entry_lines.status = 'moved_up'`
   - A new entry line is created in the target class
   - Running order is updated
   - Secretary is prompted to regenerate catalog, judge's books, and armband sheet
7. Exhibitor and secretary are emailed about the change.

### 6.2 Transfer request (A ↔ B)

Similar workflow to move-up, but validation rules differ (eligibility check vs. title-earned check). **[PENDING]** Detailed eligibility rules per AKC Obedience/Rally regulations.

### 6.3 Bitch-in-season refund

1. Exhibitor notifies the secretary (via QTrial, email, text, or phone).
2. Secretary marks the entry line `withdrawn` with reason `bitch_in_season`.
3. QTrial calculates the refund: full entry fee less the club's processing fee (default $5/class, configurable).
4. Secretary approves the refund. QTrial initiates a Stripe refund (or records the need for a check refund if payment was by check).
5. Exhibitor receives a refund confirmation email.

### 6.4 Day-of changes

When the trial is running, the secretary may need to:

- Mark an entry as `absent` (dog didn't show up)
- Mark an entry as `excused` with reason (judge excused)
- Mark an entry as `dq` with reason (judge disqualified)
- Record run-off results
- Process a scratched dog after the trial has started (usually no refund, but the dog still needs to be marked in the catalog)

QTrial's UI for day-of changes needs to be fast - the secretary is doing this on a laptop at a folding table while exhibitors are checking in.

---

## 7. Pre-trial paperwork generation

**Actor:** Trial secretary.

Once entries are closed (or close to closing), the secretary generates the paperwork package.

### 7.1 Armbands

1. Secretary clicks "Generate armbands."
2. QTrial assigns armband numbers according to the club's scheme (if not already assigned per-entry).
3. QTrial produces:
   - A spreadsheet-view of all armbands for the secretary's reference
   - Printable armband cards (PDF, 4 or 6 per page) with dog name, handler, class, trial
4. Secretary can preview and then print.

### 7.2 Catalog

1. Secretary clicks "Generate catalog."
2. QTrial renders the catalog HTML with all accepted entries, sorted by class, sorted within class by armband (or custom order), with full registered name formatting.
3. Secretary previews in the browser.
4. Secretary clicks "Export PDF."
5. **[PENDING]** Does Deborah want to print herself, send to a print shop, or have QTrial integrate with a print-on-demand service?

### 7.3 Judge's books

1. Secretary clicks "Generate judge's books."
2. QTrial produces one PDF per judge with all their classes.
3. Secretary prints and assembles.

### 7.4 Running order

1. Secretary clicks "Generate running order."
2. QTrial produces a per-class running order with armband numbers, jump heights, and handler names.
3. Exhibitors can (P2) see their own running-order position via their account.

Pacing note: Rally and Obedience default to 3 minutes per dog for schedule estimation (per Deborah's Q5 2026-04-20). Actual pacing is class-dependent - Nov 2025 Glens Falls data showed Rally Choice running ~4.3 min/dog, Rally Master ~3.5, Rally Excellent B ~3.0-3.2. Secretaries can override the platform default per class via `events.dogs_per_hour_override` JSONB. The override shape is `{"rally-choice": 4.3, "rally-master": 3.5}`; absent entries fall back to `sport_time_defaults`.

### 7.5 Scribe sheets

1. Secretary clicks "Generate scribe sheets."
2. QTrial produces Obedience scribe sheets (one per entry in Obedience classes) with pre-printed armband, class, and exercise list.
3. Secretary prints and distributes.

### 7.6 Ring signs and other logistics

**[PENDING]** Does Deborah print ring signs? Steward-check-in sheets? Anything else we haven't covered?

---

## 8. Trial day

**Actor:** Trial secretary and volunteers.

### 8.1 Check-in

1. Exhibitors arrive and check in (manually at the secretary's table in MVP; via a kiosk or phone app in P3).
2. Secretary confirms each exhibitor and hands out armbands.
3. Any day-of move-ups or late changes are processed as in section 6.4.

### 8.2 Running the trial

1. Classes run per the running order.
2. Judges use their printed judge's books to record scores AND times. The judges book cover has `Time Started` and `Time Finished` fields; the judge fills both for every dog.
3. At the end of each class (or throughout the day), judges hand completed books to the secretary.
4. Secretary enters scores into QTrial, one class at a time:
   - For each entry in the class: score, `time_started`, `time_finished`, Q/NQ, placement, and special flags (absent, excused, DQ)
   - QTrial computes `time_on_course` from the two timestamps or accepts a directly-entered interval
5. QTrial validates scores and computes placements and awards.

### 8.2.1 Tie-breaking by time

When two dogs in the same class earn identical scores, placement is determined by time on course (lower time wins). Example from the Nov 2025 marked catalog, Rally Excellent B: armband 512 and armband 524 both scored 100; placement was 1st and 2nd respectively based on time on course.

Placement SQL logic: `ORDER BY score DESC, time_on_course ASC`. This applies to both Obedience and Rally in MVP.

### 8.2.2 Judge-measurement override (rare)

If a judge doubts the submitted jump height for a dog in-ring, they can trigger a measurement override in the judge-facing app (REQ-ENTRY-015):

1. Judge taps "Re-measure dog" in the judge-facing app.
2. Judge enters the measured height (one of 4, 8, 12, 16, 20).
3. QTrial updates the dog's `dog_trial_jump_heights` row: `was_judge_measured=true`, `judge_measured_at=now()`, `judge_measured_by_contact_id`, and the new height.
4. The updated height applies to ALL of the dog's remaining entries at the current trial, not just the class being judged.
5. Running order for subsequent classes is recomputed if it was height-sorted.

Deborah reports this has happened approximately once in her entire trial-secretary career, so it is a cold path UX-wise; correctness matters, polish does not.

### 8.3 Awards

1. As classes complete, QTrial identifies:
   - Class placements (1st-4th)
   - HIT candidate (running best score across regular Obedience classes)
   - HC candidate (running best combined Open B + Utility B)
   - Rally HC, PHIT, PHC, RHC, HTQ as applicable
2. Ties for HIT/HC trigger run-off notifications.
3. Secretary (or a designated awards person) records run-off outcomes.
4. Final awards are announced and ribbons are distributed.

### 8.4 End of trial

1. All classes complete. All scores entered. All awards recorded.
2. Secretary clicks "Mark trial complete."
3. QTrial generates the marked catalog (catalog + results) and archives it.
4. QTrial generates the AKC submission package (PDF marked catalog, judges books, and populated Form JOVRY8 for Rally or Obedience equivalent) per §9 below.

---

## 9. Post-trial: AKC submission and results

**Actor:** Trial secretary.

For MVP (Obedience and Rally), AKC submission is PDF-based. Trial secretaries enter scores in real-time during the trial; after the trial, QTrial generates the three-artifact submission package and the secretary mails or emails it to AKC.

### 9.1 Generate the PDF submission package

Once all scores are entered and the secretary marks the trial complete (see §8.4):

1. QTrial generates the **marked catalog PDF** - the trial catalog with scores and placements annotated on each entry (REQ-SUB-001). Reference format: `Nov_2025_Sat_Marked_Catalog.pdf`.
2. QTrial generates (or confirms that scores are recorded onto) the **judges books** - one per class with the judge's own carbon-copy pages. The pink carbon copy goes to AKC per AKC distribution rules. White stays with AKC's records, Yellow with the club, Gold posted (REQ-SUB-002).
3. QTrial generates the **AKC Report of Rally Trial (Form JOVRY8 (03/23) v1.0 Edit)** or Obedience equivalent, auto-populated with event number, event date, dog counts, fee calculations per REQ-SUB-005 ($3.50 first entry + $3.00 additional per dog per trial + $10 secretary fee when applicable), and secretary contact info (REQ-SUB-003). Reference format: `Trial_Summary_report.pdf`.

All three artifacts are archived in `submission_records` with their S3 object keys.

### 9.2 Review

1. Secretary reviews the generated PDFs side by side in QTrial.
2. QTrial highlights any entries with validation warnings (missing reg number, score out of range, missing time for tied placements, etc.).
3. Secretary corrects source data and regenerates.

### 9.3 Send to AKC

1. Secretary downloads the package (or uses QTrial's "draft AKC email" helper to compose an email with the PDFs attached).
2. Secretary mails a physical package to AKC Event Operations (PO Box 900051, Raleigh NC 27675-9051), or emails the package to `rallyresults@akc.org` for Rally / the Obedience equivalent for Obedience (REQ-SUB-004). Payment accompanies the submission (check or credit card info on the form).
3. QTrial records the submission attempt in `submission_records` with the destination (`akc_destination_email` or `mail`), status `submitted`, and the fee total.
4. When AKC acknowledges acceptance (email reply), the secretary updates the status in QTrial to `accepted`.

### 9.4 Exhibitor results

1. QTrial sends results emails to exhibitors (entries marked with `results_email_sent_at`).
2. Exhibitors can view their results via their account.
3. P2: dog title progress tracking is automatically updated.

### 9.5 Financial reconciliation

1. Secretary runs the event financial report:
   - Total revenue collected
   - AKC fees owed (computed from REQ-SUB-005; this matches what went on Form JOVRY8)
   - Refunds issued
   - Expected bank deposits (for checks)
   - Stripe payouts (automatic via Connect)
2. Secretary uses this report to reconcile the club's books.

### 9.6 Post-MVP: XML submission for Agility

When QTrial adds Agility support, submission moves to an XML-based workflow conforming to the current AKC Agility schema. The `submission_records` table's `submission_type = 'xml'` branch covers this; `xml_payload_object_key` holds the generated XML. Deferred from MVP.

---

## 10. Mailing list and communication workflows

### 10.1 Mailing list buildup

- Every new exhibitor who enters an event via QTrial is opt-in-prompted to join the club's mailing list for their sport of interest.
- Existing mailing lists can be imported from the Access migration tool.

### 10.2 Sending to the mailing list

1. Club admin or secretary clicks "Send announcement."
2. Selects the list segment (e.g., "Obedience interested").
3. Composes a message (with variable substitution for recipient name).
4. Previews and sends.
5. QTrial dispatches via the email provider and records delivery status.

### 10.3 Individual communication

- Confirmation, waitlist, refund, and results emails are template-driven.
- Templates are editable per club (see REQ-EMAIL-001 and `email_templates` table).
- Each outgoing email records its delivery status and can be viewed in the exhibitor's communication log.

### 10.4 Template keys and variables

MVP template keys (stored in `email_templates.template_key`):

| Key | Trigger | Audience |
|---|---|---|
| `entry_confirmation` | Entry processed | Exhibitor (one email per entry) |
| `post_closing_reminder` | ~1 week pre-trial after closing | Owner (consolidated across all their dogs at the trial) |
| `cancellation_notice` | Trial cancelled | All exhibitors at the trial |
| `refund_confirmation` | Refund issued | Exhibitor who received the refund |

Variable substitution is simple `{{variable_name}}`. Jinja-style conditionals are post-MVP.

Variables available per template:

- **`entry_confirmation`**: `{{exhibitor_name}}`, `{{dog_call_name}}`, `{{dog_registered_name}}`, `{{classes_entered}}` (list), `{{armbands}}` (list, one per series), `{{jump_height}}`, `{{fees_breakdown}}`, `{{total_paid}}`, `{{trial_dates}}`, `{{venue}}`, `{{club_contact}}`, `{{secretary_signature}}`
- **`post_closing_reminder`**: `{{owner_name}}`, `{{dogs}}` (list of `{call_name, registered_name, entries_by_day, armbands}`), `{{schedule}}` (per-day running schedule), `{{registration_numbers}}` (for subject-line threading), `{{trial_dates}}`, `{{venue}}`, `{{club_contact}}`
- **`cancellation_notice`**: `{{exhibitor_name}}`, `{{trial_dates}}`, `{{cancellation_reason}}`, `{{refund_policy}}`, `{{club_contact}}`
- **`refund_confirmation`**: `{{exhibitor_name}}`, `{{dog_call_name}}`, `{{refund_amount}}`, `{{refund_reason}}`, `{{original_payment_method}}`, `{{club_contact}}`

Default templates are seeded on club creation; clubs override via the settings UI.

---

## 11. Migrating from Access

**Actor:** Secretary onboarding a new club from Obedience Solution or similar.

1. Secretary installs the QTrial migration tool (a small cross-platform desktop app that reads `.mde`/`.mdb` files). **[Decision PENDING - could also be server-side with file upload.]**
2. Secretary points the tool at their club's Access file pair.
3. Tool analyzes the file, produces a migration preview:
   - Dogs to import (with dedup based on AKC reg number)
   - Owners to import (with dedup based on email)
   - Judges to import
   - Events to import (marked as historical/read-only)
   - Any records that couldn't be parsed
4. Secretary reviews and confirms.
5. Tool calls QTrial's migration API, streaming the data.
6. Tool produces a completion report: X dogs imported, Y owners imported, Z records skipped.
7. Secretary signs into QTrial and verifies the imported data.
8. Historical events are available as read-only records but do not appear in active listings.

---

## Open questions / pending artifacts

The following workflow areas still need Deborah's input:

1. **A trial-weekend walkthrough narrative** - exactly what she does Friday setup through Sunday wrap-up. Voice memo is ideal.
2. **The paper-entry physical workflow** - what happens to the paper after she enters it? How does she track check deposit status?
3. **The move-up timing rules** - what specific AKC regulation version governs move-up deadlines for Obedience and Rally in 2026?
4. **The judge communication pattern** - how does she currently send judges their schedules and books? Email? Mail?
5. **Print logistics** - does she print catalogs herself or use a print shop? What are the quality expectations?
6. **Trial-day contingencies** - what does she do when a judge is late? When a dog bites another dog? These are edge cases but the software should stay out of her way while she handles them.
7. **Refund handling for check payments** - when a check refund is needed, does she write a check by hand or does QTrial integrate with some bill-payment service?

Resolved as of 2026-04-20:

- **AKC submission mechanism for Obedience/Rally in 2026**: PDF package (marked catalog + judges books + Form JOVRY8), sent via mail to PO Box 900051, Raleigh NC 27675-9051 or emailed to `rallyresults@akc.org`. Payment accompanies. See §9.
