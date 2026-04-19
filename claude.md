# CLAUDE.md

This repository is OffLeash. Before doing anything substantive, read the
documents under `docs/` in the order listed in `docs/README.md`. The
domain glossary in `docs/DOMAIN_GLOSSARY.md` is mandatory reading before
writing any code that touches the domain - getting terms wrong produces
convincing-looking broken code.

Full coding conventions are in `docs/CLAUDE.md`. Brief version:

- Variable and function names are full human-readable words.
- No em-dashes anywhere.
- Every function gets a doc comment explaining what it does and why.
- Every non-trivial block of code gets a domain-reason comment.
- Money is always `NUMERIC(10, 2)`. Never floats.
- Timestamps are always `TIMESTAMPTZ`. Never naive.
- Every tenant-scoped table has `club_id UUID NOT NULL` and an RLS policy.
- Check actual crate docs and web for present-day facts. Don't guess.
- When unsure, say so. Mark assumptions as assumptions.

## Progress check-ins

At the end of each substantive task, summarize what was done, list any
assumptions made, flag TODO items left behind, and note places where
documentation should be updated. These check-ins are required, not
optional.

## Things to ask about rather than guess

- AKC regulations beyond what's in `docs/DOMAIN_GLOSSARY.md`
- Any schema change that would affect migration from the legacy Access tool
- Any addition of a new external service or top-level dependency
- Any authorization shortcut, "just this once" exceptions to RLS included