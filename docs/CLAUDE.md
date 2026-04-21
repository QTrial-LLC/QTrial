# CLAUDE.md

Conventions and guidance for Claude (and Claude Code) working on QTrial.

## Project context

QTrial is a multi-tenant SaaS for managing AKC dog sport trials. Before doing anything substantive, read:

1. `docs/PROJECT_CHARTER.md` - what we're building and why
2. `docs/DOMAIN_GLOSSARY.md` - dog-sport terminology. **Read this before writing any code that touches the domain.** Getting terms like "trial", "class", "level", "Q", "leg", or "HIT" wrong will produce convincing-looking code that is factually broken.
3. `docs/REQUIREMENTS.md` - what the system must do
4. `docs/DATA_MODEL.md` - the schema and how it maps from Deborah's legacy Access system
5. `docs/ARCHITECTURE.md` - stack, tenancy approach, deployment
6. `docs/WORKFLOWS.md` - user-facing operations
7. `docs/ROADMAP.md` - phased delivery plan

## Style preferences

These are Robare's preferences. Treat them as constraints, not suggestions.

### Code

- **Variable and function names are full words.** `trial_secretary`, not `ts`. `entry_line`, not `el`. `canonical_class`, not `cc`. Industry-standard abbreviations (`db`, `http`, `api`, `id`) are fine.
- **Every function has a doc comment** explaining what it does and why the caller would want to call it. Not what the code does line-by-line; what the function accomplishes.
- **Every non-trivial block of code has a comment** explaining the domain reason for the code. "Calculate the first-entry fee" is a domain comment; "Multiply 0.15 by x" is a useless tautology.
- **No em-dashes in code comments, variable names, or user-facing strings.** Robare reads em-dashes as LLM-generated text. Use regular dashes or commas.
- **Direct and honest feedback preferred over validation.** When reviewing code, name the actual problem. Don't soften bad ideas; Robare would rather hear "this has a race condition" than "this is mostly good but consider concurrency."

### Git

- Conventional Commits format
- Branch names: `feature/entry-flow`, `fix/waitlist-promotion`, `docs/data-model-refinement`
- Squash merges preferred; branch commits may be messy, main should be readable

### Rust specifics

- `cargo fmt` before every commit
- `cargo clippy -- -W clippy::pedantic` clean (allow specific lints where necessary, with a comment explaining why)
- Prefer `?` over `unwrap()` outside tests and obvious invariants
- Derive `Debug` on almost everything; derive `Clone` only when needed
- Use `sqlx::query!` and `sqlx::query_as!` for compile-time query checking
- Structured errors via `thiserror`; result-carrying errors via `anyhow` only in the top-level binaries

### TypeScript specifics

- Strict mode on
- No `any` without a comment explaining why
- Prefer functional components with hooks over class components
- Forms: react-hook-form with zod schemas, not ad-hoc state

### Database

- Every migration is reversible (or explicitly not, with a comment)
- Migration files named `YYYYMMDDHHMMSS_description.sql`
- Never edit a committed migration - add a new one instead
- Every tenant-scoped table has `club_id UUID NOT NULL REFERENCES clubs(id)` and an RLS policy
- Every mutable table has `created_at`, `updated_at`, `created_by`, `updated_by`

## Domain rules that must never be violated

1. **The event → day → trial hierarchy is sacred.** An event has days, days have trials. Do not collapse these into one level.
2. **A "class" at the instance level (a specific class at a specific trial) is distinct from a "canonical class" (the master catalog entry).** `canonical_classes` vs `trial_class_offerings`. Do not conflate them.
3. **Entry has a state machine.** Use the enum, don't add parallel boolean columns like `is_scratched`, `is_waitlisted`, etc.
4. **Money is `NUMERIC(10, 2)` always. Never floats.**
5. **Timestamps are `TIMESTAMPTZ` always. Never `TIMESTAMP` without time zone.**
6. **AKC registered names have a specific format.** Prefix titles (space-separated), then registered name, then ", suffix titles" (comma-separated). Do not concatenate ad-hoc.
7. **Every tenant-scoped query goes through RLS.** Do not use service accounts to bypass RLS except in platform admin contexts, and when you do, it must be logged.

## Tool usage

- For any fact about the present-day world (AKC rules, Stripe pricing, library versions, current package behavior), use web search before answering. Don't guess from training data.
- For Rust crate APIs, prefer the crate's actual documentation over memory.
- When in doubt about AKC regulations, note the uncertainty and mark the code with a TODO for Robare or Deborah to verify.

## Things Claude Code should ask about, not guess

- AKC-specific behaviors beyond what's in `DOMAIN_GLOSSARY.md`
- Pricing-model mathematics (the current thinking is in `PROJECT_CHARTER.md` but specifics will evolve)
- Any time a schema change might affect migration from the Access tool
- Any time a feature would require an additional external service
- Any time authorization logic might be bypassed "just this once"

## Things Claude Code should NOT do

- Do not introduce new top-level services without explicit approval
- Do not add dependencies without checking license and maintenance status
- Do not write code that depends on a specific AKC endpoint or API without verifying the endpoint exists and is current
- Do not commit directly to main
- Do not write marketing copy, product names, or user-facing content without Robare's review
- Do not use em-dashes anywhere a human might read the output

## Progress check-ins

At the end of each feature or substantial refactor, Claude Code should:

1. Summarize what was done
2. Note any assumptions made
3. List any TODO items left behind
4. Flag anything that might conflict with other in-progress work
5. Mention any places where the documentation should be updated

Robare reads these check-ins; they are not optional.
