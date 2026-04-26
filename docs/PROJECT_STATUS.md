# QTrial Project Status

**Last updated:** 2026-04-26
**Current phase:** Phase 0 + PR 2a + PR 2b + PR 2c-surgery + PR 2c-beta + PR 2d complete
**Maintained by:** Robare Pruyn, with Claude assistance

---

## Where we are right now

PR 2d (events / clubs / awards plumbing) is complete. Main now has
62 migration pairs and 46 tables. The 9 PR 2d migrations land the
Deborah 2026-04-23 plumbing in three squash commits:

- #18 schema execution: events.mixed_breeds_allowed BOOL,
  events.trial_chair_user_id and events.event_secretary_user_id,
  events.dogs_per_hour_override JSONB, armband_scheme.per_series
  ENUM extension, clubs.officers_json JSONB,
  trial_class_offerings.pre_trial_blank_pdf_object_key and
  signed_scan_pdf_object_key (two-column shape, revising the
  2026-04-24 working assumption), combined_award_groups + the
  combined_award_group_classes junction with permissive-read RLS,
  trials.trial_chairperson dropped (replaced by the event-level
  FK). DATA_MODEL.md bumped to v0.4. AKC Rally and Obedience
  regulation PDFs committed under db/seed/akc/regulations/ as
  frozen citation sources.
- #19 seed loader: 5 combined_award_groups rows (Obedience HC plus
  Rally RHC, RHTQ, RAE, RACH) and 12 junction rows. Two-pass
  loader with sport-mismatch validation that rejects bad rows
  before any insert. 5 new integration tests in
  workers/tests/seed_loader.rs.
- #20 spec docs and decisions: this checkpoint. Three new
  Decisions-log entries (mixed_breeds scope-lock, judges-book
  two-column shape, rhtq phantom catch). Known-gaps PR-2d block
  retired. REQUIREMENTS / WORKFLOWS / DOMAIN_GLOSSARY / ROADMAP /
  db/migrations/README.md updated. RLS tests for the two new
  reference tables in shared/tests/.

A 2026-04-25 investigation against the committed Rally Regulations
PDF confirmed that "Master + Choice" is NOT an AKC-recognized
combined award; that wording on the GFKC June 2026 premium list is
a club-side fee discount, not a rulebook path. The
combined_award_groups seed reflects only the AKC paths.

`git grep -i offleash` returns zero hits outside this file. The
references that remain in `docs/PROJECT_STATUS.md` (the rename
Decisions-log entries and adjacent narrative) are intentional
historical record; the verification claim should be read as
"outside `docs/PROJECT_STATUS.md`."

Separately, QTrial LLC formation is in flight via Northwest
Registered Agent (NY domestic, Warren County principal office,
$244 filed). Awaiting filing confirmation, then EIN, Operating
Agreement, Warren County publication, Relay banking, and AWS
account setup.

---

## Recently completed

### PR 2d: events / clubs / awards plumbing (2026-04-26)

- 2026-04-26: PR #20 (pending) - feat/pr-2d-spec-docs-and-decisions -
  spec-doc closeout for PR 2d. Three Decisions-log entries
  (events.mixed_breeds_allowed BOOL-only scope-lock; judges-book
  two-column shape revising the 2026-04-24 working assumption;
  rhtq-phantom catch from CHECKPOINT 0 Phase A). Known-gaps PR-2d
  block retired (entire block done, including the phantom rhtq
  bullet). REQUIREMENTS.md gains REQ-EVENT-001 / REQ-CLUB-001 /
  REQ-AWARD-001 / REQ-AWARD-002 plus REQ-SUB-006 / REQ-SUB-007 /
  REQ-INCIDENT-001 lines and the breed-restriction line at §2.1
  rewrites to the BOOL-only-defer-list scope. WORKFLOWS.md §10.4
  variable list extended with chair / secretary contact vars; new
  §9.7 stub for the AEDSQ1 72-hour incident-reporting path
  (placed under §9 Post-trial AKC submission rather than as a new
  top-level section, since AEDSQ1 is itself an AKC submission to a
  different mailbox). DOMAIN_GLOSSARY
  refines Trial Chair / Trial Secretary; refines HC / RHC / HTQ
  (HTQ corrected from "Honor Team Qualifier" to "Highest Triple
  Qualifying" per Rally Reg. Ch. 1 §32); adds RAE / RACH /
  Combined Award / All-American Dog. ROADMAP.md gains a Phase 2+
  breed-list line and a Phase 7+ cross-club-dog-identity line.
  db/migrations/README.md gains a PR-block list. New RLS test
  file shared/tests/combined_award_rls.rs (4 tests covering
  permissive-read SELECT and tenant-INSERT/UPDATE/DELETE denial
  on both reference tables).
- 2026-04-26: PR #19 (commit e0a24ca) - feat/pr-2d-combined-award-groups-seed -
  seed CSVs and qtrial-seed-loader extension for the combined-
  award reference tables. db/seed/akc/combined_award_groups.csv
  has 5 rows (akc_obedience_hc, akc_rally_rhc, akc_rally_rhtq,
  akc_rally_rae, akc_rally_rach with verbatim AKC citations from
  the regulation PDFs committed in #18).
  db/seed/akc/combined_award_group_classes.csv has 12 rows with
  is_required_for_award TRUE on every row. Loader extension is
  two-pass: pass 1 validates every junction row (group code
  resolves, canonical class code resolves, sports match) before
  any insert; pass 2 inserts in a single transaction. Sport
  mismatch fails the load with a clear error. 5 new integration
  tests in workers/tests/seed_loader.rs covering load count,
  idempotency, sport-mismatch rejection, and is_required_for_award
  fidelity.
- 2026-04-26: PR #18 (commit 55c9cbd) - feat/pr-2d-events-clubs-awards -
  PR 2d schema execution. Nine new migrations (62 total, up from
  53) land Deborah's 2026-04-23 plumbing on events / clubs /
  trial_class_offerings, plus the combined_award_groups parent +
  junction reference tables with permissive-read RLS. Adds
  events.mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE,
  events.trial_chair_user_id / event_secretary_user_id (FK to
  users, ON DELETE SET NULL, both nullable),
  events.dogs_per_hour_override JSONB, armband_scheme.per_series
  ENUM value (with IF NOT EXISTS for round-trip safety),
  clubs.officers_json JSONB,
  trial_class_offerings.pre_trial_blank_pdf_object_key plus
  signed_scan_pdf_object_key (two-column shape; the 2026-04-24
  single-column working assumption is revised). Drops
  trials.trial_chairperson (no consumers; replaced by the
  event-level FK). AKC Rally Regulations (edition 1217) and
  Obedience Regulations (2025-03 amended) PDFs committed under
  db/seed/akc/regulations/ as frozen citation sources for the
  CHECKPOINT 2 seed CSVs. DATA_MODEL.md bumped to v0.4 with §1
  clubs / §2 events / §2 trials / §2 trial_class_offerings / §8
  combined-award-group entries reconciled.

### PR 2c-beta: dogs table reconciliation (2026-04-26)

- 2026-04-26: PR (pending) - feat/dogs-reconciliation -
  reconciles dogs against DATA_MODEL.md §4. Five migrations (53
  total, up from 48): three add-column / constraint
  (registration_type ENUM + column; parsed_name_root +
  parsed_prefix_titles + parsed_suffix_titles +
  unparsed_title_tokens; breed_id and breed_variety_id FK
  constraints deferred from Phase 0), two drop-column
  (co_owners_text superseded by dog_ownerships from PR 2b; four
  sire/dam prefix-suffix columns collapsed so sire and dam names
  are parsed at render time against the same title catalog used
  for the dog's own name). No test changes required
  (registration_type nullable; all four dog-INSERT sites
  unaffected by the spec reconciliation). DATA_MODEL.md §4
  reconciled against migration-authoritative extras
  (jump_height_measured + has_jump_height_card + the
  dogs_jump_height_nonneg CHECK; is_akc_ineligible +
  akc_ineligible_reason + akc_ineligible_recorded_at + the
  dogs_ineligible_has_recorded_at CHECK). DATA_MODEL.md metadata
  bumped to v0.3 / 2026-04-26.

### PR 2c-surgery: entry-pipeline reconciliation (2026-04-25)

- 2026-04-25: PR (pending) - feat/entry-pipeline-surgery -
  reconciles entries, entry_lines, entry_line_results, and the
  dog_title_source enum against DATA_MODEL.md §5. Six migrations
  (48 total, up from 42): two add-column (entry_lines handler +
  armband columns; entry_line_results timing + rach_points), three
  drop-column (entries.armband, entries handler columns,
  entry_lines.jump_height_inches), one enum extension
  (dog_title_source adds parsed_from_registered_name). Handler
  identity moves from entries to entry_lines; armband routes
  through armband_assignments (PR 2b) via
  entry_lines.armband_assignment_id; jump height lives on
  dog_trial_jump_heights (PR 2b) keyed by (dog, trial).
  entry_layer_rls.rs seed helper rewritten to match the new shape;
  armband_is_unique_among_live_entries_in_an_event replaced by
  armband_is_unique_within_series_and_trial_on_armband_assignments
  covering both armband_assignments UNIQUE constraints.
  tenant_fk_validation.rs seed fixture gains one-line
  handler_contact_id binding. DATA_MODEL.md §5 reconciled against
  migration-authoritative extras (transfer_intent columns +
  coupled CHECK on entry_lines; judge_annotation_text on
  entry_line_results). Dogs table reconciliation deferred to
  PR 2c-beta; events/clubs/offerings/awards additive work deferred
  to PR 2d.

### PR 2b: tenant-scoped table gap-fill (2026-04-24)

- 2026-04-24: PR (pending) - feat/tenant-scoped-gap-fill -
  nine new tables lifting the schema from 35 to 44 tables and
  from 33 to 42 migrations. Eight tenant-scoped tables
  (dog_ownerships, dog_trial_jump_heights, armband_assignments,
  email_templates, submission_records, payments, refunds,
  audit_log) with direct-club_id RLS matching the Phase 0
  pattern; platform_admins deliberately non-tenant with no
  qtrial_tenant grant. Two new ENUMs (submission_type,
  submission_status) in the submission_records migration plus
  refund_reason in the payments-and-refunds migration;
  payment_method reused from the entries migration.
  shared/src/fk_validation.rs::TenantTable gains Payment and
  ArmbandAssignment variants (closed-by-default policy established;
  other PR 2b tables deferred until a concrete FK target lands).
  New test binaries: shared/tests/tenant_scoped_gap_fill_rls.rs
  (7 tests) and 4 new cases in tenant_fk_validation.rs (14 total).
  DATA_MODEL.md §9 submission_records updated to drop
  judges_book_object_keys per Deborah's 2026-04-23 correction; §2
  trial_class_offerings gains a pending judges_book_pdf_object_key
  column (lands in PR 2c); §4 dog_trial_jump_heights verified
  in-place (the earlier NUMERIC(4,1) migration was corrected to
  INT per the elected-bucket semantics). CHECKPOINT 4
  investigation caught the NUMERIC vs INT drift and the
  judges_book artifact-location question before merge.

### AKC artifact organization and 2026-04-23 Q&A note (2026-04-23)

- 2026-04-23: PR (pending) - chore/akc-artifacts-and-research-2026-04-23 -
  renamed and organized 11 AKC reference PDFs under
  `db/seed/akc/akc_forms/` (8 blank AKC forms) and
  `db/seed/akc/sample_artifacts/` (3 filled GFKC artifacts),
  added README files for both directories explaining the naming
  convention and what each file is, renamed the two email-
  transcript PDFs under `docs/research/attachments/`, and wrote
  `docs/research/2026-04-23-deborah-round-2-answers.md` distilling
  Deborah's annotated answers to Robare's eight follow-up
  questions. Five design-doc references to the old
  `Judges_Book_Cover_Sat.pdf` filename updated to the new
  `gfkc_rally_judges_book_cover_2025_11_15_sat.pdf`; the one
  historical reference in the 2026-04-19 research note is left
  frozen in time. AEDSQ1 filename correction: Robare's placement
  filename encoded `0615` but the form is actually the
  Disqualification for Attacking a Person form revision 11/19;
  corrected on rename.

### PR 2a: reference-data foundation (2026-04-23)

- 2026-04-23: PR #12 (pending) - feat/reference-data-foundation -
  14 new reference tables (countries, states, breed_groups, breeds,
  breed_varieties, title_prefixes, title_suffixes, jump_heights,
  obedience_exercises, obedience_class_exercises, otch_points,
  om_points, rally_rach_points, sport_time_defaults) with
  permissive-read RLS; qtrial-seed-loader binary populating them
  idempotently from db/seed/akc/ CSVs; 5/5 seed-loader integration
  tests; GitHub Actions CI workflow with Postgres 16 service
  container; db/migrations/README.md; DATA_MODEL.md §8 updated
  with akc_fee_schedules (the one Phase 0 table the doc had drifted
  past). 256 junction rows loaded into obedience_class_exercises
  from 22 of 36 CSV rows; 14 rows skipped by design (4 Random-
  Reward base layouts, 10 unseeded canonical classes). Seed
  directory moved from the local-only db_seed_akc/ to
  db/seed/akc/ as the first commit on the branch.

### This merge-cleanup cycle (2026-04-21 / 2026-04-22)

- 2026-04-22: PR #10 - phase-0-tenant-fk-validation - cross-tenant
  FK validation helper (shared/src/fk_validation.rs) with
  TenantTable enum and verify_fk_targets_in_tenant. 10/10
  integration tests covering happy path, nonexistent-UUID rejection
  without information leak, cross-tenant rejection, user role
  scoping, batch validation.
- 2026-04-22: PR #9 - docs(claude): incorporate PROJECT_STATUS.md
  into CLAUDE.md reading list and maintenance guidance
- 2026-04-22: PR #8 - docs: fix "an QTrial" to "a QTrial" in judges
  table note and Access migration doc. Grammar fix inherited from
  the OffLeash era (where "an OffLeash" was correct for vowel-
  sound).
- 2026-04-22: PR #7 - phase-0-entries - entry layer tables (owners,
  dogs, dog_titles, dog_sport_participation, teams, entries,
  entry_lines, entry_line_results) with RLS, state machine, partial
  unique index on (event_id, armband), 8/8 entry_layer_rls tests
- 2026-04-22: PR #6 - phase-0-event-setup - events, event_days,
  trials, trial_class_offerings, judges, judge_assignments,
  trial_awards with RLS and parent_club_id helper
- 2026-04-22: PR #5 - docs/rename-research-notes - cleaned up the
  last 16 OffLeash references in docs/research/ (inherited from
  the docs PR before the rename caught up)
- 2026-04-22: PR #4 - phase-0-tenancy - clubs, users,
  user_club_roles with RLS; qtrial_tenant role; testcontainers
  fixture and RLS integration tests
- 2026-04-22: PR #3 - phase-0-reference-schema - registries,
  akc_fee_schedules (with 2025 and 2026 AKC rates from JOVOB7/
  JOVRY8), canonical_classes (75 rows Obedience + Rally), sqlx
  migration tooling and up/down pair convention
- 2026-04-21: PR #2 - docs/deborah-qa-integration - integrated
  Deborah's Q&A findings from 2026-04-19/2026-04-20 sessions
  across nine docs; added docs/research/ with provenance

### Prior work

- 2026-04-21: PR #1 - refactor: rename OffLeash to QTrial -
  renamed Rust packages, Postgres roles, env vars, scripts,
  docs across 38 files
- Initial repo scaffold with directory structure
  (api/, workers/, shared/, web/, db/, infra/, docs/)

---

## In flight

Nothing in flight. PR 2d is on its third (and final) squash-merge;
once merged, Phase 0 plus PR 2a / 2b / 2c-surgery / 2c-beta / 2d
is complete. Phase 1 (per ROADMAP.md) is the next major milestone:
club creation and configuration, user management, event creation,
trial class offerings, judge directory, fee configuration, basic
premium-list PDF generation.

---

## Blocked / waiting on external

- **USPTO TESS trademark clearance for "QTrial"** - search not
  yet run. Domain is registered; trademark status unverified.
- **NY LLC formation filing confirmation** - submitted via
  Northwest, awaiting confirmation
- **EIN** - DIY at irs.gov (10 minutes), blocks Relay banking
  setup
- **Warren County publication** - within 120 days of formation;
  Post-Star daily + Chronicle weekly; contact 518-761-6427
- **Operating Agreement** - NY attorney engagement pending
  (80/20 + NY marital carve-out, estimated $500-1500)
- **Relay Banking account** - blocked on EIN
- **AWS account (aws@qtrial.app)** - pending
- **Deborah Q&A round 2** - seven open questions need her
  availability (see "Known gaps" below)

---

## Decisions log

Architecture and process decisions made during planning and build,
with rationale. This section prevents re-litigating settled
questions.

### 2026-04-26: PR 2d - events.mixed_breeds_allowed ships as BOOL only; breed-list model deferred

**Decision:** PR 2d adds `events.mixed_breeds_allowed BOOL NOT NULL
DEFAULT TRUE`. The breed-list approach (junction tables associating
events with allowed breeds, breed_groups, or breed_varieties) is
deferred to a future PR. The two pieces are structurally separate
work: the BOOL handles the All-American Dog exclusion case
(predominantly conformation, post-MVP for QTrial); the list model
handles Specialty single-breed restrictions and breed-group filters.

**Rationale:**

1. PR 2d scope is already non-trivial. Adding a breed-list design
   adds another migration-ordering question plus three or four
   sub-questions plus a new junction table.
2. The mixed-breeds case Deborah called out in Q3 (conformation
   excluding mixed) is structurally the All-American Dog flag path.
   The breed-restricted-event case (a Specialty for one breed) is
   structurally separate work and warrants its own design pass.
3. The flag-only path is fully additive. The breed-list junction
   can land in a later PR without touching events again.
4. We have no real artifact to design breed-list against. GFKC
   June 2026 Rally has no breed restrictions. Designing on
   speculation produces a worse model than waiting until a
   Specialty or Group show artifact is in hand.

**Supersession note:** This decision supersedes the framing in the
2026-04-23 round-2 research note ("alongside the breed-list
approach", at
`docs/research/2026-04-23-deborah-round-2-answers.md` lines 49-57)
and the corresponding bullet in `docs/REQUIREMENTS.md` §2.1. The
research note stays unedited as historical evidence; REQUIREMENTS.md
and the Known-gaps block in this document are updated in the same
PR that lands this decision.

**Evidence:**
`docs/research/2026-04-25-pr-2d-checkpoint-0-design-note.md` §B6;
PR 2d migration
`db/migrations/20260426120500_add_events_mixed_breeds_allowed.up.sql`.

### 2026-04-26: PR 2d - judges-book PDF storage uses two columns, not one overwriting column

**Decision:** `trial_class_offerings` carries
`pre_trial_blank_pdf_object_key TEXT` and
`signed_scan_pdf_object_key TEXT` as two distinct nullable columns
rather than a single `judges_book_pdf_object_key` overwritten at
scan time. The pre-trial blank PDF (REQ-SUB-002) and the post-trial
signed scan are different artifacts at different lifecycle stages
and live in separate columns.

**Rationale:**

1. Honest about the state machine. The pre-trial blank is generated
   by QTrial for printing and signing; the signed scan is uploaded
   back into QTrial after the trial as the durable record of what
   was mailed to AKC. Different artifacts, different timestamps,
   different audit roles.
2. Re-render of the pre-trial blank when a judge changes late in
   the pre-trial cycle does not clobber a previously-uploaded
   signed scan. With one overwriting column, regenerating after a
   scan upload would either silently overwrite the scan or require
   an "is this a scan?" check before rendering.
3. Audit trail. The secretary can verify after the fact which
   artifact was actually mailed to AKC, supporting REQ-SUB-002 and
   REQ-SUB-004 audit paths. Per the 2026-04-24 Decisions-log entry
   "submission_records scope is electronic submission only", the
   signed-mail artifact intentionally does NOT live on
   `submission_records`; this column pair is its durable home.
4. The 2026-04-24 working assumption (overwrite the same column at
   scan time) was a working assumption, not a lock. CHECKPOINT 0
   Phase B revisited the question now that PR 2d is the PR landing
   the column; this entry locks the revised shape.

**Evidence:**
`docs/research/2026-04-25-pr-2d-checkpoint-0-design-note.md` §B3;
PR 2d migration
`db/migrations/20260426121100_add_trial_class_offerings_judges_book_columns.up.sql`.

**Supersession note:** Replaces the 2026-04-24 working assumption
("overwrite the same column at scan time") that was tracked as an
OPEN QUESTION in this document's Known-gaps PR-2d block. The
Known-gaps bullet is removed in the same PR that lands this entry.

### 2026-04-26: PR 2d - rhtq already in award_type ENUM since Phase 0; Known-gaps drift caught during CHECKPOINT 0

**Decision:** No ALTER TYPE migration is needed for
`trial_awards.award_type` to add `rhtq`. The value has been a
member of the ENUM since Phase 0 (migration
`20260419140400_create_judge_assignments_and_awards.up.sql` line 52
defines the type as
`('hit', 'hc', 'phit', 'phc', 'rhit', 'rhc', 'rhtq', 'htq')`). The
"add rhtq to ENUM" bullet that appeared in the PR 2d Known-gaps
block was a phantom - the gap-list drifted from reality at some
point between Phase 0 and PR 2d scoping.

**Rationale:**

1. Verify before adding. The Known-gaps list is an aspirational
   forward-looking list of pending work, but it can drift from
   reality if items are silently completed or were never gaps to
   begin with. Future audits should re-verify gap-list items
   against the actual schema rather than trust the list at face
   value.
2. The CHECKPOINT 0 verification phase exists for exactly this
   reason. Documenting the value the verification phase produced
   keeps the pattern in front of future readers: cheap reads of
   the actual state catch expensive drift in the docs.
3. Migration list shorter as a result. The PR 2d kickoff implied
   two ALTER TYPE migrations (armband_scheme.per_series and
   award_type.rhtq); only the first is real. One less ENUM
   ordering interlock to think about.

**Evidence:**
`docs/research/2026-04-25-pr-2d-checkpoint-0-state-verification.md`
A4 finding ("rhtq is ALREADY in the `award_type` ENUM as of Phase
0"); Phase 0 migration
`db/migrations/20260419140400_create_judge_assignments_and_awards.up.sql`
line 52.

### 2026-04-26: Sire and dam names are two columns, not six

**Decision:** dogs.sire_registered_name and dogs.dam_registered_name
store each parent's full registered-name string verbatim. The
Phase 0 six-column shape (sire_prefix_titles / sire_registered_name
/ sire_suffix_titles + dam equivalents) is collapsed to two columns
in PR 2c-beta. Titles within sire/dam strings are parsed at display
time against the same title catalog used for the dog's own name.

**Rationale:** The dog's OWN registered name is stored in one
column (registered_name) with companion parsed_* arrays populated
by the name parser. Storing sire and dam with a different shape
was philosophical inconsistency - either the parser handles all
three or it handles none. Direction 2 keeps all three on the
parse-at-display-time path. The render-time parsing cost is
per-catalog-page, amortized, and negligible compared to a PDF
generation pass.

**Evidence:** PR 2c-beta migration
20260426120400_drop_dogs_sire_and_dam_title_columns; DATA_MODEL.md
§4 update in this PR.

### 2026-04-26: dogs.registration_type is nullable (NULL means "unknown")

**Decision:** dogs.registration_type is an ENUM
(akc_purebred, pal, canine_partners, fss, misc) with no NOT NULL
constraint and no DEFAULT. NULL means "type not yet known."

**Rationale:** NOT NULL with DEFAULT 'akc_purebred' pretends every
dog is purebred-by-default; an Access import from tblDogData may
produce rows where the type is genuinely unrecoverable from the
available fields, and defaulting such rows to akc_purebred invents
truth. NULL is honest. App layer handles "registration_type is
NULL" as a first-class state (UI shows "Unknown" plus a correction
affordance); the data-quality review path can sweep NULL rows for
manual classification. New dogs entered via the UI are NOT NULL at
the form layer; the schema permits NULL specifically for the
import path.

**Evidence:** PR 2c-beta migration
20260426120000_add_dogs_registration_type; DATA_MODEL.md §4 update
in this PR; DATA_MODEL.md §11 Access migration mapping.

### 2026-04-26: dog_titles.title_code stays free text; FK conversion deferred

**Decision:** dog_titles.title_code remains TEXT. The "convert to
FK in the next session" comment on the Phase 0 migration is
superseded by this decision. The title_prefixes and title_suffixes
catalogs from PR 2a serve as soft-reference lookups (via
title_prefixes.code + title_suffixes.code string joins at query
time) rather than FK targets.

**Rationale:** Numeric-variant titles (RAE2, RAE3, MACH 2, MACH 3)
are distinct title_codes, enforced by a partial unique index on
(dog_id, title_code). Converting title_code to an FK requires
either populating the catalogs with every numeric variant
(explodes catalog maintenance) or splitting title_code into
base_code + instance_number (structural change to both dog_titles
and the title_prefixes/suffixes tables). Neither is
"reconciliation" scope, and neither is the right move for MVP.
The free-text column with a soft lookup fits the existing shape.
Revisit when title-progression automation (REQ-NAME-001 +
title_prefixes.earning_rules) has concrete requirements.

**Evidence:** Phase 0 migration
20260420120200_create_dogs_titles_and_participation.up.sql lines
116-119 (stale comment to be understood as superseded; the
migration itself is not edited in PR 2c-beta).

### 2026-04-25: Entry-pipeline identity routing (handler, armband, jump height)

**Decision:** Handler identity, armband, and jump height are each
modeled on the row that carries them correctly:

- Handler identity lives on entry_lines (handler_contact_id +
  junior_handler_akc_number). Not on entries, because a dog running
  multiple classes at the same event may have different handlers
  per class.
- Armband routes through armband_assignments (PR 2b) via
  entry_lines.armband_assignment_id. Not as a raw INT on entries,
  because the per-series modeling supports Obedience's 500-series
  convention where Advanced B, Excellent B, and Master dogs share
  armbands within a series.
- Jump height lives on dog_trial_jump_heights (PR 2b) keyed by
  (dog, trial). Not on entry_lines, because per Deborah's
  2026-04-20 Q1 jump height never changes between classes on the
  same day for the same dog; the rare in-ring judge override must
  update all of the dog's remaining entries at the trial.

**Rationale:** The Phase 0 migrations (PR #7) predate Deborah's
2026-04-20 Q&A and landed these three concerns on the wrong rows.
PR 2c-surgery physically realizes the DATA_MODEL.md §5 shape that
the Q&A already locked in.

**Evidence:** PR 2c-surgery migrations
20260425120200_drop_entries_armband,
20260425120300_drop_entries_handler_columns,
20260425120400_drop_entry_lines_jump_height,
20260425120000_add_entry_lines_handler_and_armband_columns;
DATA_MODEL.md §5.

### 2026-04-25: Postgres ENUM additions are one-way; down migrations are no-ops

**Decision:** When a migration adds a value to a Postgres ENUM type,
the matching down migration is a no-op with an explanatory comment.
Postgres provides no DROP VALUE or equivalent mechanism.

**Rationale:** The full workaround (drop type, recreate without the
value, alter every dependent column to TEXT and back) is heavy,
risks breaking app code that relies on the ENUM's typed form, and
is almost never what a rollback actually wants. A no-op down with
a clear comment is the honest shape and preserves the round-trip
contract for all other migrations in the same PR.

**Application:** PR 2c-surgery's dog_title_source enum extension
adds parsed_from_registered_name. The down migration documents the
one-way behavior. Future ENUM additions follow the same pattern.

**Evidence:** 20260425120500_extend_dog_title_source_parsed_from_registered_name.down.sql.

### 2026-04-25: Migration-authoritative columns folded into DATA_MODEL

**Policy:** When a Phase 0 migration carries columns or constraints
not present in DATA_MODEL.md, the first move is to evaluate the
migration's rationale (usually captured in the file's header
comment). If the rationale holds up, the migration is authoritative
and DATA_MODEL catches up. If the rationale doesn't hold, the
migration loses the extra and the spec stays.

**Application to PR 2c-surgery:**
entry_lines.transfer_intent_target_class_id,
entry_lines.transfer_intent_trigger_title_code, the
entry_lines_transfer_intent_coupled CHECK, and
entry_line_results.judge_annotation_text all had migration-header
rationale that held up on re-read. All four fold into DATA_MODEL.md
§5. No migration changes.

**Rationale:** Phase 0 migrations were written before the
DATA_MODEL was refined via Deborah's Q&A. Some columns that
landed in migrations are legitimate additions the spec should
reflect; others are drift. The policy distinguishes.

**Evidence:** DATA_MODEL.md §5 additions in this PR; migration
header comments on
20260420120500_create_entry_lines.up.sql lines 33-39 and
20260420120600_create_entry_line_results.up.sql lines 1-8.

### 2026-04-24: submission_records scope is electronic submission only

**Decision:** submission_records tracks the submission EVENT - the
marked catalog PDF and the populated AKC form (JOVRY8 / Obedience
equivalent) attached to the email to AKC. Per-class judges-book
PDF artifacts do not live on submission_records. Pre-trial blank
judges-book PDFs will live on trial_class_offerings (column added
in PR 2c). Post-trial signed-scan handling is explicitly deferred
to PR 2c scoping.

**Rationale:** The two concerns are different artifacts at different
times. submission_records is created at submission time and
represents the email to AKC. Judges books are per-class artifacts
generated before the trial and signed during it. Collapsing them
onto one row mixed the concerns and broke the Q2 "AKC requires
physical original with wet signature" constraint (the electronic
submission never carries signed books; only the physical mail does).

**Evidence:** db/migrations/20260424120500_create_submission_records.up.sql
header comment; this PR's DATA_MODEL.md §9 and §2 updates.

### 2026-04-24: dog_trial_jump_heights.jump_height_inches is INT

**Decision:** dog_trial_jump_heights.jump_height_inches uses INT,
not NUMERIC(4,1). CHECK enumerates the 15 AKC integer buckets.

**Rationale:** Elected jump height is an AKC-defined integer bucket
set per sport (Obedience: 4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26,
28, 30, 32, 34, 36; Rally: 4, 8, 12, 16). This is semantically
distinct from dogs.jump_height_measured NUMERIC(4,1), which is a
physical measurement at the withers and can be fractional. The two
columns share a name root but not a type or meaning. Modeling the
elected column as NUMERIC added storage cost, added semantic opacity,
and would have required rewriting the CHECK if the future ever added
half-inch buckets (which would then deserve their own design
conversation). INT plus enumerated CHECK is the honest shape.

**Evidence:** This PR's fix commit on
db/migrations/20260424120200_create_dog_trial_jump_heights.up.sql;
DOMAIN_GLOSSARY "Jump height" entry.

### 2026-04-24: TenantTable enum is closed by default

**Policy:** shared/src/fk_validation.rs::TenantTable admits a new
variant only when a concrete tenant-scoped table FKs the row in
question. New tenant tables do not automatically get variants;
the enum stays closed at its current set to prevent accidental
validation gaps, and new variants are added in the same PR that
introduces the FK pointing at the new target.

**Application to PR 2b:** Payment and ArmbandAssignment added
(Payment because refunds.payment_id is in PR 2b; ArmbandAssignment
because entry_lines.armband_assignment_id is in PR 2c). Deferred
with no variant (no current or PR-2c-scoped FK target): DogOwnership,
DogTrialJumpHeight, SubmissionRecord, AuditLog, EmailTemplate,
Refund. platform_admins is non-tenant and will never get a variant.

**Rationale:** Variants that exist "in case someone ever FKs to this
table" are dead weight and invite drift between the enum and the
real FK graph. Closed-by-default keeps the validation surface
honest.

**Evidence:** This PR's commit 2; shared/src/fk_validation.rs enum
definition post-PR-2b.

### 2026-04-23: Cross-tenant dog identity deferred to conformation work

**Decision:** QTrial does NOT maintain a shared cross-club dog
identity for MVP. Each club's dog directory is standalone for
the Obedience and Rally MVP. A cross-club dog model is required
for the post-MVP conformation work and remains on the long-term
roadmap.

**Rationale:** Per Deborah's Q8 answer (2026-04-23), trial
secretaries running Obedience and Rally treat dogs as unrelated
across clubs at the operational level; the data they need lives
in the entry form for that specific club's trial. The legitimate
cross-club continuity case she identified is cluster trials,
which are conformation-only today and run by superintendents
(two companies have a monopoly on that segment). Conformation
is a future QTrial release goal, and the cross-club dog model
is the right thing to build alongside that work, not before it.

For MVP, this means:
  - Each club's dog directory remains standalone (current
    Phase 0 model is correct)
  - The dog dedup-across-tenants concern in DATA_MODEL.md "Open
    questions / pending decisions" item 3 is deferred, not
    closed
  - When conformation work scopes up, the cross-club dog model
    is part of that scope (likely involving a shared
    registered_dogs table or similar, with attention to RLS
    implications)

**Evidence:** docs/research/2026-04-23-deborah-round-2-answers.md
Q8.

### 2026-04-23: Combined-award discount logic moves to MVP

**Decision:** The combined_award_groups reference table (or
equivalent modeling) moves from P2 to PR 2c MVP scope.

**Rationale:** Per Deborah's Q4 answer (2026-04-23), the
additional-entry discount applies to ANY double or triple Q
in B classes in one trial, including Open B + Utility B
(going for HC and eventually OTCH). The original premium-list
wording "Master + Choice, RAE & RACH Title entries" was
too narrow. Modeling the combined-award groupings as
reference data is the cleanest way to drive both the fee
discount logic and the future award computation.

**Evidence:** docs/research/2026-04-23-deborah-round-2-answers.md
Q4.

### 2026-04-23: PR 2a reference tables use permissive-read RLS

**Decision:** Every reference table added in PR 2a (countries,
states, breed_groups, breeds, breed_varieties, title_prefixes,
title_suffixes, jump_heights, obedience_exercises,
obedience_class_exercises, otch_points, om_points,
rally_rach_points, sport_time_defaults) enables RLS with a
permissive-read policy (`USING (TRUE)`) plus a SELECT-only grant
to `qtrial_tenant`. No INSERT/UPDATE/DELETE policy means writes
are implicitly denied; the `qtrial` table owner bypasses RLS for
admin paths and the seed loader.

**Rationale:** Matches the pattern already in
`20260419130300_enable_rls_and_grants.up.sql` for registries,
akc_fee_schedules, and canonical_classes. Keeps reference-data
access uniform across every cross-tenant table and makes
seed-loader bypass behavior consistent.

**Evidence:** `db/migrations/20260423120700_enable_rls_on_reference_data_foundation.up.sql`
batches the grants, ENABLE, and policies for all 14 tables.

### 2026-04-23: obedience_class_exercises is normalized, not wide Box1-Box13

**Decision:** Each row in `obedience_class_exercises` represents
one (canonical_class, pattern_variant, box_position) cell. The
junction carries either `obedience_exercise_id` with `max_points`
(matched scored cell), or `max_points` + `box_label` (unmatched
scored cell: compound or sub-numbered exercise name), or
`box_label` alone (rollup/header row). Two CHECK constraints
enforce that at least one column is set and that a linked
exercise row also carries max_points.

**Rationale:** The seed CSV ships exercises in a wide
Box1..Box13 layout per class, but a wide schema wastes columns
(Novice has 12 boxes, Utility has 13, Beginner Novice has 11),
forces per-class special-casing at render time, and cannot
represent Open B / Utility B's six randomized pattern variants
without schema gymnastics. A normalized junction iterates cleanly
in the judges-book generator.

**Evidence:** `db/migrations/20260423120400_create_obedience_exercises.up.sql`.
Loader parses 22 of 36 CSV rows into 256 junction rows.

### 2026-04-23: pattern_variant column on obedience_class_exercises

**Decision:** `obedience_class_exercises` carries a
`pattern_variant INT NOT NULL DEFAULT 1` column with the UNIQUE
index on `(canonical_class_id, pattern_variant, display_order)`.
Open B I-VI and Utility B I-VI load as variants 1-6 of the
"Open B" / "Utility B" base canonical classes.

**Rationale:** AKC's random-reward Obedience patterns assign
exercises in six predetermined orderings. Each ordering is a
distinct judges-book layout for the same canonical class. Without
`pattern_variant`, the six variants would need to be six separate
canonical_classes rows, which either duplicates the base class or
forces a self-FK scheme the judges-book generator would have to
unwind. The variant column lets the canonical class stay as
"Open B" and the variant be metadata on the layout.

**Evidence:** Schema is in migration 20260423120400; seed CSV
rows 901-906 / 907-912 carry the six Open B / Utility B variants.

### 2026-04-23: Seed loader assumes a single concurrent runner

**Decision:** The `qtrial-seed-loader` binary assumes only one
instance runs against a given database at a time. The
`countries` loader (and similar tables) declare an
`ON CONFLICT (alpha2_code)` target even though `countries` has a
second unique constraint on `alpha3_code`; concurrent seed runs
could collide on alpha3_code first and raise a bare unique
violation rather than falling through to DO UPDATE.

**Rationale:** Not a concern in practice. The loader is a
one-shot post-migration batch job, not a user-facing reference-
data refresh tool. Broadening the ON CONFLICT clause to cover
multiple unique keys is not supported by Postgres; the real fix
would be a wrapping advisory lock, which is overkill for the
batch-job use case.

**Evidence:** Integration tests serialize the loader behind a
`tokio::sync::OnceCell<()>` gate
(`workers/tests/seed_loader.rs`). Revisit the decision if we
ever build user-facing reference-data refresh tooling.

### 2026-04-23: Preferred Open / Preferred Utility exercise patterns deferred

**Decision:** MVP ships `obedience_class_exercises` without
judges-book support for Preferred Open and Preferred Utility. The
current seed CSV has only `#1-#N` placeholder cells for both
classes, which the loader skips.

**Rationale:** Both classes have published AKC exercise lists
(https://www.akc.org/sports/obedience/getting-started/classes/)
but the seed CSV does not yet carry them. Adding the lists
requires extending `obedience_exercises` with two entries that
are not in the current 20-row master list: "Command
Discrimination" (Preferred Open) and "Stand Stay - Get Your
Leash" (Preferred Open). A later PR will seed both classes once
those exercises are added. GFKC does not offer Preferred classes
so MVP ships unblocked.

**Evidence:** Header comment in
`db/migrations/20260423120400_create_obedience_exercises.up.sql`
documents the gap. Loader skip path logs the class name and
reason.

### 2026-04-23: Testcontainer integration tests gated off default CI

**Decision:** The seed-loader integration test binary
(`workers/tests/seed_loader.rs`) and the Phase 0 testcontainer
tests under `shared/tests/` sit behind Cargo feature flags
(`qtrial-workers/integration-tests` and
`qtrial-shared/testing`). CI's `cargo test --workspace` runs at
default features and therefore skips them. Local dev opts in via
`cargo test --workspace --features qtrial-workers/integration-tests,qtrial-shared/testing`.

**Rationale:** Testcontainers has a known history of sporadic
flakes on GitHub Actions runners. Gating the heavy tests behind
features lets CI keep its default build-gate deterministic while
still exercising the loader end-to-end via a dedicated
service-container smoke step that runs the real binary twice
(first run + idempotency second run).

**Evidence:** `.github/workflows/ci.yml`; `required-features =
["integration-tests"]` on the workers seed_loader test binary;
similar gates on shared's phase-0 test binaries were already in
place.


### 2026-04-22: Phase 0 infrastructure cleanup complete

**Decision:** All five phase-0 feature branches (reference-schema,
tenancy, event-setup, entries, tenant-fk-validation) merged to main
via independently-reviewed sequential PRs. Total 10 PRs in the
cleanup cycle (including the pre-rename docs PR, the research-notes
cleanup, the grammar fix, and the CLAUDE.md maintenance update).

**Rationale:** The "small reviewable PRs" pattern held. Every PR
prompt written during this cycle had at least one error that Claude
Code caught before damage (phantom migrations, off-by-one counts,
unmerged branches assumed merged, missed file types in sed
patterns). The friction of "stop and flag" prevented silent failures
that would have been expensive to recover.

**Evidence:** `git log --oneline --first-parent main` shows the
linear PR history. 50 files in db/migrations/, 21 tables tracked in
DATA_MODEL.md, 2,266 lines of integration tests passing in
shared/tests/. `git grep -i offleash` returns zero hits outside
`docs/PROJECT_STATUS.md` itself; the residual references in this
file are the rename Decisions-log entries and the surrounding
narrative kept as historical record.

### 2026-04-22: Group 3 merge sequence uses sequential rebase onto main

**Decision:** Phase-0 feature branches merge in strict dependency
order after being rebased onto post-rename main, rather than as a
single "phase-0 consolidation" PR or via a merge commit preserving
the original branch structure.

**Rationale:** Each phase-0 branch represents a logically distinct
unit of work (reference schema, tenancy, event setup, entries,
tenant FK validation) that deserves individual review. A single
omnibus PR would be unreviewable (~60 files, ~5000 lines). Strict
sequential merge preserves the logical groupings while adapting to
the fact that branches predated the OffLeash rename.

**Evidence:** PRs #3, #4, phase-0-event-setup, phase-0-entries, and
(pending) tenant-fk-validation each merge independently.

### 2026-04-22: Up/down migration pairs, not forward-only

**Decision:** Continue the existing repo convention of matched
.up.sql / .down.sql migration file pairs. Do not convert to
forward-only migrations.

**Rationale:** The convention is already established and working.
Up/down pairs enable rollback during development, which is valuable
when iterating on schema. The cost of maintaining down migrations
is modest (often trivially inverse of the up). Differs from the
Mediacast Platform convention but appropriate for this project's
stage.

**Evidence:** All 24 migrations in db/migrations/ follow this
pattern.

### 2026-04-22: Per-group RLS enable migrations, not omnibus

**Decision:** Each domain group has its own enable_rls_on_*
migration (enable_rls_and_grants for tenancy, enable_rls_on_event_setup
for event layer, enable_rls_on_entry_layer for entries) rather
than a single omnibus RLS migration.

**Rationale:** Keeps RLS changes co-located with the tables they
apply to. Makes it easier to add new domain groups (financial
layer, submission layer, etc.) in future PRs without touching the
tenancy RLS migration. Fits the natural phase boundaries of the
work.

**Evidence:** db/migrations/ contains three enable_rls_on_*
migrations as of 2026-04-22.

### 2026-04-22: Postgres role design - qtrial (login, owner) + qtrial_tenant (NOLOGIN, RLS target)

**Decision:** Two Postgres roles. The `qtrial` role is the login
user, owns all application tables, runs migrations, and bypasses
RLS by Postgres convention (owners bypass RLS unless
FORCE ROW LEVEL SECURITY is set). The `qtrial_tenant` role is
NOLOGIN and is what API connections assume via SET LOCAL ROLE
inside a transaction. RLS policies are written against
qtrial_tenant. The `qtrial` role is a member of `qtrial_tenant`
to enable SET LOCAL ROLE.

**Rationale:** Standard Postgres RLS pattern. Platform admin paths
(running as qtrial) bypass RLS; application request paths (running
as qtrial_tenant) enforce it. Separation gives a clear kill-switch
if RLS policies are buggy: SET LOCAL ROLE qtrial in a database
session completely bypasses tenant isolation, which is what we
want for admin actions and migration runs but nowhere else.

**Evidence:** db/docker-init/01-create-databases.sql sets up both
roles. shared/src/tenancy.rs uses SET LOCAL ROLE qtrial_tenant
for request-scoped sessions. RLS policies reference
`current_setting('app.current_club_id')::uuid` as the filter.

### 2026-04-21: akc_fee_schedules documentation deferred to PR 2

**Decision:** The `akc_fee_schedules` table was created on
feature/phase-0-reference-schema but was NOT documented in
DATA_MODEL.md §8 at that time. Documentation will be added in
PR 2 (migrations + seed loader) as part of the same PR that
creates obedience_exercises and other ref tables.

**Rationale:** The table was created before DATA_MODEL.md was
updated to reflect it. Rather than retroactively document in a
separate trivial PR, bundle the documentation with PR 2's broader
gap-fill work, which will touch DATA_MODEL.md §8 for other new
reference tables anyway.

**Evidence:** db/migrations/20260419120300_create_akc_fee_schedules.up.sql
creates the table; DATA_MODEL.md §8 currently has no
akc_fee_schedules section.

### 2026-04-21: db_seed_akc/ history not preserved in move to db/seed/akc/

**Decision:** PR 2 will move the local seed data directory via
`mkdir db/seed/akc && mv db_seed_akc/* db/seed/akc/ && rmdir db_seed_akc`
followed by `git add db/seed/akc/`, not via `git mv`.

**Rationale:** The directory was never committed to any branch. It
has only ever existed in Robare's local working tree. There is no
history to preserve because there is no tracked history. A fresh
`git add` is the correct mechanical operation.

**Evidence:** Verified across all phase-0 branches - directory
appears in `git status` as untracked on every branch.

### 2026-04-21: jump_heights, not jumps

**Decision:** The table is named `jump_heights`, not `jumps`. The
seed CSV `db_seed_akc/jumps.csv` will be renamed to
`db/seed/akc/jump_heights.csv` as part of PR 2's seed move.

**Rationale:** DATA_MODEL.md §8 is the authoritative spec.
"jump_heights" also disambiguates from "jumps" as physical
obstacle entities, which is meaningful for future sports where
the distinction matters (Rally Choice uses jumps but not heights).

**Evidence:** DATA_MODEL.md §8 lists `jump_heights` in the table
list.

### 2026-04-21: OffLeash -> QTrial rename applied to all identifiers, not just comments

**Decision:** Phase-0 branches that predated the rename PR carry
three categories of OffLeash drift: (1) cosmetic comments, (2)
functional Postgres identifiers (`offleash_tenant` role name),
(3) ENUM values (`earned_in_offleash` on dog_titles.source). All
three are renamed during rebase; the functional ones are not
optional because they produce runtime-observable behavior.

**Rationale:** Leaving functional drift on merged branches would
break development environments silently. The cost of the rename is
mechanical (sed + verify); the cost of missing one is a silent
bug.

**Evidence:** Rebase cookbook documented in planning_notes/pr_review_packet.md.
Every phase-0 branch merged to main has gone through the rename
verification (`git grep -i offleash` returns zero after rebase
outside `docs/PROJECT_STATUS.md`, which preserves these
Decisions-log entries as historical record).

### 2026-04-21: Research notes kept in repo with product-name updates applied

**Decision:** docs/research/ files from 2026-04-19 are kept in the
repo as historical evidence supporting the Q&A-driven decisions.
The OffLeash -> QTrial rename is applied to their contents rather
than leaving them as "historical artifacts" with the old name.

**Rationale:** Initially we considered keeping them unmodified as
provenance. But ongoing `git grep -i offleash` returning noise from
these files produces real friction. The files' value is in their
content, not their historical accuracy to the product name at time
of writing. Future readers get more value from clean references
than from naming-archaeology.

**Evidence:** PR #5 applied the rename across both research files.

### 2026-04-20: Entry status is an ENUM state machine, not parallel booleans

**Decision:** An entry's status is a single ENUM column (`status`)
with values like `pending`, `confirmed`, `waitlisted`, `withdrawn`,
`absent`, `excused`, `dq`. Not five separate boolean columns.

**Rationale:** State machines are exclusive by definition. A dog is
either absent or excused or DQ'd, not some combination. Parallel
booleans would allow nonsensical states like "absent AND excused
AND winning placement 2." An ENUM makes illegal states unrepresentable.

**Evidence:** DATA_MODEL.md §1 modeling principles; entries table
and entry_lines table both use status ENUMs.

### 2026-04-20: Jump height per (dog, trial), not per entry or per dog

**Decision:** Jump height is stored in `dog_trial_jump_heights`,
keyed by (dog_id, trial_id). Not on the dog row (would be wrong
if a dog ages up between trials) and not on the entry line (would
require duplicate updates when a judge overrides the height
in-ring).

**Rationale:** Per Deborah's Q1 (2026-04-20), jump height is
determined per dog per trial. The rare judge-measurement override
(once per career) must update the height for all of that dog's
remaining classes at the current trial - cleanest if the height
is on one row, not repeated per entry line.

**Evidence:** DATA_MODEL.md §4.1, REQ-ENTRY-013, REQ-ENTRY-015.

### 2026-04-20: Co-ownership via dog_ownerships junction, not co_owners_text field

**Decision:** The legacy Access `dogs.co_owners_text` free-text
field is replaced with a `dog_ownerships` junction table
(dog_id, owner_contact_id, is_primary). Partial unique index
enforces exactly one primary owner per dog.

**Rationale:** Per Deborah's Q2 (2026-04-20), co-owners are common
in real AKC trial data. Modeling as free text loses the ability
to query or validate; modeling as structured rows lets us display
the co-owner list correctly in catalogs and confirmation emails.

**Evidence:** DATA_MODEL.md §4 dog_ownerships table definition.

### 2026-04-20: MVP AKC submission is PDF-based (Obedience/Rally), not XML

**Decision:** For MVP, AKC submission is the three-artifact PDF
package (marked catalog, judges books, Form JOVRY8 or Obedience
equivalent), sent via email to rallyresults@akc.org. XML-based
electronic submission is deferred to post-MVP when Agility is
added.

**Rationale:** Per Deborah's Q4 (2026-04-19), XML submission is
Agility-only in current AKC practice. Obedience and Rally clubs
submit PDFs. The 2004 XML schema referenced in the legacy Access
system is not the current AKC format for Obedience/Rally.

**Evidence:** REQUIREMENTS.md §14, ROADMAP.md Phase 6,
DATA_MODEL.md migration-mapping section.

### 2026-04-20: No em-dashes

**Decision:** No em-dashes in code comments, variable names,
user-facing strings, or docs written by Claude. Use regular
dashes, commas, or parentheses.

**Rationale:** Robare reads em-dashes as LLM-generated text.
Consistent with human authorship style across the codebase.

**Evidence:** docs/CLAUDE.md style section.

---

## Known gaps and pending items

### Code/schema gaps

**Tenant-scoped tables to resolve in PR 2b:** DONE 2026-04-24.

**Table alterations to resolve in PR 2c-surgery (entry pipeline):** DONE 2026-04-25.

**Table alterations to resolve in PR 2c-beta (dogs):** DONE 2026-04-26.

**Table alterations to resolve in PR 2d (events / clubs / offerings /
awards):** DONE 2026-04-26. Per the 2026-04-26 Decisions-log entries,
the BOOL-only-defer-list scope locked
`events.mixed_breeds_allowed` and the two-column shape locked the
judges-book PDF storage. The phantom "add rhtq to award_type ENUM"
bullet was a Known-gaps drift caught during CHECKPOINT 0 Phase A;
`rhtq` has been a member of the ENUM since Phase 0. The
breed-list / breed-group / breed-variety allow-list / deny-list
work is deferred to a future PR; see ROADMAP.md Phase 2+ for the
trigger and scope.

**Future PR (post-PR 2d) follow-ups:**

- Cleanup migration adding `IF NOT EXISTS` to migration
  `20260425120500_extend_dog_title_source_parsed_from_registered_name.up.sql`
  to make the existing dog_title_source ALTER TYPE round-trip
  safe under the project's no-op-down ENUM policy. PR 2d's own
  ALTER TYPE migration (`20260426120900`) already uses the
  guard; the precedent migration was caught after the fact.
- App-layer validation for `events.dogs_per_hour_override` JSONB
  keys against `canonical_classes.code` rows for the event's
  registry+sport. CHECK constraints cannot reference other
  tables; the events handler enforces. TODO marker is in the
  migration header.
- Cleanup PR for the 20 pre-existing pedantic-clippy warnings
  surfaced during CHECKPOINT 1's `cargo clippy ... -W
  clippy::pedantic` run. Pre-existing baseline; not introduced
  by PR 2d.

**Reference-data follow-ups (small, post-PR 2a):**

- Preferred Open and Preferred Utility
  `obedience_class_exercises` patterns, including two new
  `obedience_exercises` entries: "Command Discrimination" and
  "Stand Stay - Get Your Leash". Source:
  https://www.akc.org/sports/obedience/getting-started/classes/
- `jump_heights.akc_secondary_class_code` backfill from
  `db/seed/akc/post_mvp/akc_xml_jump_heights.csv`, due when the
  Agility XML submission workstream lands.
- `states.display_name` via a future migration sourcing a
  hardcoded US + CA state/province lookup.

### Business/legal pending

- USPTO TESS trademark clearance for "QTrial"
- NY LLC formation: awaiting filing confirmation from Northwest
- EIN: DIY at irs.gov (required for Relay banking)
- Warren County publication (within 120 days of formation)
- Operating Agreement (NY attorney engagement)
- Relay banking account setup (blocked on EIN)
- AWS account provisioning (aws@qtrial.app)

### Open Q&A items with Deborah

Tracked in-doc so they survive into future Q&A rounds. The
2026-04-23 round-2 email addressed Robare's eight design-focused
follow-up questions (captured in
`docs/research/2026-04-23-deborah-round-2-answers.md`); most of
the items in the list below were not part of that round and
remain open.

- Paper-entry physical workflow (folder/filing, check-stapling,
  incomplete-entry handling)
- Trial-day contingencies (late judge, dog bite, etc.)
- Refund handling for check payments (manual vs bill-pay)
- Print logistics (home printer vs print shop)
- Judge communication pattern (email vs mail)
- Missing Judges_Book_Sat.pdf body pages (needed for Phase 3
  judges book generation)
- Obedience judges-book templates from AKC's January 1, 2019 set
  (link in Deborah's 2026-04-23 email; only the Rally templates
  are in the repo today)

Resolved in 2026-04-23 round-2 email:

- AKC move-up regulation citation: Rally Regulations Chapter 1,
  Section Transfers (per Q7 answer; authoritative PDF is
  `db/seed/akc/akc_forms/akc_rally_move_up_transfer_form_2017_11.pdf`).

---

## Next planned work

In rough priority order:

1. **Phase 1 work begins** (per ROADMAP.md): club creation and
   configuration, user management, event creation, trial class
   offerings, judge directory, fee configuration, basic premium
   list PDF generation.
2. **Future PR cleanups** (small, can land alongside Phase 1): the
   `IF NOT EXISTS` guard on the `20260425120500` ENUM-extension
   migration; the pedantic-clippy cleanup PR; refresh the
   `currentDate: 2026-04-23` line in `CLAUDE.md` (stale per the
   actual wall clock).
3. **Business/legal threadwork** in parallel: EIN, Warren County
   publication, Operating Agreement, Relay banking, AWS account.

---

## How to maintain this document

Update when:

- A PR is merged (move from "In flight" to "Recently completed"
  with date and one-line summary)
- A meaningful decision is made (add to Decisions log with date,
  decision statement, rationale, and evidence)
- A pending item is resolved (remove from "Blocked / waiting" or
  "Known gaps")
- A new pending item surfaces (add to appropriate section)

Read at:

- Start of any new Claude or Claude Code session (first document
  after CLAUDE.md in the reading order)
- Before scoping any new PR
- When re-engaging with the project after time away

The Decisions log is the section most worth keeping honest. It
prevents re-litigating settled questions and gives future
contributors the rationale they need to evaluate whether a
decision still holds.