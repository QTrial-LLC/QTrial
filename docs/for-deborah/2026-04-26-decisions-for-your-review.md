# Decisions Robare made on your behalf

**For:** Deborah
**From:** Robare
**Date:** 2026-04-26

These are the calls I made between our last conversation in April
and now where reasonable people could have gone the other way.
Each one had me weighing a tradeoff and picking a path, usually
based on how I thought GFKC actually runs. I'd like you to push
back on anything that surprises you. Where the path matches what
you'd do, "yep" or a checkmark is plenty; where it doesn't, even
a one-line correction saves me from building toward the wrong
target.

Eight items below. None are urgent. Take them at your own pace.

---

## 1. Breed restrictions ship as a single yes/no flag for now

**What we decided.** QTrial events carry a single "mixed breeds
allowed" flag (yes by default). The broader breed-list
mechanism (allow only Goldens, exclude this group, etc.) is
deferred to a later piece of work.

**Why.** Per your April 23 answer, the immediate need is the
mixed-breed exclusion case for conformation, where some events
won't accept All-American Dog entries. A single flag handles that
cleanly. The richer model (lists of allowed or excluded breeds and
breed groups, validated against the dog's breed at entry time)
needs a real Specialty or Group show premium to design against,
and we don't have one in hand. Building it on speculation produces
a worse model than waiting.

**What we want from you.** Does the GFKC trial calendar have
anything coming up in the next year or two that's breed-restricted
beyond the All-American Dog case? Specialty events, Group shows
where you'd be helping someone? If yes, that changes the urgency
on the richer model.

---

## 2. The judges book has two PDF records, not one

**What we decided.** For each class at each trial, QTrial keeps
the pre-trial blank judges-book PDF (the one we generate so you
can print the books for the judge to score on) and the post-trial
signed scan as two separate records, not one record overwritten
when you upload the signed version.

**Why.** A judge changing late in the cycle is real (you've
mentioned it), and re-rendering a blank book shouldn't risk
overwriting a signed scan that's already been uploaded. Two
records also gives you a clean audit later: the blank we generated
sits beside the scan you actually signed and mailed. The cost is
trivial, just two pointer fields instead of one.

**What we want from you.** Does this match how you'd want to use
the system? When QTrial shows you a class's judges book, do you
expect to see both the blank and the scan side by side, or are
you more likely to just need the most-recent version of whichever
is current? That's a screen-design question for later, but the
data shape supports either, so worth knowing.

---

## 3. Trial Chair and Event Secretary are event-level, not per-trial

**What we decided.** For each event, QTrial tracks one Trial Chair
and one Event Secretary. Those are two distinct roles per your
April 23 framing: the chair handles pre-trial arrangements
(judges, accommodations, stewards, expenses); the secretary
handles the trial itself (paperwork, scores, entries). One of each
per event, shared across the trials within the event.

We dropped the per-trial "trial chairperson" string field that
was on the trials table from earlier in the project. Per your
answer, the chair is event-level. If a future case comes up where
one weekend needs different chairs for Saturday and Sunday, we can
add that back; we won't carry it preemptively.

**Why.** Your June 2026 Rally premium has Chris Argento as the
single Rally Trial Chair across both Saturday and Sunday, and
you as the single Trial Secretary across both. That seems to be
the GFKC pattern, and it's what AKC's premium-list conventions
expect.

**What we want from you.** Confirm or push back. Is there a GFKC
trial weekend, past or upcoming, where you've had two different
chairs across the days?

---

## 4. The combined-award groups, including the Master + Choice drop

**What we decided.** QTrial seeds five combined-award groups: AKC
Obedience HC (Open B + Utility B), Rally HC (Advanced B +
Excellent B), Rally HTQ (Advanced B + Excellent B + Master),
Rally RAE title path (Advanced B + Excellent B), and Rally RACH
title path (Advanced B + Excellent B + Master). Master + Choice
as a sixth group is NOT in the seed.

**Why.** The five groups above each have a clear AKC source: HC
in Obedience Reg. Ch. 1 §22; RHC in Rally Reg. Ch. 1 §31; RHTQ
in Rally Reg. Ch. 1 §32; RAE in Rally Reg. Ch. 3 §15; RACH in
Rally Reg. Ch. 4 §§2 and 4. Master + Choice does not appear in
the AKC Rally Regulations as a combined-award path, despite the
"Master + Choice" wording on your June 2026 premium. A direct
text search of the current Rally Regulations confirmed three
independent absences: Chapter 1 Section 24 lists the AKC-recognized
combined entries (RAE = Advanced B + Excellent B; RACH = Advanced B
+ Excellent B + Master) and Master + Choice is not among them;
Chapter 1 Section 32 defines HTQ as Advanced B + Excellent B +
Master only, with Choice not in the triple; and Chapter 3 Sections
18 and 19 define Choice as a standalone class with no combined-
award role. The reading is that Master + Choice is a GFKC fee-
discount line on the premium, offered as a courtesy to handlers
who are running a RACH-track dog and adding Choice the same day,
but not an AKC award path.

**What we want from you.** This is the most important push-back
item in this list. You know the rulebook in practice better than
I do reading it cold. Is there a club convention or AKC reading
I'm missing that would put Master + Choice on the seeded list?
Is there an award given for Master + Choice that I haven't
accounted for? If the drop is wrong, the discount logic and the
catalog rendering both pick up the change automatically once we
add the row back.

---

## 5. The additional-entry discount applies to any combined-award track

**What we decided.** The fee engine applies the additional-entry
discount when a dog is entered in two or more classes from the
same combined-award group at the same trial. This includes Open B
plus Utility B for the OTCH track, not just the
Master/Choice/RAE/RACH path your premium calls out.

**Why.** Per your April 23 answer to the additional-entry-rate
scope question, the discount applies to "ANY double or triple Q
in B class in one trial. Most definitely, including Open and
Utility B." Modeling combined-award groups as data, then asking
"is this dog entered in 2+ classes from the same group?" is the
most honest way to encode that rule. It also generalizes naturally
when other combined-award groups land later.

**What we want from you.** This is your own answer, just turned
into a rule. Reading it back, does anything feel off? Is there a
case at GFKC where you've applied the discount that wouldn't fit
the "two or more classes from the same combined-award group"
shape? An OTCH-track or HC dog who runs three classes counts
because of the Open B + Utility B membership, but is there a
non-combined-award case where you'd discount?

---

## 6. Each club's dogs are tracked separately for now

**What we decided.** For MVP, each club's dog directory stands on
its own. A dog entered at GFKC and at a different club where
you've been secretary lives as two separate records in QTrial,
one per club.

**Why.** Per your April 23 answer to the cross-club dog-identity
question, Obedience and Rally trial-secretary work doesn't need
cross-club dog identity. The case where it does matter is cluster
trials, where multiple clubs share a venue across consecutive days
and a dog runs at all of them with title history that has to
follow. That's predominantly a conformation phenomenon and is
typically run by superintendents (the two big ones, Onofrio and
MB-F), not by individual club secretaries. Cross-club dog identity
becomes a real concern when QTrial adds conformation; deferring it
keeps the multi-tenancy story simple for MVP.

**What we want from you.** Confirm or push back. Two questions
worth flagging if they apply: (a) Have you ever helped at a cluster
trial as the secretary rather than as the superintendent, and
needed to track the same dog across clubs? (b) For owners who
enter at multiple clubs you secretarial-help, would they be
confused by needing to register the dog separately at each one,
or is that already how things work today?

---

## 7. Officer slate is updated, not historically preserved

**What we decided.** The club officer slate (President, Vice
President, Treasurer, Recording Secretary, AKC Delegate, Board
Members) lives in one record per club, updated when the club's
slate changes. A premium list QTrial generates today shows the
current slate; if the slate changes in February and you generate
another premium in March, the March premium picks up the new
names.

The slate as it stood when an older premium was originally
printed is not preserved. If that turns out to matter (you've
mentioned the timing of your departure from the GFKC board as a
concrete example), QTrial can grow a historical-preservation
table that records the slate per date range. Today it does not.

**Why.** Per your April 23 answer, officers are a club-side
determination, the same across all events the club runs in a
given year, and updated yearly with elections. The simple shape
matches that cadence cleanly. Historical preservation is real but
not load-bearing for MVP.

**What we want from you.** Does the GFKC slate ever differ between
events at the same club within the same year, outside of an
election? If yes, the simple shape doesn't fit and we need the
historical model sooner. If no, the simple shape is right.

---

## 8. AKC submission stays PDF for Obedience and Rally

**What we decided.** QTrial submits Obedience and Rally results
to AKC the way you do today: a marked catalog PDF plus the populated
JOVRY8 PDF (or its Obedience equivalent) plus the signed judges
books, emailed to rallyresults@akc.org or its Obedience parallel,
or mailed physically to PO Box 900051 in Raleigh. XML submission
is for Agility, and Agility is post-MVP; we're not building XML
for the MVP path.

**Why.** Per your April 19 conversation, the PDF package is what
AKC currently accepts for Obedience and Rally; XML is Agility-
only. This is the lowest-risk path because you're already using
it; QTrial's job is to produce the PDFs cleanly and let you
attach them to an email you send.

**What we want from you.** Has anything changed on AKC's side
since April? If they've started accepting another format for
Obedience/Rally, or started rejecting the PDF package, that's a
ground-shift we'd want to know about before this gets too far
into implementation.

---

That's the list. Anything you confirm with a checkmark, anything
you push back on, anything you want me to ask a follow-up about,
all useful. Take your time. As always, the corrections you'd make
that I'd never have caught on my own are why we have these
conversations in the first place.

Thanks.
