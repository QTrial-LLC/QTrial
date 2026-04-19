# OffLeash

**Status:** Pre-development. This repository currently contains foundational documentation only. Code begins after document review.

OffLeash is a cloud-hosted, multi-tenant platform for managing AKC (and eventually UKC) dog sport trials. It is being built to replace the Microsoft Access-based tools that volunteer trial secretaries have been using since the early 2000s.

The primary user is Deborah Pruyn, trial secretary for Glens Falls Kennel Club. The first real use is targeted for a spring 2027 trial.

## Document map

Read in this order:

1. **[PROJECT_CHARTER.md](./PROJECT_CHARTER.md)** - why OffLeash exists, who it's for, what success looks like, and what it explicitly is not.
2. **[DOMAIN_GLOSSARY.md](./DOMAIN_GLOSSARY.md)** - precise definitions of dog-sport and AKC terminology. Every contributor should read this before writing code.
3. **[REQUIREMENTS.md](./REQUIREMENTS.md)** - what OffLeash must do, tagged by phase (MVP, P2, P3) and by certainty (confirmed, pending, assumed).
4. **[DATA_MODEL.md](./DATA_MODEL.md)** - the database schema, with mappings from Deborah's current Access schema to the OffLeash model.
5. **[ARCHITECTURE.md](./ARCHITECTURE.md)** - the technical stack, multi-tenancy approach, and deployment model.
6. **[WORKFLOWS.md](./WORKFLOWS.md)** - narrative walkthroughs of user-facing operations, from a new club signing up through submitting results to AKC.
7. **[ROADMAP.md](./ROADMAP.md)** - phased delivery plan with dates keyed to Deborah's real trials.

## Quick reference

**Product name:** OffLeash (working name, pending trademark clearance before monetization)

**Tagline (TBD):** something around "the trial secretary's best friend," but deliberately not generic dog-software copy.

**Stack:**
- Backend: Rust + axum + tokio + sqlx
- Database: PostgreSQL 16 with row-level security
- Identity: Keycloak (OIDC)
- Frontend: React + TypeScript + Next.js
- Message bus: NATS
- Cache: Valkey
- Payments: Stripe Connect
- Hosting: AWS

**Tenancy:** multi-tenant from day one; row-level security enforces isolation.

**Sport scope:**
- MVP: AKC Obedience and Rally
- Data model supports all AKC and UKC sports
- Post-MVP expansion order: Agility, Scent Work, Conformation, FastCAT, Barn Hunt

**Pricing model (working):**
- Free for Deborah
- Free for small trials under a to-be-determined entry threshold
- Per-class fee above threshold, with per-checkout cap (similar to Secreterrier's model)
- Card processing fees passed to exhibitors or absorbed by club, at club's choice

## What's in Claude project context vs. this repo

This folder is the seed for the **Claude project** used for architecture and planning work. When Robare moves to implementation, these documents will also live in the actual code repository (under `docs/`) so that Claude Code has access to them while writing code.

The code repository will additionally contain:
- `CLAUDE.md` - coding conventions and tool usage guidance for Claude Code
- `README.md` - developer-facing setup instructions (will supersede this file once code exists)
- `CONTRIBUTING.md` - for if/when others contribute
- Source code, migrations, tests, infrastructure, and so on

## Open questions (summary)

Each document has its own "pending" section. The most important unanswered questions are:

- **AKC submission mechanism in 2026** - still email-attached XML, or has it moved to a portal or API? Needs direct contact with AKC.
- **Current AKC XML schema** - Deborah's software references a 2004 schema; we need the current one.
- **Deborah's trial-weekend narrative** - what actually happens from Friday setup through Sunday wrap-up?
- **PDF examples** - catalog, judge's book, scribe sheet, confirmation email, AKC Report of Trial.
- **Other clubs' data** - if Deborah has Access files for other clubs she works with, their configuration variety would inform section 1.3 of `REQUIREMENTS.md`.

## Artifacts received

- **`Outline_of_Online_Trial_software.pdf`** - Deborah's initial outline (April 2026).
- **`Obedience_Solution.mde`** (35 MB) - the application half of her current Access tool (Obedience Solution by Lab Tested Databases).
- **`ObedienceData.mde`** (11 MB) - the data half. Schema and reference data extracted and used as a starting point throughout this documentation set.

## Contact and ownership

- **Project lead:** Robare Pruyn
- **Primary stakeholder / first user:** Deborah Pruyn
- **Development model:** solo engineer + Claude Code, evenings and weekends
- **Target first real use:** spring 2027 (approximately 9 months from project kickoff)
