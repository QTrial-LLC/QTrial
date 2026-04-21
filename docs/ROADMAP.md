# QTrial - Roadmap

**Status:** Draft v0.2
**Last updated:** 2026-04-20

---

## Philosophy

QTrial is being built by one engineer (Robare) on evenings and weekends, with Claude Code doing most of the typing. The constraint on the roadmap is not feature ambition - it's time, cognitive load, and the ability to ship working software without breaking Deborah's trust on her first real trial.

Three principles guide the roadmap:

1. **Deborah runs a real trial on MVP.** The release that counts is the one she actually uses. Every date targets a specific trial on her calendar.
2. **No feature ships without a test-the-waters moment.** Every major feature gets used by Deborah in a shadow mode (alongside her existing tools) before replacing them.
3. **The data model is correct from day one; the features grow into it.** We spend the extra time to model things right so we don't have to refactor under pressure.

---

## Phase 0: Foundation (Weeks 1-4)

Goal: a running project with the right shape, no functionality yet.

- Repository initialized with the agreed structure
- CI/CD pipeline running: build, test, lint, type-check on PRs
- Database schema for reference data (AKC canonical classes, breeds, titles, groups, jump heights) in migrations
- Seed data loaded and verified against the extracted data from Deborah's `ObedienceData.mde`
- Keycloak deployed to staging with QTrial as a registered client
- Rust API skeleton with health check, OIDC authentication middleware, and Postgres connectivity with row-level security template
- Next.js frontend skeleton with sign-in flow working end-to-end
- One smoke test covering the full sign-in flow

**Deliverable:** a user can sign up, sign in, and see an empty dashboard.

---

## Phase 1: Club and Event Setup (Weeks 5-8)

Goal: Deborah can set up a Glens Falls Kennel Club trial in QTrial.

- Club creation and configuration
- User management (invite, grant roles)
- Event creation with days and trials
- Trial class offerings (selecting from the canonical class catalog)
- Judge directory and judge assignments
- Fee configuration
- Basic premium list generation (HTML preview and PDF export)

**Deliverable:** Deborah can configure her fall 2026 Rally/Obedience trial, end to end, in QTrial (alongside her existing Access setup). The premium list PDF is acceptable to her eye.

**Checkpoint:** Deborah reviews the event setup UX. Any friction identified here is fixed before Phase 2.

---

## Phase 2: Entry Flow (Weeks 9-14)

Goal: exhibitors can enter the trial online.

- Exhibitor account creation
- Dog directory with full AKC-compliant registered name handling
- Online entry flow (single dog, single class → single dog, multi-class → multi-trial)
- Stripe Connect integration with the club's bank account
- Entry fee calculation with first-entry / additional-entry logic
- Confirmation email generation and dispatch
- Waitlist mechanics
- Paper entry entry-by-secretary workflow
- Basic entry management (cancel, change jump height)

**Deliverable:** the entry page is live for Deborah's fall 2026 trial. Entries are accepted online. Exhibitors receive confirmation emails. Deborah can process paper entries.

**Checkpoint:** Deborah processes 5-10 real entries in shadow mode (entering them both in QTrial and her Access tool). Discrepancies are investigated and fixed.

---

## Phase 3: Pre-Trial Paperwork (Weeks 15-18)

Goal: QTrial produces all the paper Deborah needs to run a trial.

- Catalog generation (PDF, with proper AKC registered name formatting)
- Judge's book generation (PDF, per judge, per class)
- Scribe sheet generation (Obedience)
- Armband assignment and card generation
- Running order generation with jump-height sorting
- Ring-assignment view

**Deliverable:** Deborah can print all pre-trial paperwork from QTrial. Content is verified against the equivalent documents from her Access system.

**Checkpoint:** Deborah compares QTrial-generated documents to her old ones. Formatting issues, missing fields, and AKC compliance gaps are identified and closed.

---

## Phase 4: Move-ups, Transfers, and Day-of (Weeks 19-22)

Goal: QTrial handles the post-closing and trial-day workflows.

- Move-up request, approval, and propagation to all downstream documents
- Transfer workflow (A↔B)
- Bitch-in-season refund workflow with Stripe integration
- Day-of change tools (absent, excused, DQ, scratched)
- Quick-entry scoring UI

**Deliverable:** QTrial handles every realistic scenario Deborah has encountered in her last 5 trials.

**Checkpoint:** Deborah and Robare run through a mock trial-day operations scenario, including move-ups, a bitch-in-season refund, and a judge excusal. Friction is measured and fixed.

---

## Phase 5: Scoring and Awards (Weeks 23-26)

Goal: QTrial handles scoring and awards computation.

- Score entry for Obedience (exercise-by-exercise) and Rally (total + time)
- Validation (score ranges, placement eligibility)
- Placement calculation
- HIT, HC, PHIT, PHC, RHC, HTQ computation
- Run-off identification and recording
- OTCH point and OM point calculation
- Marked catalog generation (catalog + results)

**Deliverable:** QTrial can take a fully-scored trial and produce the marked catalog.

**Checkpoint:** Deborah re-scores one of her historical trials from 2025 into QTrial and compares results to her Access system. Any discrepancies are investigated.

---

## Phase 6: AKC Submission (Weeks 27-30)

Goal: QTrial generates the PDF-based AKC results submission package for Obedience and Rally.

Per Deborah's Q4 answer (2026-04-19), AKC Obedience and Rally submission in 2026 is PDF-based, not XML. The MVP scope is therefore:

- Marked catalog PDF generation (REQ-SUB-001) - the catalog with final scores annotated per entry
- Judges book PDF generation with cover sheet and per-class score/time pages (REQ-SUB-002)
- Form JOVRY8 PDF form-fill (REQ-SUB-003) for Rally; Obedience equivalent form for Obedience
- Fee calculation per REQ-SUB-005: $3.50 first entry + $3.00 additional per dog per trial + $10 secretary fee after 12 trials/year
- "Draft AKC email" helper that composes an email with the three PDFs attached to `rallyresults@akc.org` (or the Obedience equivalent) - but does not auto-send; the secretary reviews and sends themselves for MVP (REQ-SUB-004)
- Submission record tracking and archiving

XML-based electronic submission is Agility-only and deferred until post-MVP. AKC's Agility XML schema will drive that work when Agility support lands.

**Deliverable:** QTrial produces the three-artifact PDF submission package matching the format AKC expects today (reference PDFs: `Nov_2025_Sat_Marked_Catalog.pdf`, `Judges_Book_Cover_Sat.pdf`, `Trial_Summary_report.pdf`).

**Checkpoint:** Deborah submits her next trial using the QTrial-generated package. If AKC accepts it, we are done for MVP. If there are format issues, we iterate on the PDF rendering.

---

## Phase 7: Migration Tooling (Weeks 31-32)

Goal: Deborah's historical data from Access is visible in QTrial.

- Access file reader (server-side; accepts `.mde`/`.mdb` upload)
- Parsing of all relevant tables
- Dedup logic for dogs and owners
- Preview report
- Import execution
- Historical event view (read-only)

**Deliverable:** Deborah's Glens Falls data (and any other club she works with) is migrated to QTrial. Historical entries are queryable.

**Checkpoint:** Deborah validates the migration by looking up dogs and owners she knows.

---

## Phase 8: Polish, Testing, and First Real Trial (Weeks 33-36)

Goal: MVP is ready for a real trial.

- Load testing for entry-open rush (simulate 100 simultaneous entries)
- Usability polish based on cumulative feedback
- Admin dashboard for the club
- Mailing list functionality (MVP level)
- Financial reporting
- Observability: error monitoring, dashboards, alerts
- Backup and disaster recovery verified
- Documentation: user-facing help articles for the workflows Deborah will use

**Deliverable:** Deborah runs her spring 2027 trial (or whatever the target trial is) on QTrial as her primary system. Her Access system is the backup.

**Checkpoint:** Post-trial retrospective. What worked, what didn't, what needs to change for the next trial.

---

## Post-MVP (Phase 9 and beyond)

Not sequenced yet, but on the horizon. These are grouped by theme, not timeline.

### Queue Management (Phase 9)

Real-time run order for trial day. Exhibitors see their position. Ring stewards check off dogs. Notifications at "2 out" and "you're up." Optional tablet-based scoring for judges.

This is the competitive parity feature with Secreterrier, EagerDog, DogShow.com, and AgilityGate. Not required for MVP but is the obvious next major investment.

### Agility, Scent Work, Conformation, FastCAT, Barn Hunt

Each sport requires:
- Canonical class definitions
- Sport-specific scoring rules
- Sport-specific judge's book formats
- Sport-specific catalog conventions
- AKC XML class codes and submission variations

Agility and Scent Work are the highest-priority additions given their volume in the AKC trial market.

### UKC and other registries

The data model is registry-aware from day one. Adding UKC is mostly reference-data work (classes, titles, submission formats) plus UI copy changes.

### Exhibitor features

- Dog title progress tracking with automatic leg-counting
- Trial history and achievements
- Cross-club profile (one dog, seen across multiple clubs)
- Jump height card management

### Club features

- Sponsorship and trophy management
- Club member vs non-member fee differentials
- Volunteer tracking and credits
- Revenue and budget reports

### Mobile apps

A native app is probably not needed even in P2 (the web experience can be excellent). A progressive web app for trial-day offline capability is probably the first native-adjacent investment.

### Platform operations

- Self-service tenant provisioning
- Billing and subscription management
- Fraud detection on exhibitor entries
- Abuse handling
- Club analytics dashboard

---

## Milestones that matter

- **First Deborah-usable release:** end of Phase 1, ~week 8. She can set up a trial. Not usable end-to-end, but the foundation she can give feedback on.
- **First entries released release:** end of Phase 2, ~week 14. Exhibitors can enter. Shadow mode.
- **First pre-trial paperwork release:** end of Phase 3, ~week 18. Documents are AKC-acceptable.
- **First full shadow trial:** end of Phase 5, ~week 26. Deborah runs a trial on QTrial alongside her Access system.
- **First real trial on QTrial:** end of Phase 8, ~week 36 (approximately 9 months from kickoff, which targets a spring 2027 trial).

This is aggressive for a single evening-and-weekend engineer. Slippage is expected. The discipline is to let features fall back without moving the first-real-trial date.

---

## Explicit deferrals (to prevent scope creep during implementation)

These are things we want but will actively refuse to build during the MVP phase:

- Queue management / trial-day run order app
- Native mobile apps
- Judge-in-ring tablet scoring
- Live streaming or video
- Pedigree database
- Training log / class management
- Inter-exhibitor messaging
- Sweepstakes, Futurities, Junior Showmanship class management
- Agility, Scent Work, FastCAT, Barn Hunt (sport-specific logic, though the data model supports them)
- UKC, CKC, and other registries
- Multi-language support
- White-labeling
- Marketplace-style public event discovery

---

## Risk register

### Risks to the schedule

- **AKC PDF form format changes.** If AKC updates Form JOVRY8 or its Obedience equivalent between now and MVP, the form-fill layer needs to be updated. Mitigation: treat the form-fill as data-driven templates, re-verified before each release.
- **PDF rendering fidelity.** The marked catalog and judges books have specific layout expectations that AKC reviewers are used to. Mitigation: compare side-by-side with Deborah's reference PDFs during Phase 6.
- **Deborah's availability.** She's a volunteer. If she's overwhelmed with her real trial secretary work, feedback cycles slow down.
- **Stripe Connect approval delays.** Some business categories take weeks to approve. We should start the Stripe application early.
- **Multi-tenancy bugs.** RLS bypasses or bugs that leak data between clubs are catastrophic. Heavy testing required.
- **Robare's AmpThink situation.** If Robare's day-job situation escalates (per the Miami Freedom Park / Yankee Stadium conflict context), QTrial evenings become rarer.

### Risks to the product

- **AKC rejects a submission for formatting reasons.** Mitigation: the secretary retains the ability to hand-edit and resubmit, and Deborah reviews every Phase 6 artifact against her reference PDFs before QTrial is used for a real trial.
- **Deborah's workflow reveals requirements we haven't imagined.** Likely. Mitigation: narrative walkthrough + shadow mode early and often.
- **A competitor beats us to market** with a feature Deborah strongly prefers. Mitigation: the competitor stack being surveyed doesn't look like it will leap ahead on the architectural fundamentals QTrial is investing in.

### Risks to the business

- **No one else wants QTrial.** Deborah's happiness is the primary goal; broader adoption is a secondary aspiration. If QTrial serves only her and a handful of other clubs, that's still a success.
- **IP entanglement with AmpThink.** Robare has an active IP protection engagement. QTrial should be developed on Robare's personal hardware and infrastructure, with no overlap with AmpThink resources.
- **Legal issues around migration from competitor products.** Mitigated by using migration only for direct customer onboarding, not marketing.

---

## What a "done MVP" does not look like

A few anti-patterns to avoid:

- Announcing a launch before Deborah has used it for a real trial
- Adding paying customers beyond Deborah's circle before the product is proven
- Spending time on marketing before the product is feature-complete
- Optimizing for scale before anyone is using it
- Writing blog posts before shipping code

The MVP success criterion is narrow: Deborah says "I used QTrial for my trial, and it worked." Everything else is later.
