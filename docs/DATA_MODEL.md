# QTrial - Data Model

**Status:** Draft v0.2 - integrates Deborah's Q&A (2026-04-19 / 2026-04-20) and confirmation-letter artifact review.
**Last updated:** 2026-04-20

---

## Modeling principles

1. **Every table has `tenant_id`** (pointing to `clubs.id`), with row-level security enforced at the database layer. Reference data (AKC canonical classes, breeds, title codes, countries, states) is shared across tenants and is explicitly marked.
2. **UUIDs for primary keys** everywhere, except for small reference tables where stable human-readable codes are more useful.
3. **Surrogate keys everywhere** - we never use business identifiers (AKC reg number, AKC event number, AKC judge number) as primary keys, because those are assigned by external authorities and can change.
4. **Soft delete** (`deleted_at timestamp`) rather than hard delete, for every table containing user data. Enables recovery and audit.
5. **`created_at`, `updated_at`, `created_by`, `updated_by`** on every mutable table.
6. **Explicit state machines** over boolean grab-bags. An `entry` has a `status` enum, not five separate booleans for `absent`, `excused`, `dq`, `withdrawn`, `wl`.
7. **Audit trails for anything that touches money or AKC reporting.** Dedicated audit tables for entries, payments, and results.
8. **Registry-neutral modeling where possible.** An event belongs to a `registry` (AKC, UKC, ...); classes, titles, and codes are scoped to a registry.

## Entity overview (high level)

```
Tenant (Club)
├── User → UserClubRole
├── EmailTemplate (per-club, per template_key)
├── Event
│   ├── EventDay
│   │   └── Trial
│   │       ├── TrialClassOffering (which canonical classes are offered at this trial)
│   │       │   └── JudgeAssignment
│   │       ├── ArmbandAssignment (per dog, per armband series)
│   │       ├── DogTrialJumpHeight (per dog at this trial)
│   │       └── TrialAward (HIT, HC, PHIT, PHC, RHC, HTQ)
│   └── EventConfig (settings, template overrides; includes dogs_per_hour_override JSONB)
├── Entry
│   ├── EntryLine (one per class the entry is competing in; joins ArmbandAssignment + DogTrialJumpHeight at render time)
│   │   └── EntryLineResult (score, time_started, time_finished, time_on_course, placement, points)
│   └── Payment
├── Dog
│   └── DogOwnership (junction: dog ↔ owner contact, is_primary flag)
├── Owner
├── Judge
├── MailingListEntry
└── SubmissionRecord (PDF package for MVP: marked catalog + judges books + Form JOVRY8)

Registry-scoped reference data (shared across tenants):
├── Registry (AKC, UKC, ...)
├── Breed, BreedVariety
├── BreedGroup
├── TitlePrefix, TitleSuffix (extended with source_organization, long_name, sport_scope_code)
├── CanonicalClass (the master class catalog per sport per registry; 75 rows for MVP Obedience+Rally)
├── JumpHeight (per sport per registry)
├── OtchPoints / OmPoints / RallyRachPoints (scoring lookup tables; single current ruleset)
├── SportTimeDefault (default minutes/dog per sport or Agility event-type)
└── ExerciseDefinition (per class per registry)

Global shared data:
├── Country
└── State
```

## Tables in detail

All column types expressed in PostgreSQL syntax. All timestamp columns use `TIMESTAMPTZ`. All monetary amounts use `NUMERIC(10, 2)` (dollars and cents, up to ~$100M).

### 1. Tenancy and identity

#### `clubs`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `display_name` | TEXT | e.g., "Glens Falls Kennel Club" |
| `abbreviation` | TEXT | e.g., "GFKC" |
| `akc_club_number` | TEXT | nullable, 4 chars when present |
| `ukc_club_number` | TEXT | nullable, reserved for P2 |
| `akc_status` | ENUM | `member`, `licensed`, or `none` |
| `logo_object_key` | TEXT | S3 key for the logo image |
| `primary_contact_user_id` | UUID | FK to `users` |
| `billing_status` | ENUM | `active`, `comped`, `suspended` |
| `created_at` / `updated_at` / `deleted_at` | TIMESTAMPTZ | |

#### `users`

A user account, independent of any club.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `keycloak_sub` | TEXT | the `sub` claim from our Keycloak-issued JWT |
| `email` | CITEXT | unique |
| `display_name` | TEXT | |
| `first_name`, `last_name` | TEXT | |
| `phone` | TEXT | |
| `address_line1`, `address_line2`, `city`, `state`, `postal_code`, `country_code` | TEXT | |
| `junior_handler_number` | TEXT | nullable |
| `birthdate` | DATE | nullable; used for senior/junior eligibility |
| `created_at` / `updated_at` / `deleted_at` | TIMESTAMPTZ | |

#### `user_club_roles`

Junction table granting users roles at clubs.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `user_id` | UUID | FK |
| `club_id` | UUID | FK |
| `role` | ENUM | `club_admin`, `trial_secretary`, `judge`, `exhibitor` |
| `granted_by_user_id` | UUID | FK |
| `granted_at` / `revoked_at` | TIMESTAMPTZ | |

Platform administrators are tracked via a separate `platform_admins` table, not via this junction.

#### `platform_admins`

Platform-level administrators with authority across tenants. Kept separate from `user_club_roles` so that platform admin grants are not visible to club-scoped queries or affected by tenant RLS.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `user_id` | UUID | FK to `users`, unique |
| `granted_by_user_id` | UUID | FK to `users`, nullable (first admin is self-granted during bootstrap) |
| `granted_at` | TIMESTAMPTZ | |
| `revoked_at` | TIMESTAMPTZ | nullable |

Not tenant-scoped. No RLS policy. Access is via platform-admin authorization check at the API layer.

### 2. Events, days, trials

#### `events`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `name` | TEXT | human-readable event name |
| `cluster_name` | TEXT | nullable, for events part of a cluster |
| `venue_name` | TEXT | |
| `venue_address_line1` / `line2` / `city` / `state` / `postal_code` / `country_code` | TEXT | |
| `entry_opens_at` / `entry_closes_at` | TIMESTAMPTZ | |
| `moveup_deadline_at` | TIMESTAMPTZ | |
| `armband_scheme` | ENUM | `per_trial`, `per_event`, `per_day`, `per_class`, `per_series` |
| `armband_start_number` | INT | |
| `armband_interval` | INT | |
| `catalog_fee` | NUMERIC(10,2) | |
| `dogs_per_hour_override` | JSONB | nullable; shape `{"obedience": 3.5, "rally-choice": 4.3}`; absence means fall back to `sport_time_defaults` |
| `waitlist_accepted` | BOOL | whether this event accepts waitlist entries |
| `status` | ENUM | `draft`, `open`, `closed`, `in_progress`, `complete`, `archived` |
| `registry_id` | UUID | FK to `registries` |
| `created_at` / `updated_at` / `deleted_at` | TIMESTAMPTZ | |

#### `event_days`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `event_id` | UUID | FK |
| `day_number` | INT | 1, 2, 3... within the event |
| `date` | DATE | |
| `start_time` | TIME | planned event start on this day |

#### `trials`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `event_day_id` | UUID | FK |
| `trial_number` | INT | 1 or 2 within the day (typically AM/PM) |
| `sport` | ENUM | `obedience`, `rally`, `agility`, etc. |
| `akc_event_number` | TEXT | registry-assigned event number |
| `trial_chairperson` | TEXT | |
| `start_time` | TIME | |
| `entry_limit` | INT | nullable; no limit if null |
| `first_class_fee` | NUMERIC(10,2) | |
| `additional_class_fee` | NUMERIC(10,2) | |
| `nonregular_class_fee` | NUMERIC(10,2) | |
| `nonregular_second_class_fee` | NUMERIC(10,2) | |
| `brace_fee` | NUMERIC(10,2) | |
| `team_fee` | NUMERIC(10,2) | |
| `rally_pairs_fee` | NUMERIC(10,2) | |
| `rally_team_fee` | NUMERIC(10,2) | |
| `first_class_fee_jr` | NUMERIC(10,2) | junior handler rate |
| `additional_class_fee_jr` | NUMERIC(10,2) | |
| `status` | ENUM | `draft`, `open`, `closed`, `running`, `complete` |

#### `trial_class_offerings`

Which canonical classes are being offered at a given trial. This is the join between trials and the canonical class catalog.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `trial_id` | UUID | FK |
| `canonical_class_id` | UUID | FK to `canonical_classes` |
| `ring_number` | INT | |
| `class_limit` | INT | nullable |
| `scheduled_start_time` | TIME | |
| `running_order_strategy` | ENUM | `short_to_tall`, `tall_to_short`, `random`, `manual` |
| `jump_start_height` | INT | for the short-to-tall order |

#### `judge_assignments`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `trial_class_offering_id` | UUID | FK |
| `judge_id` | UUID | FK to `judges` |
| `is_co_judge` | BOOL | for shared assignments |

#### `trial_awards`

One row per award per trial.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `trial_id` | UUID | FK |
| `award_type` | ENUM | `hit`, `hc`, `phit`, `phc`, `rhc`, `htq` |
| `winning_entry_line_id` | UUID | FK to `entry_lines`, nullable until scored |
| `winning_armband` | INT | denormalized for catalog printing |
| `winning_score` | NUMERIC(5,1) | |
| `notes` | TEXT | |

### 3. People directories

#### `judges`

Note: judges are club-scoped in the sense that clubs track them in their judge directory, but the judge identity itself (AKC number) is unique across clubs. We could model this either as a club-scoped contact record or a shared judge registry. **[DECISION PENDING]** - for MVP, treating as club-scoped contact record with AKC number as a soft-unique constraint.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root (see note above) |
| `user_id` | UUID | FK, nullable - present if the judge has an QTrial account |
| `last_name`, `first_name` | TEXT | |
| `akc_judge_number` | TEXT | |
| `address_line1`, `city`, `state`, `postal_code`, `country_code` | TEXT | |
| `phone`, `cell`, `email` | TEXT | |
| `is_provisional` | BOOL | |

#### `owners`

The registered owner of a dog. Distinct from the `exhibitor` (the user who's entering the dog), though the same person usually.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `user_id` | UUID | FK, nullable - present if the owner is also a user |
| `last_name`, `first_name` | TEXT | |
| `address_line1`, `city`, `state`, `postal_code`, `country_code` | TEXT | |
| `phone`, `email` | TEXT | |
| `mailing_list_optin` | BOOL | |
| `prefers_email_contact` | BOOL | |
| `is_club_member` | BOOL | |
| `active` | BOOL | |

### 4. Dogs

#### `dogs`

Primary ownership is now modeled via `dog_ownerships` (see §3). The `owner_id` column is retained transitionally as a convenience pointer to the primary owner; it is populated from `dog_ownerships` where `is_primary = true`.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `owner_id` | UUID | FK to `owners`; convenience pointer to primary owner |
| `breed_id` | UUID | FK |
| `breed_variety_id` | UUID | FK, nullable |
| `breed_division` | TEXT | free text, nullable |
| `call_name` | TEXT | |
| `sex` | ENUM | `male`, `female`, `male_neutered`, `female_spayed` |
| `registered_name` | TEXT | verbatim as submitted; may contain embedded title tokens |
| `registry_id` | UUID | AKC, UKC, etc. |
| `registration_type` | ENUM | `akc_purebred`, `pal`, `canine_partners`, `fss`, `misc` |
| `registration_number` | TEXT | verbatim; leading zeros preserved; formats vary by type (e.g., `DN71750607`, `SS22371305`, `SR95697401`, `PAL282370`, `MB11524001`) |
| `registration_country_code` | TEXT | ISO country code |
| `birthdate` | DATE | |
| `breeder` | TEXT | |
| `sire_registered_name` | TEXT | free text; titles embedded; parsed at display time |
| `dam_registered_name` | TEXT | free text; titles embedded; parsed at display time |
| `parsed_name_root` | TEXT | registered name with recognized titles stripped; populated by the name parser |
| `parsed_prefix_titles` | TEXT[] | recognized prefix title codes extracted from the registered name (e.g., `{CH,GCH}`) |
| `parsed_suffix_titles` | TEXT[] | recognized suffix title codes extracted from the registered name (e.g., `{CD,RE,JH}`) |
| `unparsed_title_tokens` | TEXT[] | title-like tokens that did not match the catalog; preserved verbatim for trial-secretary review |
| `inactive` | BOOL | |

Jump height is NOT on this table. It is modeled per-(dog, trial) in `dog_trial_jump_heights` (see §4.1). The dog's optional AKC jump-height card, when present, provides a default at entry-time but does not constrain what the handler may elect at a specific trial.

Sire and dam names are stored as free text because their titles are not queried often enough to justify denormalization. The catalog renderer parses them at display time against the same title catalog used for the dog itself.

#### `dog_titles`

A dog can have many earned titles. We store each as a row to enable calculating "how many legs toward CDX" without parsing a free-text field.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `dog_id` | UUID | FK |
| `title_code` | TEXT | e.g., "CD", "CDX", "OTCH" |
| `title_category` | ENUM | `prefix` or `suffix` |
| `earned_at` | DATE | nullable |
| `source` | ENUM | `owner_entered`, `registry_verified`, `earned_in_qtrial`, `parsed_from_registered_name` |

#### `dog_ownerships`

Models the many-to-many relationship between dogs and owner contacts. Exactly one ownership row per dog is designated as primary.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `dog_id` | UUID | FK to `dogs` |
| `owner_contact_id` | UUID | FK to `owners` (or `contacts` once consolidated) |
| `is_primary` | BOOL | exactly one primary owner per dog, enforced by partial unique index |

```sql
CREATE UNIQUE INDEX idx_one_primary_owner_per_dog
    ON dog_ownerships (dog_id) WHERE is_primary = true;
CREATE UNIQUE INDEX idx_unique_owner_per_dog
    ON dog_ownerships (dog_id, owner_contact_id);
```

Rationale: Per Deborah's Q2 answer (2026-04-20), co-owners are common in the real world and go on the dog record, not the entry. This replaces the legacy `dogs.co_owners_text` free-text field. The registered handler for a given entry is tracked on `entry_lines.handler_contact_id` (see §5).

#### `dog_trial_jump_heights`

Jump height per (dog, trial). A dog running multiple jumping classes at the same trial jumps the same height in all of them. Rally Choice entries do not consult this table.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `dog_id` | UUID | FK |
| `trial_id` | UUID | FK |
| `jump_height_inches` | INT | CHECK (jump_height_inches IN (4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36)) |
| `was_judge_measured` | BOOL | true when a judge overrode the submitted height in-ring |
| `judge_measured_at` | TIMESTAMPTZ | nullable |
| `judge_measured_by_contact_id` | UUID | FK to `owners`, nullable. Column name preserved for a potential future contacts-table consolidation; today the target is `owners`. |

```sql
UNIQUE (dog_id, trial_id)
```

Rationale: Per Deborah's Q1 answer (2026-04-20), jump height never changes between classes on the same day for the same dog, except in the rare case (approximately once per trial-secretary career) where a judge doubts the submitted height and measures the dog in-ring. That override must update the height for all of the dog's remaining entries at the current trial, which is cleanest if the height lives on a single row per (dog, trial) rather than on each entry line.

Entry lines inherit jump height via join at render time. See REQ-ENTRY-013 and REQ-ENTRY-015.

#### `dog_sport_participation`

Which sports a dog actively competes in (used for mailing list filtering).

| Column | Type | Notes |
|---|---|---|
| `dog_id` | UUID | FK |
| `sport` | ENUM | `obedience`, `rally`, ... |
| `active` | BOOL | |

PK is composite `(dog_id, sport)`.

### 5. Entries and results

#### `entries`

One row per (exhibitor × dog × event). The top-level entry record.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `event_id` | UUID | FK |
| `dog_id` | UUID | FK |
| `exhibitor_user_id` | UUID | FK to `users` (the person who submitted the entry) |
| `owner_id` | UUID | FK (may differ from the exhibitor; typically equals `dogs.owner_id` i.e., the primary owner) |
| `submitted_at` | TIMESTAMPTZ | |
| `payment_method` | ENUM | `card`, `check`, `money_order`, `cash`, `paypal`, `coupon`, `discount` |
| `total_owed` | NUMERIC(10,2) | |
| `total_paid` | NUMERIC(10,2) | |
| `catalog_number` | INT | assigned at catalog generation |
| `entry_confirmation_pdf_object_key` | TEXT | S3 key for the generated per-dog entry confirmation PDF (REQ-ENTRY-010) |
| `confirmation_email_sent_at` | TIMESTAMPTZ | `entry_confirmation` template |
| `post_closing_email_sent_at` | TIMESTAMPTZ | `post_closing_reminder` template; populated ~1 week pre-trial |
| `results_email_sent_at` | TIMESTAMPTZ | |
| `notes` | TEXT | secretary-facing |

Handler identity (which may differ from owner) lives on `entry_lines`, not on `entries`, because in edge cases a handler assignment could differ per class (e.g., one class handled by a junior, another by the owner). See `entry_lines.handler_contact_id`.

#### `entry_lines`

One row per (entry × class). Each line is one class the dog is entered in at this event.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `entry_id` | UUID | FK |
| `trial_class_offering_id` | UUID | FK - ties to a specific class at a specific trial |
| `armband_assignment_id` | UUID | FK to `armband_assignments`; multiple entry lines for the same dog in the same series share one assignment |
| `handler_contact_id` | UUID | NOT NULL; defaults to the dog's primary owner on creation; may differ for junior handlers |
| `junior_handler_akc_number` | TEXT | nullable; populated only when a junior handles the dog (REQ-ENTRY-016) |
| `also_entered_in` | TEXT | nullable - denormalized list of other classes for catalog convenience |
| `team_id` | UUID | FK to `teams`, nullable |
| `brace_partner_entry_line_id` | UUID | self-FK for brace partner |
| `status` | ENUM | `pending_payment`, `active`, `waitlist`, `scratched`, `withdrawn`, `transferred`, `moved_up`, `absent`, `excused`, `dq` |
| `status_changed_at` | TIMESTAMPTZ | |
| `status_reason` | TEXT | free text, for excused/DQ/scratched |
| `waitlist_position` | INT | nullable, populated when on waitlist |
| `is_alternate` | BOOL | for team alternates |
| `is_veteran` | BOOL | |
| `running_order_position` | INT | nullable until the running order is finalized |
| `random_order_number` | INT | seeds the randomized portion of running order |

Notes:

- Jump height is NOT on this table. It is joined from `dog_trial_jump_heights` via `(entries.dog_id, trials.id)`. Rally Choice entries do not attempt that lookup.
- Armband is NOT stored as a raw integer on this row. It is joined from `armband_assignments` via `armband_assignment_id`. Multiple entry lines for the same dog in the same armband series share one assignment row, so they naturally share the armband value.

#### `armband_assignments`

One row per (dog, trial, armband series). See REQ-ENTRY-012 and the DOMAIN_GLOSSARY "Armband series" entry.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `dog_id` | UUID | FK |
| `trial_id` | UUID | FK |
| `armband_series` | TEXT | e.g., `"500"` or `"Advanced B / Excellent B / Master"` |
| `armband_number` | INT | |

```sql
UNIQUE (trial_id, armband_series, armband_number)  -- no two dogs share an armband within a series
UNIQUE (dog_id, trial_id, armband_series)          -- one assignment per (dog, trial, series)
```

The armband-series values come from the trial's class configuration. On entry processing, QTrial:

1. Determines which armband series each offered class belongs to (e.g., Advanced B, Excellent B, and Master all map to the "500 series" under the default AKC convention).
2. Creates one `armband_assignment` per (dog, trial, series) the dog is entered in.
3. Links each entry line to the appropriate assignment via `entry_lines.armband_assignment_id`.

#### `entry_line_results`

One row per entry_line after the class is run.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `entry_line_id` | UUID | FK, unique |
| `score` | NUMERIC(5,1) | nullable if Abs/Exc/DQ |
| `time_started` | TIMESTAMPTZ | nullable; when judging began for this dog |
| `time_finished` | TIMESTAMPTZ | nullable; when judging ended |
| `time_on_course` | INTERVAL | nullable; computed from `time_finished - time_started` or entered directly; used for tie-breaking |
| `time_seconds` | NUMERIC(7,2) | nullable, legacy / convenience for renderers that want a raw number of seconds |
| `qualifying` | ENUM | `q`, `nq`, `na` (for nonregular without Q concept) |
| `placement` | INT | 1-4 or NULL |
| `otch_points` | INT | for Obedience qualifying dogs |
| `om_points` | NUMERIC(5,1) | |
| `rach_points` | INT | for Rally qualifying dogs |
| `entered_at` | TIMESTAMPTZ | when the result was recorded |
| `entered_by_user_id` | UUID | FK |

Tie-breaking rule: placement within a class is computed as `ORDER BY score DESC, time_on_course ASC`. This matches the Nov 2025 marked catalog (Rally Excellent B page: armbands 512 and 524 both scored 100, placed 1st and 2nd respectively on time). Both Obedience and Rally track time; the judges book cover has "Time Started" and "Time Finished" fields that the judge fills in during the run.

#### `teams`

For Team Obedience, Rally Team, etc.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `event_id` | UUID | FK |
| `team_name` | TEXT | |
| `team_type` | ENUM | `obedience_team`, `rally_team_novice`, `rally_team_advanced`, `rally_team_excellent`, `rally_t_challenge_team` |

Entry lines are associated with a team via `entry_lines.team_id`.

### 6. Payments

#### `payments`

A payment against an entry.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `entry_id` | UUID | FK |
| `amount` | NUMERIC(10,2) | |
| `method` | ENUM | `card`, `check`, `money_order`, `cash`, `paypal`, `coupon`, `discount` |
| `reference` | TEXT | check number, Stripe charge ID, PayPal transaction ID, etc. |
| `paid_at` | DATE | |
| `recorded_at` | TIMESTAMPTZ | |
| `recorded_by_user_id` | UUID | FK |
| `deposited` | BOOL | for checks - has the club deposited it? |
| `deposit_date` | DATE | |
| `note` | TEXT | |

#### `refunds`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `payment_id` | UUID | FK |
| `amount` | NUMERIC(10,2) | |
| `reason` | ENUM | `bitch_in_season`, `scratched_before_closing`, `withdrawn`, `duplicate_entry`, `other` |
| `reason_detail` | TEXT | |
| `refunded_at` | TIMESTAMPTZ | |

### 7. Waitlist

Waitlist is modeled via `entry_lines.status = 'waitlist'` with a `waitlist_position` ordering. A separate `waitlist_events` audit log records transitions.

### 8. Registry reference data (shared across tenants)

#### `registries`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `code` | TEXT | "AKC", "UKC" |
| `name` | TEXT | |

Seeded with AKC as the first entry; UKC added when we extend support.

#### `breed_groups`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `group_number` | INT | 1-11 per AKC |
| `name` | TEXT | |
| `registration_prefix_codes` | TEXT[] | the letter codes that begin AKC reg numbers for breeds in this group (e.g., SR, SN, SS for Sporting) |

Seeded from `tblAKCGroups`.

#### `breeds`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `breed_group_id` | UUID | FK |
| `name` | TEXT | |
| `abbreviation` | TEXT | |
| `is_giant` | BOOL | |
| `is_three_quarters` | BOOL | |
| `has_variety` | BOOL | |
| `has_division` | BOOL | |

Seeded from `tblBreeds`.

#### `breed_varieties`

For breeds that compete by variety (Poodles, Dachshunds, Cocker Spaniels, etc.)

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `breed_id` | UUID | FK |
| `name` | TEXT | |
| `display_order` | INT | |

#### `title_prefixes` / `title_suffixes`

Extended in v0.2 to support non-AKC issuing bodies and human-readable expansions.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK, nullable when sourced from a non-registry body (e.g., Barn Hunt Association) |
| `source_organization` | TEXT | NOT NULL DEFAULT `'AKC'` - issuing body |
| `code` | TEXT | "CH", "GCH", "OTCH", "CD", "CDX", ... |
| `long_name` | TEXT | nullable; human-readable expansion (e.g., "Companion Dog") |
| `sport_scope_code` | TEXT | nullable; single-letter or multi-letter code: "O" (Obedience), "R" (Rally), "T" (Tracking), "F" (Field), "H" (Herding), etc. |
| `sport_scope_description` | TEXT | nullable; human-readable scope |
| `display_order` | INT | |
| `earning_rules` | JSONB | computed rules for when QTrial can infer title earned (e.g., "3 Qs in Novice with 2+ judges → CD") |

MVP seed scope (per Deborah's Q2 2026-04-20):

- All 49 AKC prefix titles
- All 244 AKC suffix titles
- 5 legacy compound suffix titles (for name-parser compatibility with historical data)
- 10 Barn Hunt titles (`source_organization = 'Barn Hunt Association'`)
- 81 other non-AKC titles are preserved in the seed package for post-MVP; they are NOT seeded in MVP

The schema supports these without further migration. The name parser (REQ-NAME-001) looks up tokens against this catalog and records unknowns in `dogs.unparsed_title_tokens` rather than auto-creating rows here.

Seeded from `tblAKCTitlesPrefix` and `tblAKCTitlesSuffix`, augmented per the v2.2 seed package.

#### `otch_points`

Lookup table mapping class and entry count to OTCH point values for the top four placements. Single current ruleset - no version columns (per Deborah's Q3 2026-04-20: QTrial trusts the handler's word for titles; AKC keeps the authoritative records).

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `class_name` | TEXT | `'Open B'` or `'Utility B'` |
| `entries_min` | INT | |
| `entries_max` | INT | |
| `first_place_points` | INT | |
| `second_place_points` | INT | |
| `third_place_points` | INT | |
| `fourth_place_points` | INT | |

Seeded from `otch_points.csv` (23 rows).

#### `om_points`

Lookup table mapping Obedience score to Obedience Master points. Single ruleset.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `score` | NUMERIC(4,1) | 190.0 to 200.0 in half-point increments |
| `om_points` | INT | |

Seeded from `om_points.csv` (21 rows).

#### `rally_rach_points`

Lookup table mapping Rally score to RACH points.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `score` | INT | 91 to 100 |
| `rach_points` | INT | |

Seeded from `rally_rach_points.csv` (10 rows).

#### `sport_time_defaults`

Platform-level pacing defaults per sport; per-event overrides live on `events.dogs_per_hour_override`. Per Deborah's Q5 (2026-04-20), Obedience and Rally default to 3.0 min/dog for schedule estimation, but real pacing varies class-by-class (Nov 2025 data: Rally Choice ~4.3 min/dog, Rally Master ~3.5, Rally Excellent B ~3.1), which motivates the override.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `sport_or_event` | TEXT | UNIQUE - `'Obedience'`, `'Rally'`, `'Agility-Standard'`, `'Agility-JWW'`, `'Agility-FAST'`, `'Agility-ISC'` |
| `minutes_per_dog` | NUMERIC(3,1) | |
| `class_change_seconds` | INT | |
| `event_change_seconds` | INT | |

Seeded from `trial_time_calculations.csv` (6 rows).

#### `email_templates`

Per-kennel-club templates for system-generated emails (REQ-EMAIL-001). Per Deborah's Q4 (2026-04-20), different clubs have different voices and boilerplate; templates must be club-configurable.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `template_key` | TEXT | `'entry_confirmation'`, `'post_closing_reminder'`, `'cancellation_notice'`, `'refund_confirmation'` |
| `subject_template` | TEXT | with `{{variable_name}}` placeholders |
| `body_template` | TEXT | with `{{variable_name}}` placeholders |
| `updated_at` | TIMESTAMPTZ | |

```sql
UNIQUE (club_id, template_key)
```

Template variable syntax is simple `{{variable_name}}` substitution for MVP. Jinja-style conditionals are post-MVP. Variables available per template are documented in `WORKFLOWS.md` §10. Default templates are seeded on club creation; clubs override via the settings UI.

#### `canonical_classes`

The master catalog of all classes supported per registry per sport. Locked-in v0.2 schema (75 rows seeded for MVP: Obedience + Rally).

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `sport` | ENUM | `obedience`, `rally` for MVP; others later |
| `name` | TEXT | display name, e.g., `'Rally Novice A'` |
| `code` | TEXT | canonical internal code, e.g., `akc_rally_novice_a` |
| `class_type` | ENUM | `Regular`, `Optional Titling`, `Preferred`, `Nonregular` |
| `akc_class_code` | INT | nullable; AKC's internal numeric code matching `tblkAKCObedClassInfo.ClassCode` for migration |
| `moveup_target_id` | UUID | nullable self-FK; preserves the "Transfer to X" pseudo-class pattern Lab Tested Databases uses for class-to-class transfer modeling |
| `is_sanctioned` | BOOL | NOT NULL; `false` for classes that don't earn AKC titles (e.g., Random Reward) |
| `is_displayed` | BOOL | NOT NULL DEFAULT true; `false` for "Transfer to X" pseudo-classes that exist only as move-up targets |
| `has_jump` | BOOL | NOT NULL |
| `has_multiple_entries` | BOOL | NOT NULL; for teams/braces/pairs |
| `max_total_score` | INT | NOT NULL; 200 for Obedience, 100 for Rally |
| `min_qualifying_score` | INT | 170 for Obedience, 70 for Rally |
| `default_dogs_per_hour` | INT | NOT NULL; overridable at the event level via `events.dogs_per_hour_override` |
| `has_broad_jump` | BOOL | NOT NULL DEFAULT false |
| `has_walkthrough` | BOOL | NOT NULL DEFAULT false |
| `qualifies_for_title_code` | TEXT | what suffix title this class can earn legs toward |

Seeded from `canonical_classes.csv` (75 rows) - Lab Tested Databases's `tblkAKCObedClassInfo` normalized and extended to cover Rally.

The `is_sanctioned = false` flag is used for nonregular classes like Random Reward that appear on the trial schedule but do not earn AKC titles. The `is_displayed = false` flag hides "Transfer to X" pseudo-classes from class-offering UIs while preserving them as valid `moveup_target_id` references.

#### `jump_heights`

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `sport` | ENUM | |
| `height_inches` | NUMERIC(4,1) | |
| `akc_secondary_class_code` | TEXT | e.g., `4INCHES`, used for AKC XML submission |

Seeded from `tblAKCxmlJumpHeights`.

#### `exercises`

The exercises within Obedience classes (Heel on Leash, Figure Eight, etc.)

Seeded from `tblAKCObedienceExercises`.

### 9. AKC submission records

#### `submission_records`

Each attempt at submitting results to AKC. For MVP (Obedience and Rally), submission is PDF-based: the marked catalog PDF, the judges books (carbon-copy pink pages), and the populated AKC Report of Rally Trial (Form JOVRY8) / Obedience equivalent. XML submission is Agility-only and deferred post-MVP.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `trial_id` | UUID | FK |
| `submission_type` | ENUM | `pdf_package` (MVP: marked catalog + judges books + form JOVRY8), `xml` (post-MVP Agility), `csv` (fallback) |
| `marked_catalog_object_key` | TEXT | S3 key; populated for `pdf_package` |
| `judges_book_object_keys` | TEXT[] | S3 keys, one per class; populated for `pdf_package` |
| `form_jovry8_object_key` | TEXT | S3 key for the populated Form JOVRY8 (or Obedience equivalent); populated for `pdf_package` |
| `xml_payload_object_key` | TEXT | S3 key; populated for `xml` submissions only |
| `akc_destination_email` | TEXT | default `rallyresults@akc.org` for Rally; Obedience equivalent; override per-event permitted |
| `fee_total` | NUMERIC(10,2) | the AKC recording fee total computed for this submission (REQ-SUB-005) |
| `submitted_at` | TIMESTAMPTZ | |
| `submitted_by_user_id` | UUID | FK |
| `status` | ENUM | `draft`, `generated`, `submitted`, `accepted`, `rejected` |
| `akc_response` | JSONB | response details if we ever get API acceptance vs. manual upload |
| `rejection_reason` | TEXT | |

### 10. Audit and logging

#### `audit_log`

A general-purpose audit table. Entries are written by the backend for sensitive operations.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant scope |
| `actor_user_id` | UUID | FK, may be null for system actions |
| `entity_type` | TEXT | "entry_line", "payment", "submission_record", etc. |
| `entity_id` | UUID | |
| `action` | TEXT | "status_changed", "payment_recorded", ... |
| `diff` | JSONB | before/after snapshot |
| `occurred_at` | TIMESTAMPTZ | |

## Multi-tenancy enforcement

1. **Every tenant-scoped table has `club_id`**, with a NOT NULL constraint and a FK to `clubs.id`.
2. **Row-level security policies** are written on every tenant-scoped table. The policy restricts SELECT, INSERT, UPDATE, DELETE to rows where `club_id = current_setting('app.current_club_id')::uuid`.
3. **Before each request**, the backend sets `app.current_club_id` (and `app.current_user_id`) via `SET LOCAL` within the transaction.
4. **Reference data (`registries`, `breeds`, `canonical_classes`, etc.) is readable by all tenants** via policies that allow SELECT regardless of `app.current_club_id`.
5. **Platform admin override**: platform admins set a special role that bypasses RLS for support scenarios. Those sessions are logged separately.

## Migration considerations from Deborah's Access schema

Key mappings from `ObedienceData.mde` → QTrial:

| Access table | QTrial table(s) | Notes |
|---|---|---|
| `tblEventData` | `events` | `Organization` maps to `registries`; `ConfLetter`/`WaitList` memo templates migrated to `event_templates` (P2 - for MVP use defaults) |
| `tblEventDayData` | `event_days` + `trials` | Each row in EventDayData is really a day-trial combo and maps to one `trials` row plus its parent `event_day` |
| `tblAKCObedienceClassInfo` | `trial_class_offerings` | **Filter by `Offered=true`** - the Access model pre-allocates all possible class slots; we only store offered ones |
| `tblAKCObedienceLimits` | `trial_class_offerings.class_limit` or `trials.entry_limit` | Depending on whether limit is per-class or per-trial |
| `tblkAKCObedClassInfo` | `canonical_classes` | Seed data, not per-tenant |
| `tblAKCObedienceEntryInfo` | `entries` | One-to-one, mostly renames |
| `tblAKCObedienceEntries` | `entry_lines` + `entry_line_results` | Split: entry data + scoring into separate tables |
| `tblDogData` | `dogs` + `dog_titles` | Parse `AKCPrefix` and `AKCSuffix` text fields into `dog_titles` rows |
| `tblOwnerData` | `owners` | |
| `tblJudges` | `judges` | |
| `tblPayments` | `payments` | |
| `tblRevenue` | Not directly - computed as a view over `entries.total_paid` and related | Access denormalized this; we'll compute |
| `tblClubInfo` | `clubs` | Per-club record; this Access file had three `clubs` rows but it's really one club with two license statuses (GFKC Member and GFKC Licensed). QTrial models this as a single `clubs.akc_status`. |
| `tblSecretaryInfo` | `users` with `user_club_roles` | The secretary is a user; their role is a club-scoped grant |
| `tblPremiumMailingList` | `mailing_list_entries` (not yet detailed above - defined in REQUIREMENTS §15) | |
| `tblAKCxmlResults` | `submission_records` + per-record generation logic | For MVP (Obedience/Rally) submission is PDF-based, so this table is largely informational for migration. For post-MVP Agility support, QTrial will generate XML on demand. |
| `tblAKCxmlDefaults` | Configuration loaded from environment + registry metadata | Post-MVP Agility |
| `tblAKCxmlClassNames` | A future `akc_xml_class_codes` table (Agility post-MVP) | Not in MVP schema; Obedience/Rally submission does not need XML class codes |
| `tblAKCTitlesPrefix` / `tblAKCTitlesSuffix` | `title_prefixes` / `title_suffixes` | Seed data |
| `tblBreeds`, `tblBreedVariety`, `tblAKCGroups` | `breeds`, `breed_varieties`, `breed_groups` | Seed data |
| `tblCountries`, `tblStates` | `countries`, `states` | Global shared |
| `tblOrganization` | `registries` | |
| `tblPaymentMethod` | Enumerated as ENUM type; no table needed | |
| `tblTrialTimeCalculation` | `canonical_classes.dogs_per_hour_default` or per-event override | |
| `ttbl*` (temp tables) | Not migrated - these are Access-specific work tables | |

## Open questions / pending decisions

1. **Judge record scoping**: club-scoped contact records or a shared cross-club judge registry? MVP: club-scoped. Revisit if it becomes friction.
2. **Title earning automation**: should QTrial auto-advance titles when a dog accumulates qualifying legs? MVP: no - let the owner self-manage. P2: yes, based on `canonical_classes.qualifies_for_title_code` + `title_prefixes.earning_rules`.
3. **Dog dedup across tenants**: if the same dog is entered at Club A and Club B via QTrial, should they be the same dog record? MVP: separate records per tenant to simplify RLS. Consider a global `registered_dogs` shared table in P2.

## Resolved decisions (2026-04-20)

These were open in v0.1 and are now settled:

- **Co-owner structure** (was §4): modeled as `dog_ownerships` junction with `is_primary` flag (partial unique index enforces exactly one primary). Replaces the legacy `dogs.co_owners_text` free-text column. Per Deborah's Q2.
- **Confirmation email template storage**: per-club templates keyed by `(club_id, template_key)` in the new `email_templates` table. Per Deborah's Q4.
- **AKC submission format for Obedience/Rally**: PDF-based (marked catalog + judges books + Form JOVRY8). XML submission is Agility-only and deferred post-MVP. Per Deborah's Q4 (2026-04-19).
- **Jump height scope**: per-(dog, trial), modeled in `dog_trial_jump_heights`. NOT per-entry. Per Deborah's Q1.
- **OTCH/OM/RACH points versioning**: single current ruleset, no `effective_from`/`effective_until` columns. QTrial trusts the handler for title claims; AKC maintains authoritative records. Per Deborah's Q3.
- **Title catalog scope for MVP**: 49 prefix + 244 AKC suffix + 5 legacy compound + 10 Barn Hunt titles. 81 other non-AKC titles available in the seed package but not seeded in MVP. Per Deborah's Q2.

## Pending artifacts

- `Judges_Book_Sat.pdf` body pages → may surface additional judges-book columns needed for REQ-SUB-002.
- Screen grabs of entry and scoring screens → will reveal any fields Deborah relies on that we have not modeled yet.
- AKC Agility XML schema (current) → needed when QTrial adds Agility support post-MVP. Will drive the future `akc_xml_class_codes` table.
