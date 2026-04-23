# Post-MVP Reference Data

These CSVs are NOT seeded for QTrial MVP. They will become relevant when
QTrial expands beyond AKC Obedience and Rally.

## Why deferred

Per Deborah's 2026-04-20 Q&A:
- AKC Obedience and Rally submission is **PDF-based**, not XML-based
  (AKC Form JOVRY8 for Rally; equivalent form for Obedience)
- The marked catalog + judges books + signed report is mailed or emailed
  to rallyresults@akc.org (or obedience equivalent)
- There is no XML submission endpoint for MVP sports

These XML code tables cover AKC Agility only (as extracted from Deborah's
database). They are retained here for the future when QTrial adds Agility
support, at which point the XML submission format becomes relevant.

## Files

- `akc_xml_class_codes.csv` (45 rows) - Agility class → AKC XML code mapping
- `akc_xml_jump_heights.csv` (7 rows) - Agility jump height → XML code mapping

## Also missing from here

Obedience and Rally XML codes were never found in Lab Tested Databases's
catalog. If AKC ever introduces XML-based submission for Obedience and
Rally, those codes will need to be sourced directly from AKC at that time.
