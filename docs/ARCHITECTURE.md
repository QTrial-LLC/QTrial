# QTrial - Architecture

**Status:** Draft v0.2
**Last updated:** 2026-04-20

---

## Summary

QTrial is a web application with a Rust backend, PostgreSQL database, Keycloak identity provider, and a Next.js frontend. The stack is deliberately aligned with Robare's Mediacast Platform to maximize shared infrastructure, shared mental models, and shared operational experience.

The product is delivered as a multi-tenant SaaS. Tenancy is enforced at the database layer via PostgreSQL row-level security. Authentication is handled by Keycloak; authorization is handled in application code and via RLS policies.

## Why this stack

**Rust + axum + tokio + sqlx**
Stack consistency with Mediacast Platform is the primary reason. Secondary reasons: type-safe database access via `sqlx`, compile-time query checking, strong concurrency guarantees, small deployable binaries, low memory footprint, no GC pauses. Disadvantages: slower iteration than higher-level languages, smaller talent pool, more upfront boilerplate for CRUD.

**PostgreSQL 16**
Row-level security for multi-tenancy, robust JSON support for flexible fields (audit diffs, template variables), mature full-text search, first-class timestamp and money types, excellent operational tooling.

**Keycloak**
Single sign-on across Robare's projects (QTrial, Mediacast Platform, Mediacaster). OIDC standard. Handles password policies, 2FA, account recovery, session management, and social login without QTrial reimplementing any of it. Adds operational overhead (another service to run), but that cost is paid once across all of Robare's projects.

**React + TypeScript + Next.js**
Server-side rendering for speed and SEO on public pages (event listings, club pages, premium lists). Client-side rendering for the secretary's app. Good ecosystem for forms, tables, print-to-PDF workflows, and accessibility.

**NATS (messaging) and Valkey (caching)**
Consistent with Mediacast Platform. In MVP, most operations are synchronous HTTP; NATS is reserved for email dispatch, PDF generation offload, and submission processing. Valkey caches rendered catalogs and long-running queries.

**Stripe**
Payment processing with Connect for routing funds to club bank accounts. Handles PCI, fraud detection, and refunds. QTrial collects platform fees via Connect's application fee mechanism.

## System decomposition

### Services (MVP)

Intentionally small. Three services total:

1. **`qtrial-api`** - the core Rust backend serving the web, exhibitor, and internal APIs. Handles all business logic, validation, and direct database access. Issues signed URLs for S3 assets.
2. **`qtrial-workers`** - a Rust worker process consuming NATS jobs for email dispatch, PDF generation (catalog, judges books, AKC submission package, etc.) and batch jobs (overnight reconciliation, report aggregation). AKC XML generation is deferred until Agility support is added post-MVP.
3. **`qtrial-web`** - the Next.js frontend serving HTML to users and calling `qtrial-api` for data.

Infrastructure services (not written by us):

- **PostgreSQL** - primary data store
- **Keycloak** - identity provider
- **NATS** - message bus
- **Valkey** - cache
- **S3-compatible object storage** - PDF files, club logos, CSV exports

### Why not more services?

Microservices are seductive but the operational cost is high for a small team. Three deployables is the minimum viable structure: the backend, the async workers, and the frontend. Additional services (a dedicated PDF service, a dedicated Stripe webhook receiver) can be split off later when warranted by scale or by operational friction, not before.

## Multi-tenancy approach

QTrial is a pooled multi-tenant SaaS: one database, one set of services, all tenants share them. Tenant isolation happens at three layers:

### 1. Request routing and authentication

Every request arriving at `qtrial-api` is authenticated via a Keycloak-issued JWT. The JWT contains the user's QTrial user ID (`sub` claim). From the URL or request body, the request's target `club_id` is determined. The API validates that the authenticated user has a valid role at that club (or is a platform admin).

### 2. Database session scoping

Before executing queries for a request, the API opens a transaction and sets session-scoped settings:

```sql
SET LOCAL app.current_user_id = '<user uuid>';
SET LOCAL app.current_club_id = '<club uuid>';
SET LOCAL ROLE qtrial_tenant;
```

The `qtrial_tenant` role has SELECT/INSERT/UPDATE/DELETE permissions on all tables, but RLS policies on each table restrict rows to those with `club_id = current_setting('app.current_club_id')::uuid`.

### 3. Row-level security policies

Every tenant-scoped table has policies like:

```sql
CREATE POLICY tenant_isolation ON entries
  USING (club_id = current_setting('app.current_club_id')::uuid);
```

Reference data tables have permissive policies (read-only for all tenants). Tables involving cross-tenant resources (the user directory) are accessed via the application layer's own authorization logic, with RLS disabled.

### Trade-offs

- **Pros:** single schema to operate, simple backups, cross-tenant analytics is trivial (for platform admins), no "provisioning" step when a new club signs up
- **Cons:** all tenants share query performance and must scale together, a single bad query can affect everyone, RLS adds a small per-query cost, developer discipline required to avoid bypassing RLS accidentally

This approach is appropriate for a small-to-medium SaaS. When we reach thousands of clubs with heavy load, we re-evaluate.

## Authentication and authorization

### Authentication (who are you?)

Keycloak handles:
- Sign-up (self-service for new clubs and exhibitors)
- Password management, recovery, optional 2FA
- Sessions and refresh tokens
- Social login (Google, Apple) - P2, nice to have

QTrial web and API consume standard OIDC flows. Tokens are short-lived JWTs signed by Keycloak.

### Authorization (what can you do?)

The user's role at a given club is looked up in `user_club_roles`. Role-based checks are done in the API layer via a small middleware that loads roles into the request context.

Specific role capabilities:

- **Platform admin** - everything, with audit logging
- **Club admin** - everything within the club
- **Trial secretary** - everything within the club except inviting other users or changing club settings
- **Judge** - read their own assignments, enter scores for their classes
- **Exhibitor** - submit entries for their own dogs, view their own results and financials

Capability checks are implemented as functions (`can_edit_event`, `can_record_payment`, etc.) rather than strings parsed from the role name. This keeps authorization logic visible in code review.

## Data and integration boundaries

### External integrations

- **Stripe** - entry payments, refunds, Connect for routing funds to clubs
- **Keycloak** - identity (listed above)
- **S3 or compatible object storage** - asset storage for PDFs and logos
- **Email provider** - for confirmation emails and mailing list dispatch; likely SES or Postmark (no decision yet)
- **AKC** - for results submission. For MVP (Obedience/Rally), the mechanism is PDF package via mail or email (default `rallyresults@akc.org` for Rally). Post-MVP (Agility) adds XML submission conforming to AKC's current Agility schema.

### Internal boundaries

The API exposes three surface areas:

1. **Secretary API** - the full trial management API
2. **Exhibitor API** - limited to an exhibitor's own data
3. **Judge API** - limited to a judge's own assignments

All three are served from `qtrial-api` but use different authorization scopes.

## PDF generation

The document catalog has grown as artifact review progressed. The PDFs QTrial produces for a single trial:

1. **Premium list** - pre-trial, public
2. **Entry confirmation** - one per dog, sent when entry is processed (REQ-ENTRY-010); reference: `Confirmation_Letter*.pdf`
3. **Running schedule / judging schedule** - per day, pre-trial; reference: `Nov_2025_AKC_Rally_Trial_Judging_schedule.pdf`
4. **Catalog (pre-trial)** - per trial day
5. **Marked catalog** - post-trial, one of the three AKC submission artifacts (REQ-SUB-001); reference: `Nov_2025_Sat_Marked_Catalog.pdf`
6. **Judges books** - pre-trial with cover, post-trial with scores (REQ-SUB-002); reference: `gfkc_rally_judges_book_cover_2025_11_15_sat.pdf`
7. **Form JOVRY8 / Obedience equivalent** - post-trial, one of the three AKC submission artifacts (REQ-SUB-003); reference: `Trial_Summary_report.pdf`
8. **Steward board** - per class, pre-trial; reference: `Stewards_BOard_Sat.pdf`
9. **Scribe sheets** - Obedience exercise-by-exercise scoring
10. **Armband cards** - printed, distributed at check-in

These artifacts share ~80% of their data (same dogs, same entries, same classes) but differ in structure and audience. A shared rendering layer is therefore architecturally called for.

Approach: server-side HTML rendering to PDF via a headless Chrome instance (puppeteer/playwright-equivalent) running in the workers service. Templates are HTML with Tailwind CSS, rendered in Rust using a templating engine (`askama` or `minijinja`), then converted to PDF. A shared data-fetching layer loads the per-trial data once and passes it to each renderer.

Form JOVRY8 (and the Obedience equivalent) is a different shape: it's a fixed AKC-published PDF form that QTrial populates via PDF form-field fill rather than HTML-to-PDF. This is a distinct code path using a PDF manipulation library (leading candidate: `pdf-lib` via a Node helper, or `lopdf` in Rust) rather than the HTML pipeline.

Alternative considered for HTML-based PDFs: typst (Rust-native typesetter), which produces beautiful output but has a smaller ecosystem for template sharing. Reserved for P2 consideration.

## Deployment

### MVP target

- Single environment to start: production, deployed on AWS
- `qtrial-api` and `qtrial-workers` run as ECS tasks or on Fargate
- `qtrial-web` served via Vercel or as an ECS task behind CloudFront
- PostgreSQL via Amazon RDS (Postgres 16)
- Keycloak via ECS task backed by its own RDS instance
- NATS via self-hosted ECS task (single node is fine for MVP volumes)
- Valkey via ElastiCache
- S3 for object storage
- Secrets in AWS Secrets Manager
- IaC via Terraform

### Staging

Deferred until we have paying customers. Until then, Deborah's first real trial is staged on a dedicated tenant within production.

### Observability

- Structured JSON logs from all services shipped to CloudWatch (or via OpenTelemetry collector to a backend TBD)
- Metrics via OpenTelemetry to CloudWatch Metrics or similar
- Tracing via OpenTelemetry, with span propagation across services
- Error tracking via Sentry

## Security considerations

### PII handling

QTrial holds real names, home addresses, phone numbers, email addresses, and payment methods. We are not processing credit card numbers directly (Stripe handles those), but we still hold sensitive personal data.

Controls:
- TLS 1.3 everywhere, HSTS enforced
- Database encryption at rest (RDS default)
- S3 bucket encryption
- No PII in logs (enforced via a logging middleware that strips known PII fields)
- Password hashes in Keycloak, never in QTrial
- Access audit log for platform admin actions

### Financial data

We hold transaction records (amount, date, method) but never full card numbers. Stripe handles PCI compliance for card data. For checks, we record only the check number (not the bank account number).

### AKC submission integrity

AKC submissions are legal records of competition results. Controls:
- Every submission generated is archived in S3 with an immutable key
- The submission record tracks who generated it, when, and its lifecycle status
- Score changes after submission require a secretary-authorized re-submission with audit trail

### Child safety

Junior handlers can be as young as 9 years old. We restrict:
- Junior handler accounts are managed by a parent/guardian account
- We do not expose junior handler contact information publicly
- Entry confirmation emails for junior handlers go to the parent/guardian account

## Development workflow

### Repository layout

Single monorepo (`qtrial/`):

```
qtrial/
├── api/          (Rust: axum + tokio + sqlx)
├── workers/      (Rust: background jobs)
├── web/          (Next.js)
├── shared/       (Rust crate shared between api and workers)
├── db/
│   ├── migrations/   (sqlx-managed SQL migrations)
│   └── seed/         (reference data seeds - AKC classes, breeds, titles)
├── infra/        (Terraform)
├── docs/         (markdown docs, including this one)
└── CLAUDE.md     (conventions for Claude Code)
```

### Conventions

- Rust style follows rustfmt defaults with `cargo clippy` at `-W clippy::pedantic` level
- TypeScript style follows Prettier + ESLint with strict mode
- Commits follow Conventional Commits
- Every PR must pass CI (build, test, lint, type check) before merge
- Every database migration is reviewed separately from application code changes
- No em-dashes in code comments or user-facing strings (Robare preference; reads as LLM-generated)
- Variable names are full human-readable words, not abbreviations, unless the abbreviation is an industry standard (like `db`, `api`, `http`)
- Every function has a brief doc comment explaining what it does and why

### Testing strategy

- **Unit tests** - Rust: `#[cfg(test)]` modules; TypeScript: Vitest
- **Integration tests** - Rust: `tests/` with a real Postgres database spun up via testcontainers
- **End-to-end tests** - Playwright tests against a staging environment
- **Load tests** - before each major release, run load tests simulating 100 concurrent entries submitting simultaneously (the worst case at entry opening)

## Open architecture questions

1. **PDF generation: Rust HTML-to-PDF vs Node service vs typst.** Leaning Rust + headless Chrome for MVP, but worth a quick prototype comparison.
2. **AKC PDF form-fill library.** Form JOVRY8 is a fillable PDF. Evaluating `lopdf` (Rust) vs a Node `pdf-lib` helper service. Decision before Phase 6 kicks off.
3. **Hot path performance for the entry-open rush.** Events can have 100+ exhibitors all submitting at entry opening. Needs rate limiting and queue-based ordering. TBD in implementation.
4. **How do we handle trial-day offline scenarios?** Many trials are in areas with poor internet. For MVP we assume internet is available; P2 may need a progressive web app with offline support or a desktop fallback for critical trial-day operations.
5. **Where do the PDF generation jobs live?** All in `qtrial-workers` for isolation (leaning this way; submission is inherently async and PDF generation is CPU-heavy enough to benefit from running off the request path).
6. **Keycloak administration UI** - do club admins have a Keycloak admin UI or do they manage users via QTrial's own UI? Leaning QTrial UI (keep Keycloak as an invisible identity backend).
