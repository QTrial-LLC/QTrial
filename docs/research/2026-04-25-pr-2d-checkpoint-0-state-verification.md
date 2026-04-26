# PR 2d CHECKPOINT 0 - Phase A state verification

**Branch:** `feat/pr-2d-events-clubs-awards` off main at `9543fab1940b4a03b88292e902a9e720aabe81cf`
**Phase A author:** Claude (continuing from prior session)
**Status:** STOPPED at end of Phase A; one DRIFT finding requires Robare's
direction before Phase B begins.

The kickoff for PR 2d directs that "if Phase A surfaces drift catastrophic
enough that one or more Phase B questions become moot or materially
different, stop after Phase A, note this in the verification report under
a Phase B impact section, and ask Robare for direction before continuing."
Phase A surfaces exactly one such drift (item A3 below); Phase A is
otherwise clean. Phase B is therefore not started.

---

## A1. Baseline (CONFIRMED with date drift)

- HEAD: `9543fab` "PR 2c-beta: dogs table reconciliation (#17)"
- Branch off main: `feat/pr-2d-events-clubs-awards`
- Migration count: 53 up/down pairs (counted; matches expectation)
- Working tree: clean except for the new `memory/` scratch dir, which is
  not staged or tracked
- DATA_MODEL.md: `Last updated: 2026-04-26`, version v0.3

**Date drift to flag.** Three different "now" timestamps exist in the
repository at the moment of this report:

| Source | Date |
|---|---|
| CLAUDE.md `currentDate` | 2026-04-23 |
| Git HEAD commit timestamp | 2026-04-25 13:42 -0400 |
| DATA_MODEL.md "Last updated" | 2026-04-26 |
| Most-recent Decisions-log entries | 2026-04-26 |
| PR 2d kickoff narrative (per session summary) | "main is at 9543fab dated 2026-04-26" |

The kickoff narrative claims main is dated 2026-04-26; git says 2026-04-25.
The CLAUDE.md context has today as 2026-04-23. The 2026-04-26 dates inside
DATA_MODEL.md and the four most-recent Decisions-log entries appear to be
forward-dated relative to commit timestamps. Not a blocker for Phase A but
worth Robare's eye: if the system clock and the in-doc dates disagree by
multiple days, future PR 2d Decisions-log entries should pick one
convention and stick to it.

The path of this report uses 2026-04-25 because the kickoff prompt
specified that filename verbatim.

## A2. Entry-subtree reconciliation (CONFIRMED)

PR 2c-surgery (merged as #16) and PR 2c-beta (merged as #17) collectively
land the entry-pipeline reconciliation. Verified directly in the
migrations:

| Change | Migration | State |
|---|---|---|
| `entries.armband` dropped | `20260425120200_drop_entries_armband.up.sql` | DROPPED |
| `entries.handler_name`, `junior_handler_number`, `is_senior_handler` dropped | `20260425120300_drop_entries_handler_columns.up.sql` | DROPPED |
| `entry_lines.jump_height_inches` dropped | `20260425120400_drop_entry_lines_jump_height.up.sql` | DROPPED |
| `entry_lines.armband_assignment_id` (UUID FK to armband_assignments, ON DELETE SET NULL) added | `20260425120000_add_entry_lines_handler_and_armband_columns.up.sql` | ADDED |
| `entry_lines.handler_contact_id` (UUID FK to owners, ON DELETE RESTRICT, NOT NULL) added | same migration | ADDED |
| `entry_lines.junior_handler_akc_number` TEXT added | same migration | ADDED |
| `entry_line_results.time_started`, `time_finished` (TIMESTAMPTZ) added | `20260425120100_add_entry_line_results_timing_and_rach.up.sql` | ADDED |
| `entry_line_results.time_on_course` (INTERVAL) added | same migration | ADDED |
| `entry_line_results.rach_points` (INT, CHECK >= 0) added | same migration | ADDED |
| `dogs.registration_type` ENUM added | `20260426120000_add_dogs_registration_type.up.sql` | ADDED |
| `dogs` parsed_* arrays + name parser columns added | `20260426120100_add_dogs_name_parser_columns.up.sql` | ADDED |
| `dogs.co_owners_text` dropped | `20260426120300_drop_dogs_co_owners_text.up.sql` | DROPPED |
| `dogs` sire/dam prefix/suffix four columns dropped | `20260426120400_drop_dogs_sire_and_dam_title_columns.up.sql` | DROPPED |

The entry pipeline is therefore in the post-2c-surgery + post-2c-beta
state the kickoff assumes. No further drift in the entry/dog subtree.

## A3. DRIFT - Decisions-log scope-lock for `events.mixed_breeds_allowed` NOT FOUND

**The kickoff prompt asserts** that a Decisions-log entry locks the scope
of `events.mixed_breeds_allowed` to "ship the BOOL flag, defer the
breed-list model" for PR 2d.

**No such entry exists in `docs/PROJECT_STATUS.md`.**

Evidence (all `grep -n` against the current tree):

1. The Decisions log contains 32 entries between 2026-04-20 and
   2026-04-26 (heading lines 243, 265, 286, 312, 344, 363, 390, 411, 431,
   455, 487, 504, 525, 547, 567, 589, 611, 634, 654, 671, 687, 703, 726,
   744, 758, 772, 790, 806, 820, 836, plus the two newest 2026-04-26
   entries at 243 and 265). None of the headings reference
   mixed_breeds_allowed, breed restrictions, or All-American Dog scope.

2. The only mentions of mixed_breeds_allowed in PROJECT_STATUS.md are:
   - Lines 33-35 (in-flight narrative): lists mixed_breeds_allowed as
     part of "Deborah's 2026-04-23 plumbing" alongside trial chair /
     secretary columns, officers_json, combined_award_groups, rhtq enum,
     judges_book_pdf_object_key, signed-scan handling.
   - Lines 895-896 (Known-gaps): **"events breed restrictions, including
     events.mixed_breeds_allowed BOOL alongside the breed-list approach
     (Deborah's item 3 follow-up)"**. This is the OPPOSITE of "BOOL
     only, defer breed-list."
   - Lines 969-971 (PR 2d kickoff prep): lists mixed_breeds_allowed as
     a topic the PR 2d prompt should cover.

3. The source research note at
   `docs/research/2026-04-23-deborah-round-2-answers.md` is consistent
   with PROJECT_STATUS.md and inconsistent with the kickoff:
   - Line 49-57: explicit reasoning that `events.mixed_breeds_allowed
     BOOL` is needed **alongside** the breed-list mechanism, because
     "a breed-list-only shape would either force every restricted event
     to enumerate 288 breeds to exclude mixed, or force the renderer
     to treat 'All-American Dog' as a special-case breed. An explicit
     flag is cleaner."
   - Lines 255 and 284: PR 2c scope-expansion summary lists
     mixed_breeds_allowed "alongside the breed-list approach."

4. `docs/REQUIREMENTS.md` line 87 states the breed-restriction model
   needs both group/breed lists AND mixed-breed inclusion as separate
   knobs.

The corpus is unanimous: mixed_breeds_allowed is intended to ship
**alongside** the breed-list model, not as a deferral substitute for it.
The kickoff's claim of a "BOOL only, defer breed-list" scope-lock is not
in the repository.

### Phase B impact

Per the kickoff: "If the lock cannot be found, say so and stop; this
changes Phase B item B6 materially."

B6 (the breed-restriction design question for PR 2d) cannot be answered
on the assumption the kickoff stated. Three possibilities and the
corresponding Phase B reshape:

(a) **The lock was a verbal decision that was never written down.** If
so, it must be added to the Decisions log as part of PR 2d so future
sessions can find it, and the Known-gaps line 895-896 must be rewritten
to match. B6 then becomes "ship `events.mixed_breeds_allowed BOOL`,
defer breed/group lists to a later PR; record the rationale."

(b) **The kickoff inverted the intent.** The actual decision (per all
written sources) is that BOTH ship together: the BOOL flag AND a
breed-list model land in PR 2d. B6 then becomes a much larger design
question covering BreedRestriction / GroupRestriction / BreedExclusion /
GroupExclusion shape (as REQUIREMENTS.md line 87 frames them), plus
the integration with the BOOL.

(c) **The lock is real and is documented somewhere I didn't search.**
PR notes, GitHub PR descriptions, Slack, calendar invites, etc. If so,
point me at the source and I'll cite it in B6 verbatim.

**Recommendation: option (a).** The breed-list model is a meaningful
design effort (288 breeds, group hierarchy, exclusion semantics) and PR
2d is already large with clubs/events/awards/signed-scan plumbing.
Shipping the BOOL only and deferring the list model is the small,
correct first move. But this is Robare's call, not mine.

**Phase A is otherwise clean and continues below; Phase B is paused
pending direction.**

## A4. ENUM state inventory (CONFIRMED, with one prompt-side error)

| ENUM | Values | Defined in | Notes |
|---|---|---|---|
| `armband_scheme` | per_trial, per_event, per_day, per_class | `20260419140100_create_events_and_days.up.sql:21` | 4 values, default `per_event` on `events.armband_scheme` |
| `award_type` (for trial_awards) | hit, hc, phit, phc, rhit, rhc, rhtq, htq | `20260419140400_create_judge_assignments_and_awards.up.sql:45` | 8 values |
| `entry_line_status` | pending_payment, active, waitlist, scratched, withdrawn, transferred, moved_up, absent, excused, dq | `20260420120500_create_entry_lines.up.sql:22` | 10 values |
| `event_status` | draft, open, closed, in_progress, complete, archived | `20260419140100_create_events_and_days.up.sql:12` | 6 values |
| `trial_status` | draft, open, closed, running, complete | `20260419140200_create_trials_and_class_offerings.up.sql:15` | 5 values |

**Prompt-side error to flag.** The kickoff lists `entry_status` as one
of the ENUMs to inventory. **There is no `entry_status` ENUM and no
`entries.status` column.** Status lives only on `entry_lines`.
`grep -n entry_status db/migrations/*.sql` returns zero hits. This
matches the 2026-04-20 Decisions-log entry "Entry status is an ENUM
state machine, not parallel booleans" (line 806) which is about
entry_lines, not entries. If Phase B item B-something assumed a
top-level entries.status, that assumption needs to drop.

## A5. Full column listings for PR 2d tables (CONFIRMED)

**`clubs`** (current shape after `20260419130000_create_clubs.up.sql`
and `20260419140000_tighten_clubs_with_check.up.sql`):

```
id UUID PK
display_name TEXT NOT NULL
abbreviation TEXT
akc_club_number TEXT
ukc_club_number TEXT
akc_status akc_club_status NOT NULL DEFAULT 'none'
logo_object_key TEXT
primary_contact_user_id UUID
billing_status billing_status NOT NULL DEFAULT 'active'
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
deleted_at TIMESTAMPTZ
created_by UUID REFERENCES users(id) ON DELETE SET NULL
updated_by UUID REFERENCES users(id) ON DELETE SET NULL
```

**Absent (PR 2d candidates per Known-gaps and Deborah Q&A):**
`officers_json` (Q6), `website_url`, `phone`, `email`, `mailing_address`.

**`events`** (current shape after `20260419140100_create_events_and_days.up.sql`,
no later migrations modify):

```
id UUID PK
club_id UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE
registry_id UUID NOT NULL REFERENCES registries(id)
name TEXT NOT NULL
cluster_name TEXT
venue_name TEXT
venue_address_line1, _line2, _city, _state, _postal_code, _country_code TEXT
entry_opens_at, entry_closes_at, moveup_deadline_at TIMESTAMPTZ
armband_scheme armband_scheme NOT NULL DEFAULT 'per_event'
armband_start_number INT NOT NULL DEFAULT 1
armband_interval INT NOT NULL DEFAULT 1
catalog_fee NUMERIC(10, 2)
waitlist_accepted BOOL NOT NULL DEFAULT TRUE
status event_status NOT NULL DEFAULT 'draft'
created_at, updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
deleted_at TIMESTAMPTZ
created_by, updated_by UUID REFERENCES users(id) ON DELETE SET NULL
```

CHECK constraints: entry window ordered, armband_start_number >= 0,
armband_interval > 0, catalog_fee >= 0.

**Absent (PR 2d candidates):** `mixed_breeds_allowed` (per A3 above),
`event_secretary_user_id` (Q5), `trial_chair_user_id` (Q5),
`combined_award_groups` reference (separate table per Q4).

**`event_days`** (no PR 2d work expected; included for completeness):

```
id, club_id, event_id, day_number INT, date DATE, start_time TIME,
created_at, updated_at, deleted_at, created_by, updated_by.
```

**`trials`** (current shape after
`20260419140200_create_trials_and_class_offerings.up.sql`,
no later migrations modify):

```
id UUID PK
club_id UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE
event_day_id UUID NOT NULL REFERENCES event_days(id) ON DELETE CASCADE
trial_number INT NOT NULL CHECK (>= 1)
sport sport NOT NULL
akc_event_number TEXT (partial unique global; YYYYNNNNNN)
trial_chairperson TEXT (free text, NOT a user FK)
start_time TIME
entry_limit INT
first_class_fee, additional_class_fee NUMERIC(10, 2)
nonregular_class_fee, nonregular_second_class_fee NUMERIC(10, 2)
brace_fee, team_fee, rally_pairs_fee, rally_team_fee NUMERIC(10, 2)
first_class_fee_jr, additional_class_fee_jr NUMERIC(10, 2)
status trial_status NOT NULL DEFAULT 'draft'
created_at, updated_at, deleted_at TIMESTAMPTZ
created_by, updated_by UUID REFERENCES users(id) ON DELETE SET NULL
```

**Absent (PR 2d candidates):** Q5 calls for `trial_chair_user_id` (FK
to users) replacing/coexisting with the free-text `trial_chairperson`.

**`trial_class_offerings`** (current shape after
`20260419140200` plus `20260420120000_fix_reference_day_restrict`):

```
id, club_id, trial_id, canonical_class_id (FK)
ring_number INT NOT NULL DEFAULT 1 CHECK (>= 1)
class_limit INT
scheduled_start_time TIME
running_order_strategy running_order_strategy NOT NULL DEFAULT 'short_to_tall'
running_order_reference_day_id UUID REFERENCES event_days(id) ON DELETE SET NULL
jump_start_height INT
per_dog_minutes, walkthrough_minutes, ribbon_presentation_minutes,
class_transition_minutes NUMERIC(4, 1)
created_at, updated_at, deleted_at, created_by, updated_by
```

CHECK couples `running_order_strategy = 'reverse_previous_day'` <=>
`running_order_reference_day_id IS NOT NULL`.

**Absent (PR 2d candidates):** `judges_book_pdf_object_key` (one PDF per
offering, decided in PR 2b CHECKPOINT 4 to land here in PR 2c then
deferred to PR 2d - see also A6 below).

**`trial_awards`** (current shape after
`20260419140400_create_judge_assignments_and_awards.up.sql` plus FK
backfill in `20260420120400_create_entries.up.sql`):

```
id, club_id, trial_id (FK ON DELETE CASCADE)
award_type award_type NOT NULL
winning_entry_id UUID REFERENCES entries(id) ON DELETE SET NULL  (FK added in entries migration)
winning_armband INT (CHECK >= 1)
winning_score NUMERIC(5, 1) (CHECK >= 0)
contributing_entry_line_ids UUID[] NOT NULL DEFAULT '{}'  (GIN index)
notes TEXT
created_at, updated_at, deleted_at, created_by, updated_by
```

Partial unique on (trial_id, award_type) WHERE deleted_at IS NULL.

**Absent (PR 2d candidates):** `combined_award_groups` reference table
(Q4 widened scope; not yet built); any HTQ extension if rhtq enum
expansion is needed.

## A6. `submission_records` column shape (CONFIRMED)

From `20260424120500_create_submission_records.up.sql`:

```
id UUID PK
club_id UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE
trial_id UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE
submission_type submission_type NOT NULL  (pdf_package, xml, csv)
marked_catalog_object_key TEXT
form_jovry8_object_key TEXT
xml_payload_object_key TEXT
akc_destination_email TEXT NOT NULL
fee_total NUMERIC(10, 2) NOT NULL CHECK (>= 0)
submitted_at TIMESTAMPTZ
submitted_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL
status submission_status NOT NULL DEFAULT 'draft'  (draft, generated, submitted, accepted, rejected)
akc_response JSONB
rejection_reason TEXT
created_at, updated_at, deleted_at, created_by, updated_by
```

The migration explicitly does NOT include `judges_book_object_keys`
(comment lines 14-16): "the judges book that AKC receives is the
PHYSICAL original with the judge's wet signature, NOT a PDF... The
earlier draft of DATA_MODEL.md §9 listed judges_book_object_keys on
this table; that column is intentionally absent here."

This matches the 2026-04-24 Decisions-log entry "submission_records
scope is electronic submission only" (line 390).

For B3 (signed-scan handling): the submission_records table by design
does not carry the signed-judges-book scan. PR 2d's three options for
signed-scan handling (per Known-gaps lines 905-907) all need to land
the artifact somewhere ELSE - either an extension of submission_records
with a signed_scan_object_key column, a parallel artifact table, or
attaching to trials directly. None of those columns exist today.

## A7. RLS policies on PR 2d tables (CONFIRMED)

All seven event-subtree tables (`events`, `event_days`, `trials`,
`trial_class_offerings`, `judges`, `judge_assignments`, `trial_awards`)
have RLS enabled with the standard symmetric `*_tenant` policy on
`club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid`
for both USING and WITH CHECK
(`20260419140500_enable_rls_on_event_setup.up.sql:34-75`).

`clubs` has the special asymmetric policy from
`20260419140000_tighten_clubs_with_check.up.sql:44-58`: USING is
`id = current_club_id OR id IN (SELECT club_id FROM user_club_roles
WHERE user_id = current_user_id AND active)`, WITH CHECK is
`id = current_club_id` only.

`canonical_classes` has permissive `read_all` RLS plus SELECT-only grant
to `qtrial_tenant` (`20260419130300_enable_rls_and_grants.up.sql:51`).

`submission_records` has standard symmetric `*_tenant` policy
(`20260424121000_enable_rls_on_tenant_scoped_gap_fill.up.sql:66`).

No drift. Any new tables PR 2d adds must follow the same symmetric
shape (or, if breed/group reference tables are added per A3 option (b),
the permissive-read shape for reference data).

## A8. `canonical_classes` shape (CONFIRMED)

From `20260419120500_create_canonical_classes.up.sql`:

```
id, registry_id (FK), sport sport NOT NULL, code TEXT NOT NULL
display_name TEXT NOT NULL
class_type canonical_class_type NOT NULL  (regular, optional_titling, preferred, nonregular)
legacy_class_code INT
is_sanctioned BOOL NOT NULL DEFAULT TRUE
has_jumps, has_broad_jump, has_multiple_entries_per_dog BOOL
total_score, min_qualifying_score, dogs_per_hour_default INT
has_walkthrough BOOL NOT NULL DEFAULT FALSE
default_walkthrough_minutes NUMERIC(4, 1)
qualifies_for_title_code TEXT
legs_required_for_title INT
title_earning_rule JSONB  (intentionally undesigned, NULL today)
ab_eligibility_rule ab_eligibility_rule NOT NULL DEFAULT 'none'
ab_eligibility_title_code TEXT
parent_class_id UUID REFERENCES canonical_classes(id)  (self-FK; NULL in modern data)
created_at, updated_at TIMESTAMPTZ
```

Natural unique key `(registry_id, sport, code)`.

For B1 (whichever PR 2d B-item touches canonical_classes), no
extension columns are missing that are in scope for PR 2d. The
`title_earning_rule JSONB` is the most likely future-extension point
but is explicitly deferred per the migration comment.

---

## Summary of Phase A findings

| Item | Status |
|---|---|
| A1 baseline | OK, with date drift to flag |
| A2 entry-subtree reconciliation | OK |
| A3 mixed_breeds_allowed scope-lock | **DRIFT - lock not found in repo; B6 blocked** |
| A4 ENUM state | OK; `entry_status` is a prompt-side phantom |
| A5 column listings | OK |
| A6 submission_records | OK |
| A7 RLS | OK |
| A8 canonical_classes | OK |

**Phase A is otherwise clean.** Only A3 changes Phase B materially.
Per the kickoff's stop rule, Phase B is paused.

## Asks for Robare

1. **A3 resolution.** Pick (a) "BOOL only, defer breed-list" (which is
   what the kickoff implied) and let me write the missing
   Decisions-log entry as part of PR 2d, OR (b) "BOOL plus breed-list
   together" (which is what every written source actually says) and
   I'll expand B6 to cover the full breed-restriction shape, OR (c)
   point me at the lock if it lives somewhere I didn't search.

2. **A4 prompt error.** Confirm I should drop any Phase B reasoning
   that depends on a non-existent `entries.status` / `entry_status`
   ENUM. Status is `entry_line_status` only.

3. **Date convention going forward.** The repo has 2026-04-23 (system
   clock), 2026-04-25 (HEAD commit), and 2026-04-26 (DATA_MODEL,
   newest Decisions entries) all coexisting. PR 2d's Decisions-log
   entries should pick one; my preference is "use the actual commit
   date at the time the entry is written" but I'll match whatever
   convention you set.

Once I have direction on (1) at minimum, I'll proceed to Phase B
(B1-B7) and then Phase C (design note).
