# AKC Reference Data Seeds (v2.2)

**Status:** Tracked in repo at `db/seed/akc/` as of PR 2a (2026-04-23)
**Last updated:** 2026-04-23 (location moved from `db_seed_akc/` to `db/seed/akc/`; `jumps.csv` renamed to `jump_heights.csv` to match target table name)
**Primary source:** `ObedienceData.accde` from Deborah Pruyn's Obedience Solution by Lab Tested Databases
**Secondary source:** AKC's public Titles & Abbreviations page (authoritative for title catalog)
**Extraction date:** 2026-04-20

---

## What changed in v2.2

Deborah's Q&A answers (2026-04-20) resolved four outstanding architecture decisions. This version integrates those answers into the seed data and documentation.

### Summary of her answers

1. **Non-AKC titles (Q2):** "Not really except things like barn hunt." → Narrowed MVP scope to barn hunt titles only
2. **Historical scoring (Q3):** "Whatever is current because the AKC usually keeps track and all we deal with is ribbons for the title and we take the handler's word for that." → Dropped versioned pre-2019 scoring tables
3. **AKC submission format (Q4):** Not XML. PDF package: marked catalog + judges books + AKC Report of Rally Trial (Form JOVRY8) emailed to rallyresults@akc.org → Moved XML tables to post-MVP
4. **Trial time defaults (Q5):** 3 minutes per dog rule-of-thumb, confirmed by her actual Nov 2025 schedule → Added Obedience and Rally defaults

### Concrete data deltas vs v2.1

| File | v2.1 | v2.2 | Change |
|------|------|------|--------|
| title_suffixes.csv | 249 | 259 | +10 barn hunt titles |
| otch_points.csv | - (was 2 files) | 23 | Collapsed to single current-rules file |
| otch_points_pre_20190801.csv | 23 | DELETED | Historical rules out of scope |
| rally_rach_points.csv | 10 (3 cols) | 10 (2 cols) | Dropped pre_20190201 column |
| trial_time_calculations.csv | 4 | 6 | Added Obedience and Rally |
| akc_xml_class_codes.csv | main dir | post_mvp/ | Moved (XML submission is post-MVP) |
| akc_xml_jump_heights.csv | main dir | post_mvp/ | Moved |
| akc_overrides_added.csv | 175 | 185 | +10 barn hunt audit entries |

---

## Authoritative source hierarchy

When current AKC published data conflicts with Deborah's database, **AKC wins**. Deborah's database is used as:

1. **Title legacy_id mapping** for migration continuity when importing from existing Obedience Solution databases
2. **Breed catalog, class catalog, scoring tables** where Lab Tested Databases stays current
3. **Discovery mechanism** for what fields/relationships exist
4. **Operational truth** for things AKC doesn't publish (time estimates, display ordering, internal codes)

The 185 rows in `akc_overrides_added.csv` document exactly which titles were added from AKC beyond what Deborah's database contains.

---

## File inventory

### Core reference data (8 files)

| File | Target table | Row count | Notes |
|------|--------------|-----------|-------|
| `breed_groups.csv` | `breed_groups` | 11 | AKC groups 1-11 |
| `breeds.csv` | `breeds` | 288 | Includes 35 post-v1 additions |
| `breed_varieties.csv` | `breed_varieties` | 19 | Includes Poodle varieties |
| `title_prefixes.csv` | `title_prefixes` | 49 | AKC-authoritative |
| `title_suffixes.csv` | `title_suffixes` | 259 | AKC-authoritative (249 AKC + 5 legacy compounds + 10 Barn Hunt - but actual count is 259 = 244 AKC + 5 legacy + 10 barn hunt) |
| `countries.csv` | `countries` | 216 | ISO 3166-1 standard |
| `legacy_akc_country_codes.csv` | (migration ref) | 144 | AKC legacy → ISO mapping |
| `states.csv` | `states` | 63 | US states + Canadian provinces |

### Class catalog and scoring (7 files)

| File | Target table | Row count | Notes |
|------|--------------|-----------|-------|
| `canonical_classes.csv` | `canonical_classes` | 75 | All classes incl. Rally + Obedience + Random Reward |
| `om_points.csv` | `om_points` | 21 | OM (Obedience Master) points by qualifying score |
| `otch_points.csv` | `otch_points` | 23 | Current AKC rules only |
| `rally_rach_points.csv` | `rally_rach_points` | 10 | Current AKC rules only |
| `obedience_exercises.csv` | `obedience_exercises` | 20 | Master exercise list |
| `obedience_class_exercises.csv` | `obedience_class_exercises` | 36 | Junction: exercises per class |
| `jump_heights.csv` | `jump_heights` | 21 | Jump height metadata |

### Non-AKC titles (2 files - data preserved, MVP seeds only barn hunt)

| File | Target table | Row count | Notes |
|------|--------------|-----------|-------|
| `non_akc_title_suffixes.csv` | (NOT seeded for MVP) | 81 | Preserved for future expansion; per Q2 only barn hunt is MVP-scope |
| `non_akc_title_suffix_breed_restrictions.csv` | (NOT seeded for MVP) | 63 | Junction table; same reasoning |

Per Deborah's Q2 answer, barn hunt titles (RATN, RATO, RATS, RATM, RATCh, RATChX, CZ8B/S/G/P) are folded directly into `title_suffixes.csv` with `source_organization="Barn Hunt Association"` and `sport_scope_code="BH"`. The 81-row non_akc file is preserved for post-MVP.

### Operational defaults (1 file)

| File | Target table | Row count | Notes |
|------|--------------|-----------|-------|
| `trial_time_calculations.csv` | `sport_time_defaults` | 6 | Agility (Standard/JWW/FAST/ISC) + Obedience + Rally |

### Audit (1 file)

| File | Purpose | Row count | Notes |
|------|---------|-----------|-------|
| `akc_overrides_added.csv` | Audit record | 185 | AKC titles added beyond Deborah's catalog (incl. barn hunt) |

### Post-MVP (subdirectory)

| File | Purpose | Notes |
|------|---------|-------|
| `post_mvp/akc_xml_class_codes.csv` | XML submission codes | Agility only; not needed for MVP (PDF-based submission) |
| `post_mvp/akc_xml_jump_heights.csv` | XML submission codes | Agility only; post-MVP |
| `post_mvp/README.md` | Explanation | Documents why these are deferred |

---

## Sport scope codes

The `sport_scope_code` column uses single-letter codes (multi-letter for some categories):

| Code | Sport |
|------|-------|
| O | Obedience |
| R | Rally |
| T | Tracking |
| A | Agility Standard |
| J | Agility Jumpers With Weaves |
| P | Preferred Agility Standard |
| Q | Preferred Agility Jumpers |
| Y | Agility FAST |
| Z | Preferred Agility FAST |
| U | Time 2 Beat |
| S | Preferred Time 2 Beat |
| F | Field / Hunt Test |
| E | Earthdog |
| H | Herding |
| C | Coursing |
| L | Lure Coursing |
| M | Coonhound |
| N | Conformation |
| V | Versatility (multi-sport titles like VCD, DC, TC) |
| X | Companion (CGC, CGCA, CGCU, STR, FDC, THD variants) |
| W | Scent Work |
| K | Trick Dog |
| D | Therapy Dog |
| I | FIT Dog |
| B | Fetch |
| G | Virtual (pandemic-era remote titles) |
| **BH** | **Barn Hunt (AKC-recognized via Barn Hunt Association)** |
| FCAT | Fast CAT |
| TT | Temperament Test |
| (empty) | Compound/legacy (e.g., UDTD) |

---

## AKC submission workflow (for downstream reference)

Per Deborah's Q4 answer and the Trial_Summary_report.pdf artifact:

1. **During trial:** Trial secretary enters scores in Obedience Solution software
2. **After trial:** Software generates marked catalog as PDF, with scores annotated on each entry
3. **Submission package contents:**
   - Marked catalog (PDF)
   - Original judges books (paper; pink carbon copy to AKC)
   - Completed AKC Report of Rally Trial form (JOVRY8) or obedience equivalent
   - Payment ($3.50 first entry / $3.00 additional per dog, plus $10 event secretary fee after 12 trials/year)
4. **Delivery:** Mail to AKC Event Operations (PO Box 900051, Raleigh NC 27675-9051) OR email to **rallyresults@akc.org**

The XML-based electronic submission infrastructure in Deborah's database covers Agility only. It is irrelevant for MVP.

---

## How this was produced

1. **Source files:** `ObedienceData.accde` (10.4 MB) uploaded 2026-04-20
2. **AKC title page:** https://www.akc.org/sports/titles-and-abbreviations/ fetched 2026-04-20
3. **Deborah's Q&A:** Answers received 2026-04-20, integrated into v2.2
4. **PDF artifacts reviewed:** Nov 2025 marked catalog, judges book cover, steward board, judging schedule, trial summary report (AKC Form JOVRY8)
5. **Cleaning script:** `clean_extracts.py` in this directory
6. **Workflow:**
   - Raw tables dumped from Access via `mdb-export`
   - AKC-authoritative title list embedded in `clean_extracts.py`
   - Cleaning script merges AKC + Deborah's IDs for migration continuity
   - Override record generated automatically

---

## Refresh procedures

### When Deborah sends an updated database

```bash
cp /path/to/NewObedienceData.accde /home/claude/obedience_data_new.accde

for table in tblAKCGroups tblBreeds tblBreedVariety \
             tblAKCTitlesPrefix tblAKCTitlesSuffix \
             tblCountries tblStates \
             tblkAKCObedClassInfo tblAKCRallyRACHPoints \
             tblAKCObedienceOMPoints tblAKCObedienceOtchPoints \
             tblAKCObedienceClassExercises \
             tblAKCObedienceExercises tblAKCObedienceJumps \
             tblAKCxmlClassNames tblAKCxmlJumpHeights \
             tblAKCTitlesSuffixNonAKC tblAKCTitlesSuffixNonAKCBreed \
             tblTrialTimeCalculation; do
    mdb-export /home/claude/obedience_data_new.accde "$table" \
        > /home/claude/db_seed_akc_v2/raw_${table}.csv
done

cd /home/claude/db_seed_akc_v2 && python3 clean_extracts.py
```

Diff against committed CSVs; commit updates.

**Note:** We no longer extract `tblAKCObedienceOtchPoints20190801` (historical OTCH brackets) since Q3 confirmed we only need current rules.

### When AKC updates their Titles page

Edit `AKC_PREFIX_TITLES` and `AKC_SUFFIX_TITLES` constants in `clean_extracts.py`. Re-run to regenerate CSVs. The cleaning script automatically preserves legacy_id mapping from Deborah's data and assigns new IDs in the 1000+ range.

---

## See also

- `DATA_MODEL_CHANGES.md` - schema changes needed in DATA_MODEL.md
- `akc_overrides_added.csv` - complete list of AKC titles added beyond Deborah's catalog
- `post_mvp/README.md` - explanation of deferred XML submission data
