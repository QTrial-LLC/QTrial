# QTrial - Data Model

**Status:** Draft v0.1 - **provisional**; derived from Deborah's `ObedienceData.mde` schema with modernization and multi-tenancy layered in.
**Last updated:** 2026-04-19

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
├── Event
│   ├── EventDay
│   │   └── Trial
│   │       ├── TrialClassOffering (which canonical classes are offered at this trial)
│   │       │   └── JudgeAssignment
│   │       └── TrialAward (HIT, HC, PHIT, PHC, RHC, HTQ)
│   └── EventConfig (settings, template overrides)
├── Entry
│   ├── EntryLine (one per class the entry is competing in)
│   │   └── EntryLineResult (score, placement, points)
│   └── Payment
├── Dog
├── Owner
├── Judge
├── MailingListEntry
└── SubmissionRecord (AKC submission artifacts)

Registry-scoped reference data (shared across tenants):
├── Registry (AKC, UKC, ...)
├── Breed, BreedVariety
├── BreedGroup
├── TitlePrefix, TitleSuffix
├── CanonicalClass (the master class catalog per sport per registry)
├── JumpHeight (per sport per registry)
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
| `armband_scheme` | ENUM | `per_trial`, `per_event`, `per_day`, `per_class` |
| `armband_start_number` | INT | |
| `armband_interval` | INT | |
| `catalog_fee` | NUMERIC(10,2) | |
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

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `owner_id` | UUID | FK |
| `breed_id` | UUID | FK |
| `breed_variety_id` | UUID | FK, nullable |
| `breed_division` | TEXT | free text, nullable |
| `call_name` | TEXT | |
| `sex` | ENUM | `male`, `female`, `male_neutered`, `female_spayed` |
| `registered_name` | TEXT | |
| `registry_id` | UUID | AKC, UKC, etc. |
| `registration_number` | TEXT | |
| `registration_country_code` | TEXT | ISO country code |
| `birthdate` | DATE | |
| `breeder` | TEXT | |
| `sire_prefix_titles` | TEXT | |
| `sire_registered_name` | TEXT | |
| `sire_suffix_titles` | TEXT | |
| `dam_prefix_titles` | TEXT | |
| `dam_registered_name` | TEXT | |
| `dam_suffix_titles` | TEXT | |
| `co_owners_text` | TEXT | free-text for catalog |
| `jump_height_measured` | NUMERIC(4,1) | optional; in inches |
| `has_jump_height_card` | BOOL | |
| `inactive` | BOOL | |

#### `dog_titles`

A dog can have many earned titles. We store each as a row to enable calculating "how many legs toward CDX" without parsing a free-text field.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `dog_id` | UUID | FK |
| `title_code` | TEXT | e.g., "CD", "CDX", "OTCH" |
| `title_category` | ENUM | `prefix` or `suffix` |
| `earned_at` | DATE | nullable |
| `source` | ENUM | `owner_entered`, `registry_verified`, `earned_in_qtrial` |

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
| `owner_id` | UUID | FK (may differ from the exhibitor) |
| `handler_name` | TEXT | free text, may differ from exhibitor |
| `junior_handler_number` | TEXT | nullable |
| `is_senior_handler` | BOOL | |
| `submitted_at` | TIMESTAMPTZ | |
| `payment_method` | ENUM | `card`, `check`, `money_order`, `cash`, `paypal`, `coupon`, `discount` |
| `total_owed` | NUMERIC(10,2) | |
| `total_paid` | NUMERIC(10,2) | |
| `catalog_number` | INT | assigned at catalog generation |
| `confirmation_email_sent_at` | TIMESTAMPTZ | |
| `results_email_sent_at` | TIMESTAMPTZ | |
| `notes` | TEXT | secretary-facing |

#### `entry_lines`

One row per (entry × class). Each line is one class the dog is entered in at this event.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `entry_id` | UUID | FK |
| `trial_class_offering_id` | UUID | FK - ties to a specific class at a specific trial |
| `jump_height_inches` | NUMERIC(4,1) | nullable for non-jump classes |
| `armband` | INT | assigned by the system |
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

#### `entry_line_results`

One row per entry_line after the class is run.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `entry_line_id` | UUID | FK, unique |
| `score` | NUMERIC(5,1) | nullable if Abs/Exc/DQ |
| `time_seconds` | NUMERIC(7,2) | nullable, for timed classes |
| `qualifying` | ENUM | `q`, `nq`, `na` (for nonregular without Q concept) |
| `placement` | INT | 1-4 or NULL |
| `otch_points` | INT | for Obedience qualifying dogs |
| `om_points` | NUMERIC(5,1) | |
| `entered_at` | TIMESTAMPTZ | when the result was recorded |
| `entered_by_user_id` | UUID | FK |

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

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `code` | TEXT | "CH", "GCH", "OTCH", "CD", "CDX", ... |
| `sport_scope` | TEXT | for suffixes: "O" (Obedience), "R" (Rally), "T" (Tracking), "F" (Field), "H" (Herding), etc. |
| `display_order` | INT | |
| `earning_rules` | JSONB | computed rules for when QTrial can infer title earned (e.g., "3 Qs in Novice with 2+ judges → CD") |

Seeded from `tblAKCTitlesPrefix` and `tblAKCTitlesSuffix`.

#### `canonical_classes`

The master catalog of all classes supported per registry per sport.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `registry_id` | UUID | FK |
| `sport` | ENUM | |
| `code` | TEXT | canonical internal code (e.g., `akc_obed_novice_a`) |
| `display_name` | TEXT | "Novice A" |
| `class_type` | ENUM | `regular`, `optional_titling`, `preferred`, `nonregular` |
| `legacy_class_code` | INT | matches `tblkAKCObedClassInfo.ClassCode` for migration |
| `is_sanctioned` | BOOL | |
| `has_jumps` | BOOL | |
| `has_multiple_entries_per_dog` | BOOL | for teams/braces |
| `total_score` | INT | 200 for Obedience, 100 for Rally, etc. |
| `dogs_per_hour_default` | INT | |
| `has_broad_jump` | BOOL | |
| `has_walkthrough` | BOOL | |
| `qualifies_for_title_code` | TEXT | what suffix title this class can earn legs toward |
| `min_qualifying_score` | INT | 170 for Obedience, 70 for Rally |
| `parent_class_id` | UUID | self-FK for "Transfer to X" records - this is how the Access schema handles transfer/move-up targets |

Seeded from `tblkAKCObedClassInfo`.

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

Each attempt at submitting results to AKC.

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `club_id` | UUID | tenant root |
| `trial_id` | UUID | FK |
| `format` | ENUM | `xml`, `csv`, `pdf_report` |
| `payload_object_key` | TEXT | S3 key for the generated file |
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
| `tblAKCxmlResults` | `submission_records` + per-record generation logic | The Access schema stores the last submission as a flat table; QTrial generates XML on demand |
| `tblAKCxmlDefaults` | Configuration loaded from environment + registry metadata | |
| `tblAKCxmlClassNames` | `canonical_classes.akc_xml_primary_code` column (to be added) | Critical for XML submission |
| `tblAKCTitlesPrefix` / `tblAKCTitlesSuffix` | `title_prefixes` / `title_suffixes` | Seed data |
| `tblBreeds`, `tblBreedVariety`, `tblAKCGroups` | `breeds`, `breed_varieties`, `breed_groups` | Seed data |
| `tblCountries`, `tblStates` | `countries`, `states` | Global shared |
| `tblOrganization` | `registries` | |
| `tblPaymentMethod` | Enumerated as ENUM type; no table needed | |
| `tblTrialTimeCalculation` | `canonical_classes.dogs_per_hour_default` or per-event override | |
| `ttbl*` (temp tables) | Not migrated - these are Access-specific work tables | |

## Open questions / pending decisions

1. **Judge record scoping**: club-scoped contact records or a shared cross-club judge registry? MVP: club-scoped. Revisit if it becomes friction.
2. **Title earning automation**: should QTrial auto-advance titles when a dog accumulates qualifying legs? MVP: no - let the owner self-manage. P2: yes, based on `canonical_classes.qualifying_for_title_code` + `title_prefixes.earning_rules`.
3. **Dog dedup across tenants**: if the same dog is entered at Club A and Club B via QTrial, should they be the same dog record? MVP: separate records per tenant to simplify RLS. Consider a global `registered_dogs` shared table in P2.
4. **How are co-owners structured for title-earning purposes?** Current schema uses free-text `CoOwners` on the dog. For catalog rendering this is fine; for title verification it may not be. **[PENDING]**
5. **Confirmation email template storage**: per-event memo field (current Access) or per-club template with per-event override (proposed)? Proposed is cleaner.

## Pending artifacts

- PDF examples of catalog, judge's book, scribe sheet → will refine print-formatting related columns and may surface missing fields.
- AKC XSD schema file → will produce `canonical_classes.akc_xml_primary_code` values for Obedience/Rally (we have them for Agility already).
- Screen grabs → will reveal any fields Deborah relies on that we haven't modeled yet.
