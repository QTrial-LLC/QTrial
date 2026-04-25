# PR 2d CHECKPOINT 0 - Phase B/C design note

**Branch:** `feat/pr-2d-events-clubs-awards` off main at
`9543fab1940b4a03b88292e902a9e720aabe81cf`
**Phase A report:** `docs/research/2026-04-25-pr-2d-checkpoint-0-state-verification.md`
**Date:** 2026-04-25 (wall-clock per `date` command)

---

## 1. Scope summary

PR 2d is the events / clubs / offerings / awards additive plumbing
PR. Scope per the kickoff:

1. `events.dogs_per_hour_override JSONB`
2. `events.armband_scheme` ENUM extension: add `per_series`
3. `events.trial_chair_user_id` and `events.event_secretary_user_id`
   (FK to users)
4. `events.mixed_breeds_allowed BOOL` (flag only; breed-list deferred)
5. `combined_award_groups` reference table + junction
6. `clubs.officers_json JSONB`
7. `trial_awards.award_type` ENUM extension: add `rhtq`  **(NO-OP, see drift)**
8. `trial_class_offerings.judges_book_pdf_object_key TEXT`
9. Post-trial signed-scan handling: lock or revise the 2026-04-24
   working assumption

The Phase A verification report logged one drift (the missing
mixed_breeds scope-lock; resolved on 2026-04-25 by Robare picking the
BOOL-only-defer-list path) and one prompt-side error (the
non-existent `entry_status` ENUM). Phase B surfaced one additional
drift below.

### Drift surfaced during Phase B

**`rhtq` is ALREADY a member of the `award_type` ENUM as of Phase 0.**
The kickoff (item 7) and `docs/PROJECT_STATUS.md:900-901` Known-gaps
both list "add `rhtq` to `award_type` ENUM" as PR 2d work. Verified
in `db/migrations/20260419140400_create_judge_assignments_and_awards.up.sql:45-54`:
the ENUM was created with all eight values including `rhtq`. There is
no ENUM extension to do for `award_type`. Item 7 of the kickoff is a
no-op; the Known-gaps bullet should be deleted in CHECKPOINT 1's
PROJECT_STATUS update.

This is a smaller drift than A3 and does not require a stop. It does
mean B7's migration list is shorter than the kickoff implied: only
one ALTER TYPE migration (for `armband_scheme`), not two.

**`armband_scheme.per_series` is forward-referenced in DATA_MODEL.md.**
`docs/DATA_MODEL.md:142` lists `per_series` as one of the
`armband_scheme` ENUM values. The migration only carries four
(`per_trial`, `per_event`, `per_day`, `per_class`). DATA_MODEL.md is
ahead of the migrations on this. PR 2d aligns reality with the doc;
no DATA_MODEL change needed for this row, only a migration.

---

## 2. Per-item decisions

### B1. `combined_award_groups` shape

**Decision: parent table + junction table (option b), with a small
extension for non-per-trial-award groupings.**

**Schema sketch:**

```sql
-- Reference data, registry-scoped, permissive-read RLS pattern.
CREATE TABLE combined_award_groups (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id           UUID NOT NULL REFERENCES registries(id),
    sport                 sport NOT NULL,
    -- Stable code for app-layer references and seed idempotency.
    -- Examples: 'akc_obedience_hc', 'akc_rally_rhtq', 'akc_rally_rae'.
    code                  TEXT NOT NULL,
    display_name          TEXT NOT NULL,
    -- The per-trial award type this group produces. NULL when the
    -- group exists for discount-eligibility only (RAE, RACH; both
    -- are title-progression paths, not per-trial awards).
    award_type            award_type,
    is_discount_eligible  BOOL NOT NULL DEFAULT TRUE,
    regulation_citation   TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX combined_award_groups_registry_sport_code_uk
    ON combined_award_groups (registry_id, sport, code);

CREATE TABLE combined_award_group_classes (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    combined_award_group_id     UUID NOT NULL
        REFERENCES combined_award_groups(id) ON DELETE CASCADE,
    canonical_class_id          UUID NOT NULL
        REFERENCES canonical_classes(id),
    -- TRUE when the class must be Q'd in to earn the per-trial award
    -- (HC requires Open B AND Utility B; HTQ requires Adv B AND Ex B
    -- AND Master). FALSE when the class is one of several optional
    -- contributors (RAE/RACH paths where A or B levels are
    -- interchangeable for discount eligibility).
    is_required_for_award       BOOL NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX combined_award_group_classes_group_class_uk
    ON combined_award_group_classes
       (combined_award_group_id, canonical_class_id);
```

**Rationale.** The straw-man parent + junction shape is the right
answer; the only extension is making `award_type` nullable on the
parent so RAE and RACH (which are title-progression paths, not
per-trial awards) can sit in the same table as HC/HTQ/RHC. This
matches Deborah's Q4 answer that the discount-eligibility logic
applies to "ANY double or triple Q in B classes," not just to the
groups that produce per-trial awards. The fee engine asks
"is this dog entered in 2+ classes in the same combined_award_group
at this trial?" regardless of whether the group produces a trial
award.

**Sub-questions answered:**

- **Sport scoping.** Column on parent (`sport sport NOT NULL`).
  No DDL trigger enforcing junction sport-match; app-layer + seed
  validates. Mirrors the existing `canonical_classes` pattern where
  `sport` is denormalized rather than walked from a foreign key.

- **Discount-eligibility flag.** Parent-level `is_discount_eligible
  BOOL NOT NULL DEFAULT TRUE`. The kickoff straw-man also proposed a
  per-junction `is_required_for_award`, which I am keeping for the
  reason stated above.

- **Linkage to `trial_awards.award_type` ENUM.** Direct ENUM
  reference on the parent. Migration ordering is trivial because
  `rhtq` is ALREADY in the ENUM (see drift section). The seed CSV
  can name `rhtq` freely without an ALTER TYPE ordering interlock.

- **RLS pattern.** Reference-data permissive-read RLS, identical to
  the 14 PR 2a reference tables. RLS-enable lands in a single
  `enable_rls_on_*` migration following the convention captured in
  `db/migrations/README.md:114-122`. No `club_id`. SELECT-only grant
  to `qtrial_tenant`. Writes happen via the `qtrial` role (the seed
  loader) which bypasses RLS.

- **Initial seed.** Five rows for AKC Obedience + AKC Rally, the
  only sports we currently seed:

  | code | display_name | award_type | members (junction, all is_required_for_award=TRUE unless noted) | citation |
  |---|---|---|---|---|
  | `akc_obedience_hc` | High Combined | `hc` | Open B, Utility B | Obedience Reg. Ch. 1 §31 |
  | `akc_rally_rhc` | Rally High Combined | `rhc` | Rally Advanced B, Rally Excellent B | Rally Reg. Ch. 1 §31 |
  | `akc_rally_rhtq` | Rally High Triple Qualifying | `rhtq` | Rally Advanced B, Rally Excellent B, Rally Master | Rally Reg. Ch. 1 §32 |
  | `akc_rally_rae` | Rally Advanced Excellent (title path) | NULL | Rally Advanced A, Rally Advanced B, Rally Excellent A, Rally Excellent B (`is_required_for_award=FALSE`) | Rally Reg. Ch. 1 §27 |
  | `akc_rally_rach` | Rally Advanced Champion (title path) | NULL | Rally Master, Rally Choice (`is_required_for_award=FALSE`) | Rally Reg. Ch. 1 §28 |

  RAE and RACH use `is_required_for_award=FALSE` on every member
  because the title-progression rule "any qualifying combination"
  cannot be expressed by a simple "all required" flag. The fee
  engine treats `FALSE` membership as "contributes to discount but
  not required for any per-trial award"; the title-progression
  engine has its own per-title rules consulting `canonical_classes`
  directly. This is enough for the PR 2d shape; the engine work
  itself is later.

  The Obedience HTQ/H Triple Qualifying path is intentionally
  absent. AKC Obedience does not have a triple-class combined award
  analogous to Rally HTQ. Confirm with Deborah if uncertain.

  AKC citations are taken from the regulation chapter/section
  references on `akc_rally_hc_htq_tiebreaker_2018.pdf` (HC = §31,
  HTQ = §32). RAE and RACH section numbers are author's
  best-guess from AKC Rally Regulations table of contents and need
  Robare/Deborah verification before the seed lands. **Open
  question marker.**

**Open issue.** The exact AKC regulation section numbers for RAE
and RACH need verification before the seed CSV is finalized.
Including a wrong citation in a seed CSV is worse than including
none. CHECKPOINT 1 should either (a) ask Deborah, (b) cite "Rally
Regulations Ch. 1 §27/§28" with a TODO comment, or (c) drop the
`regulation_citation` column from the seed and add it later. My
recommendation is (b) with a clearly-marked TODO; the column is
nullable so a NULL entry is acceptable too.

---

### B2. `clubs.officers_json` shape

**Decision: array of office records (option b), with serde-typed
struct on the app side.**

**Schema sketch:**

```sql
ALTER TABLE clubs
    ADD COLUMN officers_json JSONB;
```

**Document shape (app-layer typed struct, serde):**

```jsonc
[
  {"office": "President", "name": "...", "email": "...", "phone": "..."},
  {"office": "Vice President", "name": "...", "email": "...", "phone": "..."},
  {"office": "Treasurer", "name": "...", ...},
  {"office": "Recording Secretary", "name": "...", ...},
  {"office": "AKC Delegate", "name": "...", ...},
  {"office": "Board Member", "name": "Pat Prutsman", ...},
  {"office": "Board Member", "name": "Lois Hammond", ...}
]
```

**Rationale.** The GFKC June 2026 premium list at
`db/seed/akc/sample_artifacts/gfkc_rally_premium_2026_06.pdf`
demonstrates that real club officer lists mix singular roles
(President, VP, Treasurer, Recording Secretary, AKC Delegate) with
multi-person roles (Board of Directors lists eight names). The
object-keyed shape (option a) does not handle multi-person roles
without contortion (`"board_members": [...]` as a sub-array, which
breaks the uniform "every value is one person" property and forces
the renderer to special-case the key). Array-of-records is uniform.

The premium list renders officers in a specific order (President,
VP, Treasurer, Recording Secretary, AKC Delegate, then Board
members). Array-form preserves order by index without any extra
metadata; object-form would need either an explicit `display_order`
or a rendering-side mapping table.

**Sub-questions answered:**

- **Premium-list rendering order.** Array index is the order. The
  renderer iterates the array and emits each row.

- **Validation.** App-layer typed struct via serde. No DDL CHECK on
  `jsonb_typeof`; the app layer serializes/deserializes through a
  single typed struct so writes that don't match the shape fail at
  the API boundary, not at the database. This matches the project
  convention for other JSONB fields (see
  `canonical_classes.title_earning_rule JSONB` which has no DDL
  validation).

- **Well-known offices observed in the GFKC sample.** President,
  Vice President, Treasurer, Recording Secretary, AKC Delegate
  (singular). Board Member (multi). Show Committee members (Maureen
  Kramer, Eileen Rogers, Mary Clothier, Debbie Pruyn) appear on the
  same page but those belong to a SHOW COMMITTEE that is event-level
  (per Deborah's Q5 framing), not club-level - so they go on the
  event/personnel side, not on `clubs.officers_json`.

  Veterinarian-on-call is also event-level, captured separately if
  ever needed.

- **Future `club_officers` historical-preservation table.** The
  array-of-records shape maps cleanly to a future row-per-officer
  table with columns `(office, name, email, phone, effective_from,
  effective_to)`. Migration path is mechanical: one row per array
  element, with `effective_from` set to the day the migration
  runs and `effective_to` NULL.

- **Free-form vs. controlled list.** MVP accepts any TEXT in
  `office`. The sample PDF surfaces non-standard offices like
  "AKC Delegate" that vary club to club. Forcing a controlled list
  is YAGNI for MVP.

- **Default value and nullability.** NULL allowed (clubs already
  exist; backfill is NULL and the renderer treats NULL as "no
  officers configured"). No DEFAULT.

**Open issue.** Should the migration also seed GFKC's officer list
as part of bootstrap data, or leave it for an admin to populate via
the UI? My recommendation: leave NULL on migration; backfill via
admin tooling once UI lands. The data is club-specific and goes
stale (Deborah noting "I just left the board"), so seeding it is
the wrong path.

---

### B3. `trial_class_offerings.judges_book_pdf_object_key` and signed-scan handling

**Decision: REVISE the 2026-04-24 working assumption. Use TWO
columns (option b), not one with overwrite.**

**Schema sketch:**

```sql
ALTER TABLE trial_class_offerings
    ADD COLUMN judges_book_pdf_object_key  TEXT,
    ADD COLUMN signed_scan_pdf_object_key  TEXT;
```

`judges_book_pdf_object_key` is the pre-trial blank PDF generated
by QTrial for printing. `signed_scan_pdf_object_key` is the
post-trial scan of the signed-and-completed judges book uploaded
back into QTrial.

**Rationale.** The 2026-04-24 working assumption was "overwrite the
same column at scan time" (option a). Phase A item A6 surfaced that
`submission_records` does NOT carry a signed-scan reference (the
2026-04-24 Decisions-log entry "submission_records scope is
electronic submission only" intentionally excludes physical-mail
artifacts). That means option (a) loses the blank PDF when the scan
overwrites it AND offers no place else to put either artifact in a
durable, addressable form.

Two columns is the honest shape:

1. **Blank-regeneration safety.** If a judge changes late in the
   pre-trial cycle (Deborah's research note flags this as a real
   case), the renderer regenerates the blank without clobbering an
   already-uploaded signed scan. With one column, regenerating after
   a scan upload would either silently overwrite the scan or require
   an additional "is this a scan?" check before rendering.

2. **Audit trail per REQ-SUB-002 and REQ-SUB-004.**
   `docs/REQUIREMENTS.md:519-521` defines these as:
   - REQ-SUB-002: "QTrial shall generate pre-printed judges books
     per class for judge use during trials."
   - REQ-SUB-004: "QTrial shall support emailing the submission
     package to AKC. The secretary may alternatively mail a physical
     package to AKC Event Operations."
   The pre-printed PDF (REQ-SUB-002) and the post-trial scan
   (operational confirmation that the right artifact was mailed) are
   different artifacts at different lifecycle stages. Storing them
   separately preserves both for audit, especially for a club that
   ever needs to reproduce what was mailed.

3. **Cost is trivial.** Two `TEXT NULL` columns. No new state
   machine, no ENUM, no new table. The "two columns is more complex"
   argument doesn't hold here.

Option (c) (single column plus state ENUM) was rejected per the
kickoff's own framing: an ENUM doing the work of a second column
without the type-safety benefit is the worst of both worlds.

**Sub-questions answered:**

- **A6 implication.** `submission_records` doesn't carry the signed
  scan, so the artifact MUST live somewhere else. Two columns on
  `trial_class_offerings` is the cleanest place.

- **Re-render of pre-trial blank.** Two columns makes this a
  one-line UPDATE on `judges_book_pdf_object_key`; the signed scan
  is untouched.

- **REQ-SUB-* audit trail.** Both artifacts addressable, both
  durable. Best supported by option (b).

**Open issue.** The two-column shape is a revision of the
2026-04-24 working assumption. The 2026-04-24 Decisions-log entry
("submission_records scope is electronic submission only") is
unaffected; only the open-question framing in PROJECT_STATUS.md
Known-gaps lines 905-907 changes. CHECKPOINT 1 should add a new
Decisions-log entry locking the two-column shape and update the
Known-gaps bullet to "RESOLVED 2026-04-25: two columns on
trial_class_offerings."

---

### B4. `events.dogs_per_hour_override` JSONB shape

**Decision: keys are `canonical_classes.code` only. No
sport-level override key. App-layer validates keys on write.**

**Schema sketch:**

```sql
ALTER TABLE events
    ADD COLUMN dogs_per_hour_override JSONB;
```

**Document shape:**

```jsonc
{
  "rally_choice": 4.3,
  "rally_master": 3.5,
  "rally_excellent_b": 3.1
}
```

Keys are exact `canonical_classes.code` strings (per A8: the
natural unique key is `(registry_id, sport, code)`; the event row
already pins `registry_id`, so `code` alone is unambiguous within
an event). Values are NUMERIC-castable strings or numbers
representing minutes-per-dog.

The fallback chain when the schedule generator computes per-class
pacing is:

1. `events.dogs_per_hour_override[class.code]` (event-level per-class override)
2. `trial_class_offerings.per_dog_minutes` (per-trial-offering override; already exists)
3. `canonical_classes.dogs_per_hour_default` (catalog default)
4. `sport_time_defaults.minutes_per_dog` keyed by sport_or_event (last-resort sport-level default)

**Rationale.** Deborah's actual GFKC numbers were per-class (Rally
Choice 4.3, Rally Master 3.5, Rally Excellent B 3.1), not per-sport.
A sport-level override key on this column would duplicate
`sport_time_defaults` while introducing precedence ambiguity. The
schedule generator already has `sport_time_defaults` as the
sport-level fallback; the event-level override is for the
per-class case Deborah described, nothing else.

**Pushback on the kickoff straw-man.** The straw-man proposed
"keys are either canonical_classes.code or sport_or_event;
precedence is class-code beats sport." I am rejecting the
sport_or_event key. Reasons:

1. The use case Deborah cited is per-class. No documented use case
   for per-event sport-level override.
2. Sport-level pacing is global per platform, not per event. If
   a club's whole-Rally pacing differs from AKC defaults, that is
   an upstream `sport_time_defaults` concern, not a per-event
   override.
3. Two key namespaces in one JSONB column is a foot-gun. A
   secretary typing "Obedience" into the override editor would
   silently get sport-level behavior; typing "open_b" would get
   class-level. Mixing them invites confusion.

If a future sport-level event override use case surfaces, it can
be added as a separate column (`sport_pacing_override JSONB` or
similar). YAGNI today.

**Sub-questions answered:**

- **Granularity for Agility sub-events.** Not in MVP scope. Agility
  doesn't ship in PR 2d's seed; the sport ENUM only has
  `obedience` and `rally`. When Agility lands, sub-events
  (Standard, JWW, FAST, ISC) get their own canonical_classes rows
  with codes; the same `dogs_per_hour_override` keyed by code
  works.

- **`class_change_seconds` and `event_change_seconds`.** Stay only
  on `sport_time_defaults`. No per-event override at this layer
  for those knobs. Same YAGNI argument.

- **Validation.** App-layer validates keys on write against
  `canonical_classes.code` rows for the event's registry+sport.
  Schedule generator falls through to the next fallback level on
  unknown keys (logged at WARN), not error. Tolerant of seed CSVs
  that include a class code the event doesn't actually offer.

**Open issue.** Should the JSONB enforce numeric values via a
`CHECK (jsonb_typeof(value) = 'number')` for each value? My
recommendation: NO at DDL level, YES at app-layer typed struct.
JSONB CHECKs that walk values are awkward to write and the
typed-struct path is the project convention for JSONB.

---

### B5. `events.trial_chair_user_id`, `events.event_secretary_user_id`, and the disposition of `trials.trial_chairperson`

**Decision:**

1. Add `events.trial_chair_user_id UUID REFERENCES users(id) ON DELETE SET NULL` (NULL allowed).
2. Add `events.event_secretary_user_id UUID REFERENCES users(id) ON DELETE SET NULL` (NULL allowed).
3. **DROP `trials.trial_chairperson`** (no consumers found in code).

**Schema sketch:**

```sql
ALTER TABLE events
    ADD COLUMN trial_chair_user_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN event_secretary_user_id  UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX events_trial_chair_user_id_ix
    ON events (trial_chair_user_id) WHERE deleted_at IS NULL;
CREATE INDEX events_event_secretary_user_id_ix
    ON events (event_secretary_user_id) WHERE deleted_at IS NULL;

ALTER TABLE trials DROP COLUMN trial_chairperson;
```

**Rationale.**

- **ON DELETE behavior.** SET NULL matches the existing
  `created_by` / `updated_by` pattern on
  `events` (`db/migrations/20260419140100_create_events_and_days.up.sql:55-56`).
  RESTRICT would block user hard-deletion any time the user has
  ever been assigned, which is too aggressive for the chair/
  secretary case where user accounts may legitimately be
  removed.

- **Nullability.** NOT NULL would force an event in `draft`
  status to have both assigned, which contradicts the workflow:
  per Deborah's WORKFLOWS.md §1.3, the Trial Secretary role is
  an early-invite role often filled before the chair is
  identified. NULL allowed means a draft event can be saved
  without either assigned; the API layer enforces "both must be
  set before status transitions to `open`."

- **Indexes.** Partial unique on `deleted_at IS NULL` matches the
  pattern on every other event-subtree FK index.

- **Disposition of `trials.trial_chairperson`.**
  `rg -n trial_chairperson` against `api/`, `workers/`, `shared/`,
  `web/` returns ZERO consumers. The only references are
  `docs/DATA_MODEL.md:171` (column-table row) and the Phase A
  verification report. No deprecation path is needed; drop the
  column outright.

  The reason it should be dropped (not kept as a per-trial
  override of an event-level chair): per Deborah's Q5 and the
  GFKC June 2026 premium PDF, the trial chair role is an
  event-level concern. "Rally Trial Chair: Chris Argento" is
  printed once on the premium for both Saturday and Sunday
  trials. Multiple trials within an event share one chair. A
  per-trial override is a hypothetical use case with no
  documented real-world need.

  If a future case ever surfaces where a single event hosts
  trials with different chairs (e.g., a cluster running
  Obedience and Rally trials with separate chairs), it can be
  added back as `trials.trial_chair_user_id_override` at that
  time.

**Sub-questions answered (all checked above).**

**Open issue.** Robare confirms drop, not deprecate. (My read of
the project convention is to drop columns outright when no
consumers exist; deprecation paths are for cross-PR work where
external consumers might exist. PR 2d has no such consumers.)

---

### B6. `events.mixed_breeds_allowed` scope establishment

**Decision: add `events.mixed_breeds_allowed BOOL NOT NULL DEFAULT
TRUE`. Defer the breed-list model to a future PR.**

**Schema sketch:**

```sql
ALTER TABLE events
    ADD COLUMN mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE;
```

**Decisions-log entry text (drafted for verbatim paste into
PROJECT_STATUS.md in CHECKPOINT 1):**

> ### 2026-04-25: events.mixed_breeds_allowed ships as BOOL only; breed-list model deferred
>
> **Decision:** PR 2d adds `events.mixed_breeds_allowed BOOL NOT
> NULL DEFAULT TRUE`. The breed-list approach (junction tables
> associating events with allowed breeds, breed_groups, or
> breed_varieties) is deferred to a future PR.
>
> **Rationale:**
>
> 1. PR 2d scope is already non-trivial. Adding a breed-list
>    design adds another migration-ordering question plus three
>    or four sub-questions plus a new junction table.
> 2. The mixed-breeds case Deborah called out in Q3
>    (conformation excluding mixed) is structurally the
>    All-American Dog flag path. The breed-restricted-event
>    case (a Specialty for one breed) is structurally separate
>    work and warrants its own design pass.
> 3. The flag-only path is fully additive. The breed-list
>    junction can land in a later PR without touching events
>    again.
> 4. We have no real artifact to design breed-list against.
>    GFKC June 2026 Rally has no breed restrictions. Designing
>    on speculation produces a worse model than waiting until a
>    Specialty or Group show artifact is in hand.
>
> **Evidence:** `docs/research/2026-04-25-pr-2d-checkpoint-0-design-note.md`.
>
> **Supersession note:** This decision supersedes the framing in
> the 2026-04-23 round-2 research note ("alongside the
> breed-list approach", at
> `docs/research/2026-04-23-deborah-round-2-answers.md` lines
> 49-57) and the corresponding bullet in `docs/REQUIREMENTS.md`
> line 87. Those documents capture earlier intent before the
> deferral was scoped. The research note stays unedited as
> historical evidence; `docs/REQUIREMENTS.md` and
> `docs/PROJECT_STATUS.md` Known-gaps are updated in CHECKPOINT 1.

**Sub-questions answered:**

- **Default value: BOOL NOT NULL DEFAULT TRUE.** Confirmed.
  Most events accept All-American Dog; opting out is the
  exception (predominantly conformation, which is post-MVP
  anyway). Default-TRUE means existing events created before the
  column lands automatically inherit "mixed allowed", which is
  also the most-likely-correct value.

- **Nullability: NOT NULL.** Confirmed. Nullable would create a
  meaningless three-state where "unknown" is just "default not
  yet considered." With a sensible default, NOT NULL is cleaner.

- **Down-migration: drop column.** Acceptable. Any explicit
  FALSE rows would lose that information on rollback, but
  rollback in PR 2d is a development-time mechanic; no
  production data exists yet.

**Future work (to be added to PROJECT_STATUS.md Known-gaps in
CHECKPOINT 1):**

- breed-list, breed-group-list, or breed-variety-list allow-list
  or deny-list on events
- junction tables associating events with specific breeds
- validation that an entered dog's breed satisfies the event's
  breed restrictions
- premium-list rendering of breed restrictions

---

### B7. Migration ordering interlock

**Decision: nine migration files in a single PR, ordered as below.**

The kickoff feared two ENUM-ordering interlocks: one for
`armband_scheme.per_series` and one for
`trial_awards.award_type.rhtq`. The second is a no-op (rhtq is
already in the ENUM). Only the first matters, and per the
2026-04-25 Decisions-log entry "Postgres ENUM additions are
one-way; down migrations are no-ops" plus the 2026-04-25
precedent migration
`20260425120500_extend_dog_title_source_parsed_from_registered_name.up.sql`,
ALTER TYPE ADD VALUE in Postgres 16 runs inside a transaction as
long as the new value is not USED in the same transaction. So no
`-- no-transaction` directive is needed for the `per_series`
addition either, since the migration only adds the value (no rows
are inserted that reference it in the same migration).

The combined_award_groups seed runs OUTSIDE migrations via the
`qtrial-seed-loader` binary. Per
`db/migrations/README.md:78-83` the seed loader runs after all
migrations complete. The seed CSV will reference `rhtq` in the
`award_type` column for the new combined_award_groups rows, but
since `rhtq` has been in the ENUM since Phase 0 there is no
ordering issue.

**Proposed migration files (timestamps assume 2026-04-25
sequencing; HHMMSS pads from `120000` upward in 100-second
increments to leave room for late additions):**

| # | Filename | Description |
|---|---|---|
| 1 | `20260425120000_add_events_mixed_breeds_allowed.up.sql` / `.down.sql` | Add `events.mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE`. Down drops column. (B6) |
| 2 | `20260425120100_add_events_trial_chair_and_secretary_fks.up.sql` / `.down.sql` | Add `events.trial_chair_user_id` + `events.event_secretary_user_id`, both FK to users with ON DELETE SET NULL, both nullable; partial indexes on deleted_at IS NULL. Down drops indexes + columns. (B5) |
| 3 | `20260425120200_drop_trials_trial_chairperson.up.sql` / `.down.sql` | Drop `trials.trial_chairperson`. Down adds the column back as TEXT (nullable). Pure schema move; data loss on rollback is acceptable per project convention (Phase 0 column with no production data). (B5) |
| 4 | `20260425120300_add_events_dogs_per_hour_override.up.sql` / `.down.sql` | Add `events.dogs_per_hour_override JSONB`, nullable. Down drops column. (B4) |
| 5 | `20260425120400_extend_armband_scheme_per_series.up.sql` / `.down.sql` | `ALTER TYPE armband_scheme ADD VALUE 'per_series'`. Down is no-op with explanatory comment per the 2026-04-25 ENUM-extension Decisions-log policy. Transactional (no `-- no-transaction` directive needed). (B7) |
| 6 | `20260425120500_add_clubs_officers_json.up.sql` / `.down.sql` | Add `clubs.officers_json JSONB`, nullable. Down drops column. (B2) |
| 7 | `20260425120600_add_trial_class_offerings_judges_book_columns.up.sql` / `.down.sql` | Add `trial_class_offerings.judges_book_pdf_object_key TEXT` + `trial_class_offerings.signed_scan_pdf_object_key TEXT`, both nullable. Down drops both. (B3) |
| 8 | `20260425120700_create_combined_award_groups.up.sql` / `.down.sql` | CREATE TABLE `combined_award_groups` and `combined_award_group_classes` plus their unique indexes. No RLS or grants in this file (separate enable_rls migration follows the `enable_rls_on_*` convention). Down drops both tables. (B1) |
| 9 | `20260425120800_enable_rls_on_combined_award_groups.up.sql` / `.down.sql` | ENABLE ROW LEVEL SECURITY on both tables, CREATE POLICY `*_read_all` USING (TRUE), GRANT SELECT TO qtrial_tenant. Down disables RLS, drops policies, revokes grants. (B1) |

**Within-PR ordering rationale:**

- Migrations 1-7 are independent column-or-type adds with no
  cross-references between them. Order is arbitrary as long as
  each file has a unique timestamp. The order above groups
  related changes (event additions first, trials drop, then
  clubs and offerings, then ENUM extension before tables
  reference it).

- Migrations 8 and 9 are the only ones with a hard ordering
  requirement: 9 must follow 8 because RLS-enable references
  the tables created in 8. This is the project convention
  (separate `enable_rls_on_*` file per group of tables).

- Migration 5 (`per_series` ALTER TYPE) does not need to come
  before any other PR 2d migration. No PR 2d migration writes a
  row using the `per_series` value; the value becomes usable
  for app code in subsequent PRs. The ALTER TYPE is purely
  additive.

**No `-- no-transaction` directive needed in any PR 2d
migration.** All migrations are transactional. The ENUM ADD
VALUE precedent (2026-04-25) confirms transactional ENUM
extension is fine in Postgres 16 when the new value is not used
in the same transaction.

**Seed-loader CSV.** `combined_award_groups` and
`combined_award_group_classes` get new CSVs in `db/seed/akc/`:

- `combined_award_groups.csv` (5 rows: 1 Obedience HC, 4 Rally:
  RHC, RHTQ, RAE, RACH)
- `combined_award_group_classes.csv` (~13 rows: HC=2, RHC=2,
  RHTQ=3, RAE=4, RACH=2)

Loader code in `workers/src/seed_loader/` extends to read these
CSVs and upsert idempotently against the natural keys
`(registry_id, sport, code)` for the parent and
`(combined_award_group_id, canonical_class_id)` for the
junction.

---

## 3. Proposed migration file list (consolidated)

Same as the table in B7. Repeated here for direct copy-paste into
CHECKPOINT 1's plan:

```
20260425120000_add_events_mixed_breeds_allowed.up.sql / .down.sql
20260425120100_add_events_trial_chair_and_secretary_fks.up.sql / .down.sql
20260425120200_drop_trials_trial_chairperson.up.sql / .down.sql
20260425120300_add_events_dogs_per_hour_override.up.sql / .down.sql
20260425120400_extend_armband_scheme_per_series.up.sql / .down.sql
20260425120500_add_clubs_officers_json.up.sql / .down.sql
20260425120600_add_trial_class_offerings_judges_book_columns.up.sql / .down.sql
20260425120700_create_combined_award_groups.up.sql / .down.sql
20260425120800_enable_rls_on_combined_award_groups.up.sql / .down.sql
```

Plus seed CSV additions:

```
db/seed/akc/combined_award_groups.csv         (new, 5 rows)
db/seed/akc/combined_award_group_classes.csv  (new, ~13 rows)
```

Plus seed-loader code change in `workers/src/seed_loader/` to
parse and load the two new CSVs (idempotent upsert pattern
matching the existing PR 2a reference-table loaders).

---

## 4. Decisions-log entry text (B6) - drafted verbatim

Already drafted in B6 above. Copy-paste-ready into
`docs/PROJECT_STATUS.md` Decisions-log section in CHECKPOINT 1,
inserted before the existing 2026-04-26 entries (chronological
order: 2026-04-25 entries appear before 2026-04-26).

**A second 2026-04-25 Decisions-log entry to add** for the B3
two-column shape:

> ### 2026-04-25: judges-book PDF and signed scan use two columns on trial_class_offerings
>
> **Decision:** Pre-trial blank judges-book PDF and post-trial
> signed scan are stored as two separate columns on
> `trial_class_offerings`:
> `judges_book_pdf_object_key TEXT` and
> `signed_scan_pdf_object_key TEXT`, both nullable.
>
> **Rationale:** The 2026-04-24 working assumption was overwrite
> a single column at scan time. Two reasons to revise:
> 1. `submission_records` does NOT carry a signed-scan reference
>    (per the 2026-04-24 Decisions-log entry
>    "submission_records scope is electronic submission only";
>    physical-mail artifacts are intentionally excluded). The
>    signed scan must live somewhere; the cleanest place is the
>    same row that carries the blank.
> 2. Re-rendering the blank after a late judge change must not
>    clobber an already-uploaded scan. Two columns makes
>    regeneration a one-line UPDATE on the blank column alone.
>
> The cost is two `TEXT NULL` columns. No new state machine, no
> ENUM. Simplest honest shape.
>
> **Evidence:** `docs/research/2026-04-25-pr-2d-checkpoint-0-design-note.md`
> §B3.
>
> **Supersession note:** Replaces the 2026-04-24 working
> assumption ("overwrite the same column at scan time") that was
> tracked as an OPEN QUESTION in `docs/PROJECT_STATUS.md`
> Known-gaps lines 905-907.

---

## 5. Spec-doc updates for CHECKPOINT 1

Every spec-doc edit CHECKPOINT 1 must make alongside the
migrations. File paths and brief descriptions:

### `docs/PROJECT_STATUS.md`

1. Add the two 2026-04-25 Decisions-log entries drafted in §B6
   and §4 above (mixed_breeds scope; judges-book two-column
   shape).
2. Update Known-gaps lines 889-907:
   - **Replace** "events breed restrictions, including
     events.mixed_breeds_allowed BOOL alongside the breed-list
     approach (Deborah's item 3 follow-up)" with "events
     breed-list / breed-group / breed-variety restrictions
     (post-PR 2d; the BOOL flag landed in PR 2d, the list model
     is deferred)."
   - **Delete** the "trial_awards.award_type ENUM extension to
     include rhtq" bullet (it was a phantom; rhtq has been in
     the ENUM since Phase 0).
   - **Update** the "Post-trial signed-scan handling" bullet to
     "RESOLVED 2026-04-25: two columns
     (judges_book_pdf_object_key for the blank,
     signed_scan_pdf_object_key for the post-trial scan) on
     trial_class_offerings."
3. Move the in-flight section to "Recently completed" once the PR
   merges (standard end-of-PR maintenance).
4. Bump "Last updated" to 2026-04-25 (or whichever wall-clock
   date CHECKPOINT 1 lands on).

### `docs/REQUIREMENTS.md`

1. Line 87: rewrite the bullet "Breeds allowed (all breeds,
   specific group, specialty breed, mixed-breed inclusion)
   [ASSUMPTION: most events are all-breed; breed restrictions
   exist per the schema's BreedRestriction, GroupRestriction,
   BreedExclusion, GroupExclusion fields]" to "Mixed-breed
   inclusion via `events.mixed_breeds_allowed BOOL`. Allow-list
   and deny-list breed and breed-group restrictions are
   post-MVP; see PROJECT_STATUS.md Known-gaps."
2. Add a REQ-EVENT-* line for the two new event-level FKs:
   `trial_chair_user_id` and `event_secretary_user_id` per
   Deborah's Q5.
3. Add a REQ-CLUB-* line for `clubs.officers_json` per
   Deborah's Q6, with the yearly-elections cadence and the
   post-MVP-historical-preservation note.
4. Add a REQ-AWARD-* line (or extend an existing REQ-FEE-*) to
   reference the `combined_award_groups` reference table as
   the source of truth for combined-award discount eligibility,
   per Deborah's Q4.

### `docs/DATA_MODEL.md`

1. §2 events: add four columns (`mixed_breeds_allowed`,
   `trial_chair_user_id`, `event_secretary_user_id`,
   `dogs_per_hour_override`).
2. §2 trials: remove `trial_chairperson` row (or mark as DROPPED
   in PR 2d).
3. §2 trial_class_offerings: change "(pending, lands in PR 2c)"
   on `judges_book_pdf_object_key` to current; add
   `signed_scan_pdf_object_key` row.
4. §2 clubs: add `officers_json` row.
5. New §X for `combined_award_groups` and
   `combined_award_group_classes` (likely §3 or similar
   reference-data section).
6. ENUM table: ensure `armband_scheme` lists `per_series` (it
   already does at line 142; no change).
7. Bump "Last updated" / version.

### `docs/WORKFLOWS.md`

1. §10 email-template variables (per the 2026-04-23 research
   note's Implications-for-design-docs section): add
   `trial_chair_name`, `trial_chair_contact`,
   `event_secretary_name`, `event_secretary_contact` to the
   list of variables available to email templates.
2. §1.3 (club admin invites trial secretary): no edit required;
   workflow stays the same.

### `docs/DOMAIN_GLOSSARY.md`

1. Verify the existing "Trial Chair / Trial Chairperson" entry
   (line 224) and "Trial Secretary" entry (line 221) reflect
   Deborah's Q5 framing (the chair handles pre-trial
   arrangements + AKC approval + judge accommodations + steward
   recruitment + expense payments; the secretary handles
   on-the-day operations). The current entries are roughly
   correct but could be tightened with explicit examples.
2. Add or extend "Combined Award" entry referencing the
   `combined_award_groups` reference table and the
   per-trial-versus-title-progression distinction.
3. Add "All-American Dog" / "mixed-breed" entry referencing
   `events.mixed_breeds_allowed` (the existing Canine Partners
   entry at line 251 already covers AAD as a registration
   program; the breed-restriction angle is what's missing).

### `docs/ROADMAP.md`

1. Add a Phase 2+ line for breed-list / breed-group / breed-variety
   restrictions on events (post-PR 2d future work).

### `db/migrations/README.md`

1. Update the running list of migration groupings to include
   the PR 2d block.

### `db/seed/akc/README.md` (if it exists; if not, defer)

1. Add entries for `combined_award_groups.csv` and
   `combined_award_group_classes.csv` if README exists.

---

## 6. Open questions for Robare

These block CHECKPOINT 1 until answered.

1. **B1 RAE/RACH AKC regulation citations.** Author's best-guess
   citations are "Rally Regulations Ch. 1 §27 (RAE) and §28
   (RACH)". These need verification before the seed CSV lands.
   Options: (a) ask Deborah, (b) cite with a TODO marker, (c)
   leave NULL in the seed and add later. My recommendation: (b).

2. **B1 Obedience HTQ-equivalent.** AKC Rally has HTQ (Adv B + Ex
   B + Master). Does AKC Obedience have an analogous triple-class
   combined award? My research says no (Obedience uses HC for the
   Open B + Utility B pair, and OTCH points are accumulated over
   trials, not per-trial), so the seed only carries Obedience HC,
   not an Obedience HTQ. Confirm.

3. **B2 GFKC officer list at migration time.** Should the
   migration also bootstrap the GFKC officers_json from the
   premium PDF, or leave NULL for admin tooling to populate
   later? My recommendation: leave NULL.

4. **B3 two-column shape revision.** This revises the 2026-04-24
   working assumption. Confirm the revision lands as drafted, or
   point me at a constraint I missed.

5. **B4 sport-level override key namespace.** I am rejecting the
   sport_or_event key from the override JSONB and keeping
   class-code only. Confirm or push back.

6. **B5 trial_chairperson disposition.** I am recommending DROP
   (no consumers found). Confirm.

7. **B6 default value.** `BOOL NOT NULL DEFAULT TRUE`. Confirm.

8. **Date convention going forward.** Phase A flagged the
   CLAUDE.md / git-HEAD / DATA_MODEL.md disagreement. PR 2d's
   Decisions-log entries use 2026-04-25 (today's wall-clock per
   `date`). Confirm this is the convention, OR direct CHECKPOINT
   1 to use a different date scheme.

---

## 7. Outside-kickoff-scope items to flag

1. **`CLAUDE.md` `currentDate` line is stale.** Reads 2026-04-23,
   actual wall-clock is 2026-04-25. Standalone cleanup PR; not
   PR 2d work. Mentioned in the Phase A report.

2. **Missing `entry_status` ENUM in the kickoff.** The kickoff's
   reference to `entry_status` was a phantom; status is
   `entry_line_status` only. No follow-up needed; flagged in
   Phase A and acknowledged in Phase B kickoff.

3. **`rhtq` ENUM extension is a phantom.** Phase B newly
   surfaced. Known-gaps line 900-901 needs deletion in
   CHECKPOINT 1.

4. **`combined_award_groups` validation: cross-row sport
   matching.** Junction rows must reference
   `canonical_classes` rows whose `sport` matches the parent's
   `sport`. No DDL trigger enforces this; app-layer + seed
   validates. CHECKPOINT 1 should ensure the seed loader
   asserts this on each row, with a clear error message
   pointing to the offending CSV row.

5. **RLS tests for `combined_award_groups` and
   `combined_award_group_classes`.** New permissive-read
   reference tables. CHECKPOINT 3 (or wherever RLS tests live
   in PR 2d) should add a test verifying SELECT-as-tenant
   works and INSERT/UPDATE/DELETE-as-tenant fails. Pattern from
   `shared/tests/event_setup_rls.rs` or similar.

6. **The `xml_payload_object_key` precedent on
   `submission_records`.** PR 2b deliberately set aside that
   column for a future Agility XML submission workstream.
   Worth noting that the two-column judges-book shape sets a
   parallel precedent: PDF artifacts go on
   `trial_class_offerings`, electronic submission artifacts go
   on `submission_records`. The two tables are not collapsing.

7. **Combined-award-group seed loader rebuild.** If the seed
   loader currently lives in `workers/src/seed_loader/` and
   each reference table has its own loader function, the PR
   2d work adds two functions (one per new CSV). If the
   loader uses a generic-table-from-CSV pattern, the addition
   is config-only. Either way, scope is small but real.

8. **Index on `combined_award_group_classes.canonical_class_id`.**
   The fee-engine query "is this dog's entered set crossing
   classes in the same combined_award_group?" likely walks
   from canonical_class_id to combined_award_group_id. A
   non-unique index on `canonical_class_id` supports that
   query. Worth landing with the table create.

---

## 8. Summary of decisions

| Item | Decision |
|---|---|
| B1 | Parent + junction tables; `award_type` nullable on parent for RAE/RACH; permissive-read RLS; 5 seed rows. |
| B2 | Array of office records; serde-typed struct; NULL allowed; no DDL validation. |
| B3 | **Revise 2026-04-24 assumption.** Two columns: `judges_book_pdf_object_key` + `signed_scan_pdf_object_key`. |
| B4 | JSONB keyed by `canonical_classes.code` only. No sport-level key. App-layer validates on write; schedule generator falls through on miss. |
| B5 | Add both FKs to events with ON DELETE SET NULL, NULL allowed. **Drop** `trials.trial_chairperson`. |
| B6 | `BOOL NOT NULL DEFAULT TRUE`. Decisions-log entry drafted. |
| B7 | 9 migration files in sequence. No `-- no-transaction` needed. The `rhtq` ALTER TYPE is a no-op (already in ENUM). |
