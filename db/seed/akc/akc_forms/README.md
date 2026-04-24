# AKC reference forms

Blank AKC forms and templates that QTrial either generates filled
versions of or that secretaries reference during trial operations.
These are not seed data; they are reference artifacts kept alongside
the seed CSVs because both originate from AKC and both inform schema
and code decisions.

Filename convention:

    akc_<form_id>_<short_purpose>_<revision>.pdf

where `<revision>` is the AKC revision printed on the form footer as
MMYY (e.g. `1119` for an 11/19 revision), or YYYY when the form
prints only a year. When a form has no revision marker, the suffix
is just `blank` (the form itself is the template).

| File | What it is | When QTrial uses it |
|---|---|---|
| `akc_AEDSQ1_disqualification_for_attacking_person_1119.pdf` | Form AEDSQ1 (11/19): Disqualification by Judge/Evaluator for Attacking a Person, covering Conformation, Obedience, Agility, Rally, Tracking, and Temperament Test events. Judge completes Section A, exhibitor signs Section B, event secretary completes Section C. Must be faxed or emailed to `eventrecords@akc.org` within 72 hours of the incident. | REQ-SUB-004 (incident reporting). Surfaces in the MVP submission workflow when a dog is DQ'd in-ring with an attack-attempt or actual attack. |
| `akc_AEN999_official_entry_form_0523.pdf` | Form AEN999 (5/23) v1.0: the AKC Official Entry Form used across Conformation, Obedience, Rally, Agility, and Junior Showmanship. Carries the agreement text every exhibitor signs, the breed/class/variety/division fields, AKC/PAL registration identifiers, owner and handler details, emergency contact, junior handler block, and Owner/Handler Eligible checkbox. | REQ-ENTRY-001 through REQ-ENTRY-010 (entry form). The PDF/print variant QTrial produces for paper-entry support must match this layout. The emergency contact and Owner/Handler Eligible fields are captured on entries per Deborah's 2026-04-23 feedback items 6 and 7. |
| `akc_JEDTR1_emergency_procedures_and_disaster_plan_0822.pdf` | Form JEDTR1 (8/22): a two-part document. Page 1 is the AKC memorandum "Emergency Procedures at Dog Events" (nine policy points every club must satisfy). Pages 2+ are the Disaster and Emergency Plan template each club completes and keeps on file. | Club-configuration surface, post-MVP; referenced during club onboarding so secretaries have a prompt to attach or upload their completed plan. Not required at event submission but required to be on file. |
| `akc_agility_move_up_form_0912.pdf` | AKC Agility Trial Move-Up Form (09/12): exhibitor submits to the Trial Secretary, not to AKC. Lists Regular/Preferred x Standard/JWW/FAST x Open/Excellent/Master x jump-height divisions. | REQ-ENTRY-014 (class transfers, Agility post-MVP). The form directs the request path: the trial secretary is the endpoint, and QTrial's transfer workflow needs to receive the exhibitor-submitted form or its equivalent. |
| `akc_rally_hc_htq_tiebreaker_2018.pdf` | AKC Rally Highest Combined Score & Highest Scoring Triple tiebreaker form (2018), referencing Rally Regulations Chapter 1, Sections 31 and 32. Used only when a tie exists among top dogs for HC (Advanced B + Excellent B) or HTQ (Advanced B + Excellent B + Master). | REQ-SCORE-* (combined awards). Directly relevant to the combined_award_groups concept that moved from P2 to PR 2c scope per Deborah's Q4 answer on 2026-04-23. |
| `akc_rally_judges_book_blank_2017.pdf` | Blank template for the AKC Rally Judge's Book interior page. Columns: Armband Number, Breed of Dog, Time, Points Lost, Final Score. Placement block (1st-4th) and Time Started / Time Finished. | REQ-SUB-002 (judges book PDF generation for pre-trial printing). The printed pages are signed by the judge and physically mailed to AKC; per Deborah's 2026-04-23 correction, the judges book is not an electronic submission artifact. |
| `akc_rally_judges_book_cover_blank.pdf` | Blank template for the AKC Rally Judge's Book cover page. Procedure for Judges to Follow (5 items) and Judge's Certificate. No per-class scoring; this is the cover that precedes each class in the book bundle. | REQ-SUB-002 (judges book PDF generation). Paired with the interior-page template above. |
| `akc_rally_move_up_transfer_form_2017_11.pdf` | AKC Rally Trial: Transfer Form (November 1, 2017), citing Rally Regulations Chapter 1, Section Transfers. Lists Novice / Intermediate / Advanced / Excellent / Master class levels with A or B, plus Other, plus jump height. Submitted to the Trial Secretary or Superintendent at least 30 minutes prior to the start of the relevant trial. | REQ-ENTRY-014 (class transfers, Rally). The authoritative citation for transfer rules QTrial's transfer workflow references. Printed copy attached to the paperwork sent to AKC per Deborah's workflow. |

## Not listed here

`.DS_Store` metadata files are ignored by `.gitignore` and do not
belong in this directory's tracked set.
