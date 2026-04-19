# OffLeash - Workflows

**Status:** Draft v0.1 - **provisional**; narrative workflows derived from Deborah's outline, the Access schema, and typical AKC trial secretary practices. Will be refined once Deborah provides a trial-weekend walkthrough.
**Last updated:** 2026-04-19

---

## How to read this document

This document describes user-facing workflows, step by step. It is written as narrative: what a user is trying to accomplish, what they see, and what happens. It complements `REQUIREMENTS.md` (which is structural) and is the primary artifact Deborah (and later other secretaries) should review and annotate.

Each workflow notes the primary actor, triggering context, and the system's expected behavior. Things marked **[PENDING]** need confirmation from Deborah's walkthrough.

---

## 1. Club and account setup

### 1.1 A new club signs up

**Actor:** First user of a club (typically the club's trial secretary or treasurer).

1. User visits `offleash.dog` and clicks "Sign up your club."
2. User provides their own name, email, and password (or signs up via Google/Apple).
3. Keycloak creates the account; OffLeash creates a `users` row.
4. User is prompted: "Create a new club or join an existing one?" User selects "Create new."
5. User enters the club's name, abbreviation, AKC club number, license status, and address.
6. OffLeash creates the `clubs` row, the `user_club_roles` row (user as `club_admin`), and prompts the user to upload a logo (optional).
7. User lands on the empty club dashboard with a prompt to create their first event.

**Edge cases:**
- AKC club number already registered under a different user → OffLeash flags this and asks for resolution. For MVP, Robare or a platform admin resolves manually; P2 automates via a claim-verification workflow.
- User wants to belong to multiple clubs → they complete sign-up for one, then repeat the flow to be invited (by someone else) or create another.

### 1.2 An exhibitor signs up

**Actor:** Dog owner wanting to enter a trial.

1. Exhibitor clicks an online-entry link from a club's premium list (or finds the trial via OffLeash's public event directory).
2. Exhibitor is prompted to sign in or create an account.
3. On creation: name, email, password, home address, phone.
4. OffLeash creates the `users` row. No club role is created - an exhibitor is a user without any club-role grants.
5. Exhibitor is redirected to the entry flow for the specific trial they were trying to enter.

### 1.3 A club admin invites a trial secretary

**Actor:** Club administrator.

1. From the club dashboard, admin clicks "Team" → "Invite user."
2. Admin enters the invitee's email and selects the role (Trial Secretary).
3. OffLeash sends an invitation email.
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
3. Secretary saves. OffLeash creates:
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
3. Secretary saves. OffLeash creates a `trials` row.
4. Secretary clicks into the trial and selects which classes are offered:
   - A sport-filtered list of canonical classes is shown with checkboxes
   - Secretary checks the classes to be offered; can override dogs-per-hour, ring number, class limit
5. Secretary saves. OffLeash creates `trial_class_offerings` rows.

### 2.3 Assign judges

1. Secretary clicks "Judges" on the event dashboard.
2. Secretary adds judges (select from club's judge directory or add new).
3. Secretary assigns each judge to one or more classes across the event.
4. Secretary saves. OffLeash creates `judge_assignments` rows.
5. **[PENDING]** Should OffLeash auto-email judges with their assignments and a draft schedule? Probably yes, but only when the secretary explicitly triggers it.

### 2.4 Generate the premium list

1. Once judges, classes, and fees are configured, secretary clicks "Generate premium list."
2. OffLeash renders an HTML preview with all event details.
3. Secretary can edit club-specific content (welcome text, refund policy, trophy descriptions) via inline editors.
4. Secretary clicks "Export PDF" → OffLeash generates the premium list PDF and makes it downloadable.
5. Secretary can also publish a public landing page for the event at `offleash.dog/e/<slug>` that serves as the online premium list.

### 2.5 Open entries

1. When the entry opening date/time arrives, OffLeash automatically transitions the event status to `open`.
2. The online entry page becomes active.
3. Optionally, OffLeash sends a mailing-list notification to the club's mailing list.

---

## 3. Submitting an entry (exhibitor side)

**Actor:** Exhibitor.

### 3.1 Find the event

1. Exhibitor arrives at the online entry page (via email link, club website, or OffLeash's public event directory).
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

1. OffLeash shows the entry summary with fees broken down:
   - First entry fee
   - Additional class fees (at the additional rate)
   - Special surcharges (brace, team)
   - Junior handler discount if applicable
   - Subtotal
   - OffLeash service fee (if the club passes it to exhibitors)
   - Card processing fee (if passed to exhibitors)
   - Total
2. Exhibitor enters (or selects a saved) credit card via Stripe Elements.
3. Exhibitor clicks "Submit entry."

### 3.5 Confirmation

1. OffLeash validates:
   - Payment succeeds
   - Class still has space (atomic check against the trial's entry limit)
2. If everything validates:
   - `entries`, `entry_lines`, `payments` rows are created
   - Armband is assigned (if the club's scheme is per-entry; otherwise assigned later in bulk)
   - Confirmation email is queued to `offleash-workers`
   - Worker sends the email and marks `confirmation_email_sent_at`
3. If the class is full, the entry is placed on waitlist:
   - Payment is captured or authorized per club policy **[PENDING]**
   - Waitlist email is sent
4. Exhibitor sees a confirmation page with entry details and an option to add another dog or another entry.

---

## 4. Managing paper entries

**Actor:** Trial secretary.

Some clubs still accept paper entries by mail.

1. Secretary receives a paper entry in the mail.
2. Secretary logs into OffLeash and clicks "Enter paper entry" on the event.
3. Secretary enters:
   - Exhibitor contact info (or selects an existing exhibitor)
   - Dog info (or selects an existing dog in the directory)
   - Classes being entered
4. Secretary records payment:
   - Method: Check, money order, cash, or coupon
   - Reference number (check #)
   - Date received
5. Secretary clicks "Submit." OffLeash creates the same rows as for online entries but with payment already recorded.
6. Secretary can trigger a confirmation email (or indicate the exhibitor prefers mail).
7. **[PENDING]** How does Deborah typically handle a mailed entry form today? Does she write on the paper? Staple the check? Keep the paper? OffLeash should mirror whatever physical-artifact workflow she needs.

---

## 5. Entry changes before closing

**Actor:** Exhibitor (typically) or secretary (on exhibitor's behalf).

### 5.1 Cancel an entry

1. Exhibitor opens their entry and clicks "Cancel."
2. OffLeash warns about refund policy.
3. Exhibitor confirms. OffLeash:
   - Marks the `entry_lines.status = 'scratched'`
   - Initiates refund via Stripe (if before closing and club policy allows full refund)
   - Promotes the next waitlisted entry, if any
   - Sends refund confirmation and promotion emails

### 5.2 Add a class to an existing entry

1. Exhibitor adds an entry in an additional class for the same dog at the same trial.
2. OffLeash calculates the additional-entry fee (not first-entry fee).
3. Payment is processed as usual.

### 5.3 Change jump height

1. Exhibitor updates the dog's jump height on an existing entry line.
2. OffLeash records the change. No fee change.
3. If the catalog has already been generated, the secretary is notified.

---

## 6. After closing: move-ups, transfers, and day-of changes

**Actor:** Exhibitor (initiates) and secretary (approves and acts).

### 6.1 Move-up request

A dog earned its CD title yesterday and is currently entered in Novice B at a trial starting in 3 days. The exhibitor wants to move up to Open A.

1. Exhibitor clicks "Request move-up" on the entry.
2. Exhibitor provides: current class, target class, date the dog earned the qualifying title, supporting documentation (optional file upload).
3. OffLeash validates:
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

1. Exhibitor notifies the secretary (via OffLeash, email, text, or phone).
2. Secretary marks the entry line `withdrawn` with reason `bitch_in_season`.
3. OffLeash calculates the refund: full entry fee less the club's processing fee (default $5/class, configurable).
4. Secretary approves the refund. OffLeash initiates a Stripe refund (or records the need for a check refund if payment was by check).
5. Exhibitor receives a refund confirmation email.

### 6.4 Day-of changes

When the trial is running, the secretary may need to:

- Mark an entry as `absent` (dog didn't show up)
- Mark an entry as `excused` with reason (judge excused)
- Mark an entry as `dq` with reason (judge disqualified)
- Record run-off results
- Process a scratched dog after the trial has started (usually no refund, but the dog still needs to be marked in the catalog)

OffLeash's UI for day-of changes needs to be fast - the secretary is doing this on a laptop at a folding table while exhibitors are checking in.

---

## 7. Pre-trial paperwork generation

**Actor:** Trial secretary.

Once entries are closed (or close to closing), the secretary generates the paperwork package.

### 7.1 Armbands

1. Secretary clicks "Generate armbands."
2. OffLeash assigns armband numbers according to the club's scheme (if not already assigned per-entry).
3. OffLeash produces:
   - A spreadsheet-view of all armbands for the secretary's reference
   - Printable armband cards (PDF, 4 or 6 per page) with dog name, handler, class, trial
4. Secretary can preview and then print.

### 7.2 Catalog

1. Secretary clicks "Generate catalog."
2. OffLeash renders the catalog HTML with all accepted entries, sorted by class, sorted within class by armband (or custom order), with full registered name formatting.
3. Secretary previews in the browser.
4. Secretary clicks "Export PDF."
5. **[PENDING]** Does Deborah want to print herself, send to a print shop, or have OffLeash integrate with a print-on-demand service?

### 7.3 Judge's books

1. Secretary clicks "Generate judge's books."
2. OffLeash produces one PDF per judge with all their classes.
3. Secretary prints and assembles.

### 7.4 Running order

1. Secretary clicks "Generate running order."
2. OffLeash produces a per-class running order with armband numbers, jump heights, and handler names.
3. Exhibitors can (P2) see their own running-order position via their account.

### 7.5 Scribe sheets

1. Secretary clicks "Generate scribe sheets."
2. OffLeash produces Obedience scribe sheets (one per entry in Obedience classes) with pre-printed armband, class, and exercise list.
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
2. Judges use their printed judge's books to record scores.
3. At the end of each class (or throughout the day), judges hand completed books to the secretary.
4. Secretary enters scores into OffLeash, one class at a time:
   - For each entry in the class: score, Q/NQ, placement, time (if applicable)
   - Special flags: absent, excused, DQ
5. OffLeash validates scores and computes placements and awards.

### 8.3 Awards

1. As classes complete, OffLeash identifies:
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
3. OffLeash generates the marked catalog (catalog + results) and archives it.
4. OffLeash generates the AKC submission package (XML + supporting documents).

---

## 9. Post-trial: AKC submission and results

**Actor:** Trial secretary.

### 9.1 Review submission

1. Secretary reviews the generated AKC XML/CSV submission in OffLeash.
2. OffLeash highlights any records with validation warnings (e.g., missing reg number, score out of range, etc.).
3. Secretary corrects issues and regenerates.

### 9.2 Submit to AKC

1. Secretary clicks "Submit to AKC."
2. **[PENDING]** What is the actual submission mechanism in 2026? Email? Upload portal? API? The Access schema references a 2004 XSD, which suggests email-attached XML is the historical method. OffLeash needs to support whatever current AKC expects.
3. OffLeash records the submission attempt in `submission_records`.
4. If AKC accepts/rejects the submission (either immediately via API or later via email), the secretary updates the status in OffLeash.

### 9.3 Exhibitor results

1. OffLeash sends results emails to exhibitors (entries marked with `results_email_sent_at`).
2. Exhibitors can view their results via their account.
3. P2: dog title progress tracking is automatically updated.

### 9.4 Financial reconciliation

1. Secretary runs the event financial report:
   - Total revenue collected
   - AKC fees owed
   - Refunds issued
   - Expected bank deposits (for checks)
   - Stripe payouts (automatic via Connect)
2. Secretary uses this report to reconcile the club's books.

---

## 10. Mailing list and communication workflows

### 10.1 Mailing list buildup

- Every new exhibitor who enters an event via OffLeash is opt-in-prompted to join the club's mailing list for their sport of interest.
- Existing mailing lists can be imported from the Access migration tool.

### 10.2 Sending to the mailing list

1. Club admin or secretary clicks "Send announcement."
2. Selects the list segment (e.g., "Obedience interested").
3. Composes a message (with variable substitution for recipient name).
4. Previews and sends.
5. OffLeash dispatches via the email provider and records delivery status.

### 10.3 Individual communication

- Confirmation, waitlist, refund, and results emails are template-driven.
- Templates are editable per club.
- Each outgoing email records its delivery status and can be viewed in the exhibitor's communication log.

---

## 11. Migrating from Access

**Actor:** Secretary onboarding a new club from Obedience Solution or similar.

1. Secretary installs the OffLeash migration tool (a small cross-platform desktop app that reads `.mde`/`.mdb` files). **[Decision PENDING - could also be server-side with file upload.]**
2. Secretary points the tool at their club's Access file pair.
3. Tool analyzes the file, produces a migration preview:
   - Dogs to import (with dedup based on AKC reg number)
   - Owners to import (with dedup based on email)
   - Judges to import
   - Events to import (marked as historical/read-only)
   - Any records that couldn't be parsed
4. Secretary reviews and confirms.
5. Tool calls OffLeash's migration API, streaming the data.
6. Tool produces a completion report: X dogs imported, Y owners imported, Z records skipped.
7. Secretary signs into OffLeash and verifies the imported data.
8. Historical events are available as read-only records but do not appear in active listings.

---

## Open questions / pending artifacts

The following workflow areas need Deborah's input:

1. **A trial-weekend walkthrough narrative** - exactly what she does Friday setup through Sunday wrap-up. Voice memo is ideal.
2. **The paper-entry physical workflow** - what happens to the paper after she enters it? How does she track check deposit status?
3. **The move-up timing rules** - what specific AKC regulation version governs move-up deadlines for Obedience and Rally in 2026?
4. **The judge communication pattern** - how does she currently send judges their schedules and books? Email? Mail?
5. **Print logistics** - does she print catalogs herself or use a print shop? What are the quality expectations?
6. **The AKC submission mechanism in 2026** - email attachment? Upload portal? API? We need to confirm with an AKC contact.
7. **Trial-day contingencies** - what does she do when a judge is late? When a dog bites another dog? These are edge cases but the software should stay out of her way while she handles them.
8. **Refund handling for check payments** - when a check refund is needed, does she write a check by hand or does OffLeash integrate with some bill-payment service?
