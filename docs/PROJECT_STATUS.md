# QTrial Project Status

**Last updated:** 2026-04-22
**Current phase:** Phase 0 (Foundation) - complete; preparing PR 2 (gap-fill migrations + seed loader)
**Maintained by:** Robare Pruyn, with Claude assistance

---

## Where we are right now

Phase 0 is complete. Main has 25 migration pairs (50 files in
db/migrations/), 21 tables, the shared crate with tenancy /
ParentEntity / fk_validation helpers, and 2,266 lines of passing
integration tests. `git grep -i offleash` returns zero across the
entire repo. Every phase-0 branch landed via small, independently-
reviewed PRs with no force-pushes to main.

PR 2 (migrations gap-fill + seed loader) is the next piece of work.
It adds ~22 new tables, 7 table alterations, the seed loader binary,
integration tests, a GitHub Actions CI workflow, and
db/migrations/README.md. A fresh PR 2 prompt will be written against
actual main state; prior attempts were blocked on imagined state that
didn't match the repo.

Separately, QTrial LLC formation is in flight via Northwest
Registered Agent (NY domestic, Warren County principal office,
$244 filed). Awaiting filing confirmation, then EIN, Operating
Agreement, Warren County publication, Relay banking, and AWS
account setup.

---

## Recently completed

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

Nothing currently in flight. PR 2 (migrations gap-fill + seed
loader) is the next scheduled work; its prompt has not been
written yet.

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
shared/tests/. `git grep -i offleash` returns zero.

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
verification (`git grep -i offleash` returns zero after rebase).

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

### Code/schema gaps (to resolve in PR 2)

- 23 new tables not yet created (title_prefixes, title_suffixes,
  breed_groups, breeds, breed_varieties, jump_heights,
  obedience_exercises, obedience_class_exercises (these replace
  DATA_MODEL.md §8's current `exercises` stub; PR 2 will update
  §8 as part of the same PR that creates the migrations),
  countries, states, otch_points, om_points, rally_rach_points,
  sport_time_defaults, platform_admins, dog_ownerships,
  dog_trial_jump_heights, armband_assignments, email_templates,
  submission_records, audit_log, payments, refunds)
- 7 table alterations pending (dogs column rework, entries.armband
  drop, entry_lines handler columns, entry_line_results timing/
  scoring, events dogs_per_hour_override + per_series enum value,
  dog_titles.source enum extension with parsed_from_registered_name)
- Seed loader binary not yet written (workers/src/bin/seed_loader.rs)
- CI workflow not yet present (.github/workflows/ci.yml)
- db/migrations/README.md explaining conventions not yet written
- db_seed_akc/ is untracked; needs mkdir + mv + git add into
  db/seed/akc/
- akc_fee_schedules documentation missing from DATA_MODEL.md §8

### Business/legal pending

- USPTO TESS trademark clearance for "QTrial"
- NY LLC formation: awaiting filing confirmation from Northwest
- EIN: DIY at irs.gov (required for Relay banking)
- Warren County publication (within 120 days of formation)
- Operating Agreement (NY attorney engagement)
- Relay banking account setup (blocked on EIN)
- AWS account provisioning (aws@qtrial.app)

### Open Q&A items with Deborah

Tracked in-doc so they survive into future Q&A rounds:

- Paper-entry physical workflow (folder/filing, check-stapling,
  incomplete-entry handling)
- Trial-day contingencies (late judge, dog bite, etc.)
- Refund handling for check payments (manual vs bill-pay)
- Print logistics (home printer vs print shop)
- Judge communication pattern (email vs mail)
- AKC move-up deadline regulation specifics for 2026
- Missing Judges_Book_Sat.pdf body pages (needed for Phase 3
  judges book generation)

---

## Next planned work

In rough priority order:

1. **PR 2 prompt**: will be written against actual main state (not
   imagined state). Scope: migrations gap-fill (~22 new tables + 7
   alterations), seed loader binary with integration tests, GitHub
   Actions CI workflow, db/migrations/README.md, and documenting
   akc_fee_schedules in DATA_MODEL.md §8 alongside the new
   reference tables.
2. **PR 2 execution**: land in a dedicated session.
3. **Phase 1 work begins** (per ROADMAP.md): club creation and
   configuration, user management, event creation, trial class
   offerings, judge directory, fee configuration, basic premium
   list PDF generation.
4. **Business/legal threadwork** in parallel: EIN, Warren County
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