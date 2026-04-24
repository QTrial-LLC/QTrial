# Research capture: Deborah's round-2 answers and 2026-04-23 artifact drop

**Date:** 2026-04-23
**Participants:** Robare Pruyn, Deborah Pruyn (trial secretary, Glens Falls Kennel Club), Claude
**Purpose:** Capture Deborah's annotated answers to Robare's eight follow-up questions sent 2026-04-23 9:28 AM, and the six AKC forms she attached to her reply. Resolves or moves several questions that were still open after the 2026-04-19 round.

---

## Context

After the 2026-04-19 artifact dump and the consolidated question list in `docs/research/2026-04-19-questions-for-deborah.md`, Robare sent a shorter, more pointed follow-up on 2026-04-23 morning. It did two things:

1. Listed seven items Robare was updating in the database design based on Deborah's first-pass feedback, for her to confirm or push back.
2. Posed eight specific questions Deborah had not yet answered in clear-enough detail for the database work.

Deborah replied the same evening at 6:25 PM, with annotations inline on both sections. She also attached six AKC PDFs. This note distills what changed from her annotations, preserving her direct wording where it matters for design.

**Source artifacts (committed in this PR):**

- `docs/research/attachments/2026-04-22-deborah-stuff-to-review-round-1-email.pdf` - the first-pass email she sent 2026-04-22 9:05 PM covering her reactions to Robare's design overview.
- `docs/research/attachments/2026-04-23-deborah-stuff-to-review-round-2-email.pdf` - the annotated reply that is the primary source for this note.
- `db/seed/akc/akc_forms/akc_rally_move_up_transfer_form_2017_11.pdf` (Q7)
- `db/seed/akc/akc_forms/akc_agility_move_up_form_0912.pdf` (attached for completeness)
- `db/seed/akc/akc_forms/akc_AEDSQ1_disqualification_for_attacking_person_1119.pdf`
- `db/seed/akc/akc_forms/akc_JEDTR1_emergency_procedures_and_disaster_plan_0822.pdf`
- `db/seed/akc/akc_forms/akc_rally_judges_book_cover_blank.pdf`
- `db/seed/akc/akc_forms/akc_rally_judges_book_blank_2017.pdf`

A seventh PDF, `akc_rally_hc_htq_tiebreaker_2018.pdf`, was sourced from AKC's website rather than Deborah's email, but is directly relevant to Q3 and Q4 and is tracked in the same directory.

---

## Confirmations of Robare's "things I'm updating"

Deborah put a checkmark on every item. A few carried inline additions worth preserving:

### 1. Armband series (double check)

No changes. Confirms the armband model: one armband per dog per trial, with shared numbers for the B classes that combine for High Triple (Advanced B + Excellent B + Master) and separate numbers for Choice (which does not combine). Configurable per trial at the club level.

### 2. Judges book - AKC requires physical original with wet signature

Her annotation: *"Unless and until they change this."*

AKC's no-electronic-judges-book requirement is **policy, not technical**. The submission pipeline for MVP treats the judges book as a printed artifact that Deborah signs and mails; QTrial produces PDFs for pre-trial printing only. If AKC later moves to electronic judges books, `submission_records` is shaped to accept an additional artifact key without a schema change (the PR 2b migration set aside `xml_payload_object_key` for a similar post-MVP eventuality; a future `judges_book_pdf_object_key` column would follow the same pattern).

Implication: no code change for MVP, but a note on `submission_records` that its current PDF-only artifact set is policy-bound, not data-model-bound.

### 3. Breed restrictions on events - mixed-breed flag needed

Her annotation: *"Exactly. Additionally in some instances, like [conformation], mixed breeds are not allowed."*

(Transcribed verbatim with a spelling normalization; Deborah writes "confirmation" where she means "conformation," the dog-sport.)

This is a real MVP constraint even though AKC conformation itself is post-MVP for QTrial. The breed-restriction model in PR 2c cannot be just a list of allowed breeds or breed groups; it also needs a **boolean for whether mixed-breed dogs ("All-American Dog" in AKC's canonical breed catalog) are allowed**. A breed-list-only shape would either force every restricted event to enumerate 288 breeds to exclude mixed, or force the renderer to treat "All-American Dog" as a special-case breed. An explicit flag is cleaner.

Implication for PR 2c: `events.mixed_breeds_allowed BOOL` (or equivalent on whatever table the breed-restriction model lands on), separate from the breed-list mechanism.

### 4. Payment methods

No changes. Accepted payment methods are a club-level setting; GFKC starts check/money-order only, Stripe can be enabled later.

### 5. Senior handler

Dropped from MVP. Confirmed.

### 6. Emergency contact

Added to entries in PR 2b scope. AEN999 confirms the field exists on the AKC Official Entry Form (at the bottom of the exhibitor block).

### 7. Owner-Handler Eligible

Her annotation: *"Again, this is mainly [conformation] where there is a class for owner/handler."*

Captured on the entry as a flag (checkbox mirrored from AEN999). No MVP behavior around it; QTrial stores the data so that when conformation support lands, owner-handler placement can be computed without backfilling historical entries.

---

## Answers to the eight follow-up questions

### Q1. Judges book columns

Robare asked whether the judges book carries only armband + breed + score + time + qualifying + placement, with no call name, handler name, or owner name.

Deborah's annotation: checkmark, plus *"Additionally the 'Q: score is computed in Rally as 70 - 100, where as in Obedience it is 170 - 200. After Masters should also be th HIT and HT."*

Two separate additions:

**Q-score range is sport-dependent.** Rally qualifying range is 70-100. Obedience qualifying range is 170-200. QTrial's schema already supports either (`canonical_classes.min_qualifying_score` and `canonical_classes.max_total_score`), but the **render layer** for the judges book and the marked catalog needs to know which range to display in the score-field hints. This matters especially for the "Final Score" column on the Rally book versus the "Score" column on the Obedience book: the valid ranges are labeled differently.

**After Masters, a HIT and HT summary must appear.** Deborah wrote "HIT and HT" where HIT is High in Trial and HT is High Triple. Looking at the Rally book layout in `akc_rally_hc_htq_tiebreaker_2018.pdf`, this maps to: after the Master class scoring section, the book needs a summary block (or adjacent tiebreaker-form block) showing the High in Trial winner and the Highest Scoring Triple winner for the trial. These are computed across classes, not within one class.

Implication for QTrial:

- Judges-book renderer for Rally must emit a post-Master HIT / HT summary block, not just per-class scoring pages.
- The Rally Q-range label differs from the Obedience Q-range label; make it sport-parameterized rather than hardcoded.

The column set Deborah confirmed (armband, breed, time, points lost, final score, placement, qualifying) matches exactly what `akc_rally_judges_book_blank_2017.pdf` shows. The real-world `gfkc_rally_judges_book_2025_06_21.pdf` (also in this PR) uses the same layout populated with actual dogs. No call name, handler, or owner anywhere.

### Q2. Steward sheet

Robare asked whether `Stewards_Board_Sat.pdf` from the 2026-04-19 drop is the current steward sheet format.

Deborah's annotation: checkmark. Nothing to reshape.

### Q3. High Triple eligibility

Robare asked whether a dog qualifying for High Triple must qualify in all three of Advanced B + Excellent B + Master at the same trial, with the High Triple score being the sum of the three class scores and time as tiebreaker.

Deborah's annotation: checkmark. Confirmed.

The `akc_rally_hc_htq_tiebreaker_2018.pdf` form adds authoritative citation: Rally Regulations Chapter 1, Sections 31 and 32. Section 31 covers Highest Combined (HC), Section 32 covers Highest Scoring Triple (HTQ). The tiebreaker form layout shows: for HTQ, each candidate row carries the dog's Advanced B score, Excellent B score, Master score, and the sum as "Total Score." Tiebreaking by time is not on the form itself; the time tiebreaker is described in the Regulations text.

Implication: the combined-awards computation reads from all three class rows for the same dog and sums. An `award_type = 'rhtq'` row on `trial_awards` stores the winner (the `trial_awards` schema was re-enumerated during PR 2a research but the enum addition for `rhtq` is still pending - flag for PR 2c).

### Q4. Additional-entry discount scope

Robare asked whether the `$30 / $25` first / additional rate in the June 2026 premium applies to just the Master / Choice / RAE / RACH title path, or also to a dog entered in Open B + Utility B going for HC or OTCH.

Deborah's annotation: *"I should have been clearer but I am focused on rally right now. It is for ANY double or triple Q in B Class in one trial. Most [definitely], including Open and Utility B."*

This expands the discount's scope materially. The original premium-list wording was too narrow; the actual rule is:

**The additional-entry discount applies to any dog entered in any combination of B classes at one trial where a combined award (HC, High Triple, HIT spanning classes, or equivalent Obedience awards leading to OTCH) is at stake.**

That means Open B + Utility B is a discount case, not just Rally's Master + Choice + RAE track.

Implication for QTrial: the discount rule cannot be hardcoded to "if class code is X and Y." It needs a data-driven mapping between classes and the combined-award groups they contribute to, so the fee engine can ask "does this dog's entry set cross two or more classes in the same combined-award group?" and apply the discount accordingly.

The `combined_award_groups` (or equivalent) reference table that was previously slated for P2 therefore **moves into PR 2c scope**. Without it, the fee engine is either wrong for Open B + Utility B or has to grow ad-hoc class lists that repeat the same logic per sport. Deborah's answer is the forcing function.

### Q5. Event Secretary vs Trial Chair

Robare asked whether his split was right: Event Secretary processes entries, Trial Chair oversees the trial on-site.

Deborah's annotation rewrote the Trial Chair role more expansively:

> "The trial chair has to [acquire] the judges and get them approved by the AKC (or whatever venue) and arrange all the [accommodations] for said judges. They also are responsible for getting all the [personnel] needed for the trial (Stewards, hospitality, time keeper, course builder, etc) and any payment due to judges, [secretary] etc. for expenses. Basically all the [peripherals] of putting on a trial. The trial can and has run without the trial Chair on site but they should be."
>
> "[Secretary] is in charge of the actual trial itself. Paperwork scores, entries etc."

Two distinct roles, often two distinct people, with non-overlapping responsibilities:

- **Trial Chair**: pre-trial arrangements. Acquires judges, gets AKC approval, arranges judge accommodations, recruits stewards, hospitality crew, timekeeper, course builder, and handles judge and secretary expense payments. Can be off-site during the trial in a pinch, though the expectation is they're there.
- **Trial Secretary**: on-the-day trial operations. Paperwork, scores, entries. Deborah is the trial secretary at GFKC.

Implication for PR 2c (not PR 2b):

- `events.trial_chair_user_id UUID REFERENCES users(id) ON DELETE SET NULL`
- `events.event_secretary_user_id UUID REFERENCES users(id) ON DELETE SET NULL`

The rest of the peripheral personnel (stewards, timekeeper, course builder, hospitality) go on a **post-MVP** `event_personnel` junction. MVP does not need structured records for every steward - the premium list documents that crew in free text, and the secretary workflow does not query it.

The GFKC June 2026 premium list `gfkc_rally_premium_2026_06.pdf` demonstrates the split cleanly: "Rally Trial Chair: Chris Argento" and "Trial Secretary: Debbie Pruyn" are separate blocks on the first page.

### Q6. Officers on the premium list

Robare asked whether the officers list changes per event or is club-level.

Deborah's annotation: *"It is a club side determination. It is the same for all the events as long as it does not change with the [yearly] elections. Like I just left the board for the club."*

Club-level, updated yearly with elections. Her note that she just left the board is a concrete example: the officers list on any premium list printed after her departure differs from one printed before.

MVP implication:

- `clubs.officers_json JSONB` covers the MVP case. Any premium list QTrial renders today reads the current officers from the club row.
- **Historical accuracy is NOT preserved in MVP.** A premium list regenerated after an officer turnover will show the current officers, not the officers as of the original premium-list print date. That is almost certainly wrong for published premium lists already sent to exhibitors.
- A post-MVP `club_officers` table with `effective_from` / `effective_to` dates captures historical state, and the premium-list renderer joins against that table by the premium-list's print-date. When QTrial is used long enough for historical-accuracy queries to matter, that table is the upgrade path. The `officers_json` column stays as the "current" convenience field; a `club_officers` table would be additive.

Deborah's comment that she "just left the board" is itself the example motivating the future upgrade.

### Q7. Move-up source citation

Robare asked for the regulatory source document to cite in the system when attaching the move-up regulation to AKC paperwork.

Deborah's annotation: *"I will attach to this email."*

She attached `Rally-Move-Up-Transfer-Form.pdf` (now renamed `akc_rally_move_up_transfer_form_2017_11.pdf` in this PR). The form header cites the authoritative source:

> "Refer to the Rally Regulations - Chapter 1, Section Transfers."

That is the citation QTrial should use when referencing the source of truth for Rally transfer rules. The form also summarizes the operational constraints (request in writing, 30 minutes prior to start of the trial, subject to class availability, A-to-B at the same level is a transfer if the host club allows it).

No code change for PR 2b or PR 2c. The citation goes into REQUIREMENTS.md (section reference) and into user-facing help text. The `akc_agility_move_up_form_0912.pdf` attached in the same email is the Agility equivalent, post-MVP.

### Q8. Cross-club dog identity

Robare asked, long term, whether QTrial should link the same dog across clubs (e.g. "Buddy at GFKC" and "Buddy at Saratoga Kennel Club" is the same dog), with post-MVP phrasing.

Deborah's annotation, in full:

> "They are really unrelated. The only case where there would be a relation is in a cluster trial where multiple clubs hold trials for multiple days with a different club hosting for each day. (this is pretty much confirmation only and the would be a superintendent level thing which as of now we cannot do. 2 companies have a monopoly on that.)"

Read carefully, this answer is narrower than a blanket "no":

- **For Obedience and Rally trial-secretary operations** (the QTrial MVP scope), dogs are unrelated across clubs. Each club's dog directory stands alone. Deborah's workflow does not need cross-club dog identity.
- **The one case where cross-club identity matters is cluster trials.** A cluster trial is when multiple clubs hold trials on consecutive days at the same venue, with a different club hosting each day. Cluster trials are predominantly a conformation phenomenon today, and they are run by **superintendents**, not by individual club secretaries. Deborah flags that the superintendent segment is an effective monopoly held by two companies (Onofrio and MB-F, historically). GFKC cannot host a cluster trial under its own secretary; the club would engage a superintendent.
- **Conformation is on QTrial's long-term roadmap** (ROADMAP.md Phase 7+). Cluster-trial support is part of that scope.

Framing the implication correctly:

- **MVP (PR 2b and earlier)**: each club's dog directory is standalone. No cross-club dog identity. Confirmed by Deborah's answer for the workflows she owns today.
- **Conformation work (post-MVP, roadmap)**: cross-club dog identity becomes necessary to support cluster trials, because the same dog runs on multiple days across multiple hosting clubs and the title history has to follow. The data model for that (a shared `registered_dogs` or similar table with its own RLS story) lands alongside the conformation scope, not before.

This is a **deferral**, not a rejection. Deborah's answer is precise about the operational scope of her current work; it does not close the door on QTrial's longer-term product direction.

The open-question text in DATA_MODEL.md item 3 ("Dog dedup across tenants: if the same dog is entered at Club A and Club B via QTrial, should they be the same dog record? MVP: separate records per tenant to simplify RLS. Consider a global registered_dogs shared table in P2.") aligns with this framing and stays open. The P2 framing might sharpen to "deferred to conformation work."

---

## Other items from the email body

### Obedience judges books source link

Deborah included a link at the bottom of the email:

    https://www.akc.org/sports/obedience/obedience-judging-information/judges-books-scoresheets-effective-1-1-2019/

This is AKC's canonical location for Obedience judges book and scoresheet templates effective January 1, 2019. The templates there are the source of truth QTrial's Obedience judges-book renderer targets. The current repo has only the Rally judges-book template (`akc_rally_judges_book_blank_2017.pdf`); adding the Obedience equivalents is an outstanding artifact-gathering task for Phase 3 judges-book work.

### Attached AKC forms inventory

Six attachments, all now tracked in `db/seed/akc/akc_forms/` after the rename in this PR:

- `akc_rally_move_up_transfer_form_2017_11.pdf` (Q7 citation source)
- `akc_agility_move_up_form_0912.pdf` (parallel to Rally transfer; Agility post-MVP)
- `akc_AEDSQ1_disqualification_for_attacking_person_1119.pdf` (incident-reporting form; 72-hour submission to `eventrecords@akc.org`)
- `akc_JEDTR1_emergency_procedures_and_disaster_plan_0822.pdf` (club-configuration surface, post-MVP)
- `akc_rally_judges_book_cover_blank.pdf` (Rally judges-book template, cover page)
- `akc_rally_judges_book_blank_2017.pdf` (Rally judges-book template, interior page)

See `db/seed/akc/akc_forms/README.md` for full descriptions and consumption maps.

---

## Implications for design docs

This list is the input to a follow-up doc-update pass. Each entry names the exact change.

**DOMAIN_GLOSSARY.md**
- Add definitions: High Triple (HT), High Triple Qualifying (RHTQ / HTQ), Highest Combined (HC) with the Rally Regulations Chapter 1 Sections 31-32 citation.
- Clarify "mixed breed" as it relates to All-American Dog in the canonical breed catalog, referencing the Q3 follow-up item about conformation-only mixed-breed exclusion.
- Add definitions: Trial Chair, Trial Secretary, distinguishing the two per Q5.

**REQUIREMENTS.md**
- Update the judges-book column specification to reflect the Rally-versus-Obedience Q-range difference (70-100 vs 170-200) per Q1.
- Add a REQ-SUB-* line for the post-Master HIT / HT summary block in the Rally judges book per Q1.
- Widen REQ-FEE-* (additional-entry discount) to cover any B-class combination contributing to a combined award, not just the Master-Choice-RAE-RACH path, per Q4. Reference the `combined_award_groups` reference concept.
- Add REQ-EVENT-* lines for `trial_chair_user_id` and `event_secretary_user_id` per Q5.
- Add REQ-CLUB-* for `clubs.officers_json` with the yearly-elections update cadence per Q6, and a post-MVP marker for historical preservation.
- Add the Rally Regulations Chapter 1 Section Transfers citation in the transfer rules section per Q7.
- Add the 72-hour incident-reporting path to `eventrecords@akc.org` using AEDSQ1 per the AEDSQ1 form.

**DATA_MODEL.md**
- PR 2c scope expansion: `combined_award_groups` reference table (moved from P2 per Q4 decision log), `events.trial_chair_user_id` and `events.event_secretary_user_id` (Q5), `clubs.officers_json` (Q6), `events.mixed_breeds_allowed` alongside the breed-list approach (Q3 follow-up).
- Open questions / pending decisions item 3 (dog dedup across tenants): text stays open; consider rephrasing from "Consider a global registered_dogs shared table in P2" to "Deferred to conformation work; cluster-trial support will require a shared registered_dogs or equivalent model" to reflect Deborah's Q8 answer precisely.
- `trial_awards` needs `rhtq` added to `award_type` enum (note for PR 2c).

**WORKFLOWS.md**
- §10 email-template variables: `trial_chair_name`, `trial_chair_contact`, `event_secretary_name`, `event_secretary_contact` need to be available to relevant templates per Q5.
- Incident-reporting workflow: add a §11 or similar covering the AEDSQ1 72-hour path. Not required for entry-flow MVP but needs a slot in the workflow documentation.

**ROADMAP.md**
- Phase 7+ conformation scope gains an explicit "cross-club dog identity via shared registered_dogs table or equivalent" line, per Q8.

**PROJECT_STATUS.md**
- Updated in this PR. See commit 4.

---

## Implications for PR scope

**PR 2b (tenant-scoped gap-fill, currently in flight)**

No scope change. None of Deborah's answers touch the 9 tables PR 2b creates. PR 2b remains on its own branch and its CHECKPOINT 3 RLS work proceeds independently of this PR.

**PR 2c (table alterations + armband restructure + breed restrictions)**

Scope expands to include the following items drawn from Deborah's answers:

- `combined_award_groups` reference table (was P2, now PR 2c per Q4)
- `events.trial_chair_user_id` and `events.event_secretary_user_id` (Q5)
- `clubs.officers_json` (Q6)
- `events.mixed_breeds_allowed` BOOL alongside the breed-list approach (Q3 follow-up)
- `trial_awards.award_type` enum extension to include `rhtq` (Q3)

PR 2c scope is large enough that a split into PR 2c + PR 2d may make sense when the PR 2c prompt is written. That decision belongs to the PR 2c scoping session, not this research note.

**Future (conformation scope, post-MVP)**

- Shared `registered_dogs` or equivalent for cross-club dog identity (Q8)
- `club_officers` table with `effective_from` / `effective_to` for historical premium-list accuracy (Q6 long-term)
- `event_personnel` junction for stewards, timekeeper, course builder, hospitality (Q5 long-term)
- Obedience judges-book templates (from AKC's January 1, 2019 set linked in the email)
