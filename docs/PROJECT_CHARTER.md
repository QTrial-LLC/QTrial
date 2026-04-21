# QTrial - Project Charter

**Status:** Draft v0.1 (foundational; revise as the product definition firms up)
**Last updated:** 2026-04-19
**Owner:** Robare Pruyn
**Primary stakeholder:** Deborah Pruyn (trial secretary; first real user)

---

## What QTrial is

QTrial is a cloud-hosted, multi-tenant software platform for managing AKC and (eventually) UKC dog sport trials. It replaces the volunteer trial secretary's patchwork of desktop Access databases, email clients, Word templates, PDF forms, and Excel spreadsheets with a single integrated system that handles the complete lifecycle of a trial - from the club's application to AKC through the final electronic results submission.

QTrial is built for the volunteer running the trial, not for the national registry or the big show superintendent. The target user is a person who loves the sport, has full-time responsibilities elsewhere, and wants the software to stay out of the way.

## Why QTrial exists

Running a dog sport trial is a volunteer undertaking that currently requires mastery of software tooling that was modern in 2003. The dominant products are Microsoft Access applications that require Windows, require a separate file per club, have no real multi-user support, and force trial secretaries to email PDFs and copy data between systems. The newer cloud-based competitors are better but still leave gaps around the full lifecycle - most handle online entry or trial management but not both, and few integrate electronic results submission to AKC.

The volunteer running the trial deserves software that respects their time, their domain knowledge, and their club's money.

## Who QTrial is for

### Primary users (in order of workflow centrality)

1. **Trial secretary** - the operator. Builds the trial, processes entries, generates all paperwork, scores the event, submits results. This is the persona whose workflow must be flawless.
2. **Exhibitor** - the dog owner/handler. Enters dogs in trials, pays fees, receives confirmations, reviews run orders, gets results.
3. **Club administrator** - the organizing body. Creates the club in the system, invites secretaries, views financial reports, manages the club's profile.
4. **Judge** - receives assignments, views their judge's book, enters scores (or has them entered for them).
5. **Trial-day operations (steward, gate steward)** - works the day-of logistics.

### Secondary personas

- **Platform administrator** - Anthropic-style staff role for QTrial itself (handling cross-tenant issues, abuse, migrations).
- **AKC** - not a direct user, but a structural stakeholder whose data formats and regulations we must respect.

## What QTrial explicitly is not

- **Not a conformation superintendent platform.** The large conformation shows (Westminster, national specialties) are managed by professional superintendents with different software requirements. QTrial could theoretically grow in that direction, but not in the foreseeable roadmap.
- **Not a training management tool.** We are not tracking training logs, lesson plans, or student progress.
- **Not a pedigree database.** We record the dog's registered name, sire, and dam for catalog purposes; we are not building a pedigree tool.
- **Not a social network.** We are not building exhibitor-to-exhibitor messaging, forums, or competition leaderboards as core features.
- **Not a live-streaming or video platform.** No real-time video of rings or runs.
- **Not free of AKC regulatory constraints.** We are not reinventing how trials work; we are digitizing the volunteer workflow. AKC's rulebooks are ground truth.

## Success criteria for MVP

An MVP release is successful if:

1. **Deborah can run a real Glens Falls Kennel Club Rally/Obedience trial on QTrial end-to-end without falling back to her old Access tools.** This is the primary acceptance test.
2. **Exhibitors can enter online, pay by credit card, and receive confirmation emails** without the secretary manually touching their entries.
3. **The trial catalog, judge's books, running order, armband sheet, and scribe sheets all print correctly** and are acceptable to AKC.
4. **Results can be submitted electronically to AKC** (in the 2004 XML format, or whatever format AKC accepts in 2026), or failing that, exported to a format the secretary can upload manually.
5. **A second club can be onboarded** without code changes - multi-tenancy must actually work in practice, not just on paper.
6. **Total operational cost is low enough** that QTrial can be comped for Deborah's trials and remain viable at reasonable per-entry or per-class pricing for paid clubs.

## Success criteria for the product longer term

- Multiple unaffiliated clubs are actively using QTrial in production.
- Electronic results submission is trusted by AKC and by secretaries.
- Support load is manageable by a very small team (initially one part-time maintainer).
- The product's reputation in the dog-sport community is "it just works."

## Non-goals and deferred decisions

- **Queue management (real-time run order) - deferred to Phase 3.** A real pain point for exhibitors at the trial, but not in Deborah's outline and not required for MVP.
- **Scent Work, Agility, FastCAT, Barn Hunt - deferred.** The data model supports them; the class definitions, scoring logic, and sport-specific reports are post-MVP.
- **UKC, CKC, and other registries - deferred.** The data model is registry-aware from day one, but MVP ships AKC-only.
- **Mobile apps - deferred.** The web app must be mobile-responsive, but native iOS/Android apps are not in MVP scope.
- **Live scoring for judges in-ring - deferred.** Judges will use paper score sheets in MVP; electronic scoring comes later.

## Pricing philosophy (working assumption)

- Deborah's trials are comped indefinitely as a token of what this project is about.
- Very small trials (under a threshold of entries) are free for any club, to encourage adoption among grassroots clubs.
- Above the free threshold, pricing is per-class or per-entry with a per-checkout cap, similar to Secreterrier's published model.
- Payment processing fees are either passed to exhibitors transparently or absorbed by clubs at the club's choice.
- No long-term contracts. Pay-as-you-go.

## What makes QTrial different from the competition

Based on a survey of Secreterrier, EagerDog, DogShow.com, ShowEntries, MyK9T, Prestige Pedigrees, Lab Tested Databases, Paw Tap, and TopDogWebDesigns, the specific differentiation QTrial is pursuing:

1. **True multi-tenancy from day one.** Your mom has been running one Access file per club for years. Every competitor we surveyed is either single-tenant desktop or cloud-with-club-isolation-as-an-afterthought. QTrial treats multi-tenancy as a first-class concern, meaning one secretary can cleanly serve many clubs.
2. **Modern stack, modern uptime.** No dependencies on Microsoft Access or Microsoft Excel. Works on any device with a browser. No "Windows-only" footnotes.
3. **Lifecycle coverage.** Online entry, trial management, AKC electronic submission, and exhibitor-facing results - all in one product, not stitched across three vendors.
4. **Migration as a supported workflow.** Clubs leaving their Access-based software can bring their dog, owner, and historical event data with them.
5. **Registry-aware from the ground up.** Data model supports AKC, UKC, and other registries without architectural retrofits.

## Working assumptions that may need validation

- AKC still accepts the 2004-era XML schema that Deborah's current software targets, or a documented successor format.
- Stripe will accept a high-risk-adjacent merchant category (dog-show events are legitimate but not always recognized) without friction.
- Row-level security in Postgres is sufficient for tenant isolation at the scale we'll reach.
- Deborah is willing to be the design partner for the whole MVP cycle.

## Glossary pointer

Domain vocabulary (trial, class, entry, Q, leg, move-up, HIT, armband, etc.) is defined in `DOMAIN_GLOSSARY.md`. Do not make terminology assumptions without consulting it.
