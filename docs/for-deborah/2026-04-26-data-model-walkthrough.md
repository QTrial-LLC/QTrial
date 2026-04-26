# A walkthrough of what QTrial keeps track of

**For:** Deborah
**From:** Robare
**Date:** 2026-04-26

---

## How to read this document

This is a tour of the records QTrial keeps about a trial. Not the
screens you'll see when you use it, and not how the database is
built underneath. Just the boxes of information QTrial holds and
roughly what's in each one.

The point of you reading this is so we catch anything wrong now,
before screens are wired to it. **Anywhere a name feels off, a
field looks missing, or a case Glens Falls runs into wouldn't fit,
that's exactly what we want to hear about.** Even small things
like "you said running order, we call it ring order" are worth
flagging. Those are the kinds of mistakes that pile up and make a
tool feel like it was built by someone who doesn't run trials.

Read it in chunks. There are seven sections; each takes about
five minutes. You don't need to get through it all in one sitting,
and you don't need to answer every question I tucked in at the
end of each section. Mark up anything that catches your eye and
send it back when you have time. Pen on a printout, voice memo,
text, phone call, all fine.

I'm using your November 14-15, 2025 Glens Falls Rally weekend as
the running example, plus a few specific dogs from the Saturday
June 21, 2025 judges book where I needed armband-and-breed details
to make a sentence concrete. So when you see Denise LaCroix's name
or armbands 101 through 107, that's the June book; when I'm
pointing at the November weekend specifically, I'll say so.

---

## 1. The starting point: clubs and the people in them

QTrial holds a record for each club. For Glens Falls Kennel Club,
that record carries the things you'd expect to see at the top of
a premium list: club display name, abbreviation if you use one,
the AKC club number, and a logo if the club wants one printed on
catalogs. The Saturday-Sunday June 20-21, 2026 Rally premium has
the GFKC info block that maps to this record almost line for line.

Alongside the club record, QTrial keeps the club's officer slate.
For GFKC the slate currently looks like President Marion (Gigi)
Rowland, Vice President Brianna Alexander, Treasurer Anne O'Neil,
Recording Secretary Cate Howland, AKC Delegate Colleen Kimble,
plus the eight members of the Board of Directors (Pat Prutsman,
Lois Hammond, Dominic Amedio, Maureen Kramer, Chris Argento, Nancy
Wade, Mindy Willey, Sue McCoy). The officer slate is stored as a
list, in the order it appears on the premium list. Each entry has
the office, the person's name, an email, and a phone number. Board
members are listed individually with the same shape, so the renderer
can iterate the list in order without special-casing the multi-person
roles.

The officer slate is club-level, not event-level. When a January
election changes who's on the board, you update one record and
every premium list QTrial generates afterwards picks up the change.
The list as it stood when an older premium list was printed is not
preserved; if you need that history later, it's something we'd add
as a separate table down the road. (You mentioned the timing of
your departure from the GFKC board on the email back in April, and
that's the example that prompts the historical-preservation question.)

Separately from the officer slate, QTrial knows about the people
who actually use the software at GFKC: club admins (who can change
club settings), trial secretaries (you), exhibitors (the people who
enter their dogs), and judges (with QTrial accounts when they want
one). One person can have multiple roles, and one person can hold
roles at multiple clubs. Each role grant carries the date it was
issued and is tied to the specific club, so a person who's a club
admin at GFKC and an exhibitor at another nearby club has both
roles tracked separately.

### Questions

- The officer slate today has President, Vice President, Treasurer,
  Recording Secretary, AKC Delegate, and Board Member as the offices
  on a typical GFKC premium. Are there other offices that show up at
  GFKC or at clubs you've helped that we'd miss?
- We treat the slate as one list per club, updated when the slate
  changes. Does GFKC ever run an event where the slate on the premium
  list differs from the slate on the club's website or calendar at
  the same moment? I don't think so, but worth asking.

---

## 2. Setting up an event

QTrial follows AKC's event-day-trial hierarchy strictly. An event
is one approved AKC gathering at a venue over one or more
consecutive days. Inside the event, each calendar day is a separate
record. Inside each day, each trial is a separate record. Your
November 14-15, 2025 weekend is one event with two days, and
Saturday is one day with one Rally trial inside it (AKC event number
2025103018), Sunday another day with its own Rally trial.

The event record carries the things that are shared across all
trials in that event: the event name, cluster name if you participate
in one, the venue (474 Corinth Rd, Queensbury), the entry-open and
entry-close dates and times, the move-up deadline, the catalog fee,
and the armband numbering scheme. The big practical reason for the
hierarchy is AKC event numbers: AKC issues one number per trial per
day per sport, so a Rally trial Saturday and a Rally trial Sunday
get different numbers even though they're under one event approval.

Two new fields land here that come from your April 23 email:

- **Trial Chair.** The person doing pre-trial arrangements: getting
  the judges, getting AKC approval, arranging accommodations,
  recruiting stewards, paying judge expenses afterwards. For the
  June 2026 weekend that's Chris Argento. The Trial Chair is
  event-level, shared across both trials of the weekend, not per-trial.
- **Event Secretary.** That's you. Same setup, event-level, shared
  across the trials within the event.

QTrial used to track a single chairperson string per trial; based on
your answer that the chair is event-level, that per-trial field is
gone. If a future case comes up where one person chairs Saturday
and a different person chairs Sunday, it can be added back. For
now, one chair and one secretary per event.

The day record is small. Day number (1, 2, 3 within the event), the
calendar date, and the planned start time on that day. Saturday is
day 1, dated 2025-11-15 for last November's weekend, planned start
8 AM.

The trial record is where the per-trial details live: trial number
within the day (typically 1 unless you're running AM and PM),
sport (Obedience or Rally), the AKC event number, planned start
time, entry limit, and the per-trial fees. For November 2025 there
was one Rally trial per day and the fees were configured per trial
even though they were the same on both days.

### Questions

- We track one entry-open and one entry-close datetime per event,
  not per trial. Does that match how GFKC operates? I think so given
  the two dates are listed once at the top of each premium, but want
  to be sure you've never had a multi-trial event where the windows
  differ.
- Trial Chair as event-level, not per-trial. Is there a case at any
  GFKC weekend, past or planned, where you've had one chair for
  Saturday and a different chair for Sunday?
- The day record carries a planned start time. Do you ever publish
  one start time for the day and then run trials on different
  schedules within that day, and do you want both timestamps tracked?

---

## 3. What's offered at each trial

A trial is a vessel; what's actually being competed in are the
classes offered at that trial. Inside QTrial, classes come from a
master list (the canonical class catalog) and each trial's offerings
are pulled from that list with per-trial overrides. The master list
has 75 entries today, covering Obedience and Rally; it's the seed
data we worked through last April that you reviewed for the title
catalog.

A trial-class-offering record links one canonical class to one
trial. For your Saturday November 15 Rally trial, there'd be one
offering for Rally Novice A, one for Rally Novice B, one for Rally
Intermediate, Rally Advanced A, Rally Advanced B, Rally Excellent
A, Rally Excellent B, Rally Master, and Rally Choice. Each of those
nine offerings carries:

- Ring number (you've told us "Ring 0" was an Obedience Solution
  default rather than a real number; QTrial defaults to Ring 1 and
  rejects Ring 0 on save)
- Class limit (the cap from the premium, "Limits of 100 dogs/Trial"
  for the June 2026 premium)
- Scheduled start time for the class
- Running-order strategy (short to tall, tall to short, random,
  manual, or reverse of the previous day for Sunday classes that
  shadow Saturday)
- Per-class pacing override in minutes per dog, because one
  class-pace doesn't fit all (Rally Choice paces around 4.3
  minutes per dog at GFKC, Rally Master 3.5, Rally Excellent B
  3.1, per the November 2025 schedule you pulled apart with us)

Once classes are offered, judges get assigned. The judge directory
is a separate part of QTrial that holds AKC judge number, name,
contact info, and provisional status. For the June 2025 Rally trial
the directory entry for Denise LaCroix held her AKC number 18254
and her address. For Saturday November 15 the panel was Robin
Botelho. The judge-assignment record is the join: Denise LaCroix
assigned to Rally Novice A on June 21, Robin Botelho assigned to
Rally Novice A on November 15. A class can have a co-judge
designation if two judges are sharing it; a single judge can be
assigned to many classes within a trial.

Fees on the trial: first-class fee, additional-class fee,
nonregular fee, brace fee, team fee, Rally Pairs fee, Rally Team
fee, junior-handler rates for the first-class and additional-class
fees, and the catalog fee on the event. The June 2026 premium has
$30 first / $25 additional with the additional rate noted as
applying to "Master + Choice, RAE & RACH Title entries only" -
which is itself one of the things we want your eyes on; I'll come
back to it under awards in section 6.

### Questions

- Have we missed any per-class knob you actually use? I have ring
  number, class limit, scheduled start time, running-order strategy,
  per-class pacing override. What else gets set per class at GFKC?
- Class limit defaults to "no limit" at GFKC because you've never
  hit one, per the April 19 conversation. Confirming this is still
  true for both Saturday and Sunday trials at the upcoming weekends.
- Co-judging - has GFKC ever had two judges share a single class?
  We support it but it's worth knowing if it's a real case for you
  or just a structural possibility.

---

## 4. Dogs and their people

The owner record is the contact-information record for the person
on the dog's AKC registration. Name, address, phone, email,
mailing-list opt-ins, whether they're a club member, whether the
record is currently active. It's distinct from "exhibitor" because
the exhibitor is whoever entered the dog into the trial, who's
usually but not always one of the owners.

The dog record is what you'd expect: call name, registered name,
breed, breed variety if the breed has them (Poodles, Cocker
Spaniels, etc.), sex, AKC registration type (full AKC, PAL,
Canine Partners for All-American Dogs, FSS, Miscellaneous),
registration number kept exactly as written including leading
zeros, country of registration, date of birth, breeder, sire and
dam (each as one full registered name string with their titles
embedded), AKC jump-height card details if the dog has one, and
two flags: whether the dog has an active AKC ineligibility on file,
and whether the dog has been retired by the owner.

Co-ownership runs through a separate dog-ownerships table. One
dog can have multiple owners; exactly one is marked primary. This
replaced an older "co-owners as a single text blob" approach
because owners change and a structured record is what catalog
generation needs.

The dog title catalog itself is reference data, shared across
clubs. Today it carries 49 prefix titles and 244 suffix titles
plus the legacy compound forms and the 10 Barn Hunt titles you
flagged in last April's review. When a dog's registered name
arrives looking like "GCH OTCH Kensington's Moonlight Sonata UDX
RAE" QTrial parses out the prefix titles ("GCH OTCH"), the
registered name proper ("Kensington's Moonlight Sonata"), and the
suffix titles ("UDX RAE"), then matches each token against the
catalog. Anything it doesn't recognize ("UCGC", "WCCC?", garbled
concatenations like "CGUWCX") is preserved verbatim in the dog's
record under a separate field so you can review it instead of
QTrial silently inventing a title catalog entry.

Jump height is per-dog-per-trial, not per-class. A dog jumps the
same height in every class it runs at one trial. The per-trial
jump-height record holds the dog, the trial, the elected height
in inches, whether a judge measured the dog in-ring and overrode
it, when, and the contact who measured. The integer-only height
buckets (Obedience 4 through 36, Rally 4/8/12/16) are enforced; no
fractional heights at trial time even if the dog's measurement card
says 13.5 inches.

### Questions

- Co-ownership - we treat one owner as primary and the others as
  co-owners. Anything beyond name + address + email + phone we
  should track per co-owner that GFKC actually uses?
- The dog record has flags for "AKC ineligibility on file" and
  "retired by owner." Both surface on the entry path so a retired
  dog shows up with a warning. Are there other dog-level statuses
  you want to track, or are those two enough?
- Does GFKC ever encounter a dog that wants to enter under
  unconventional title formatting (a foreign title, a mixed-bag
  organization that doesn't fit the AKC catalog)? The unparsed-
  token route is the safety net for those, but worth sanity-checking
  it covers what you've actually seen.

---

## 5. Entries

Each dog at each event has one entry record. That record is the
top-level holder for the dog's participation: which event, which
dog, who the exhibitor is, who the owner of record is, the
exhibitor's payment method and totals, when confirmation and
results emails went out, any notes the secretary attached. There
is one entry per dog per event, not per dog per class.

Inside the entry, each class the dog is in is its own entry-line
record. So a dog entered in Rally Master and Rally Choice on
Saturday at GFKC has one entry record (with armband, owner,
exhibitor) and two entry-line records (one per class). Each
entry-line carries:

- The trial-class-offering it's against (Rally Master at this
  trial)
- The handler-of-record contact (which can differ from the entry's
  exhibitor and from the owner; junior handlers are the common case)
- The junior handler's AKC number if there is one
- A team-membership pointer if it's a brace, team, or pairs entry
- Where it is in the entry lifecycle (more on this in a second)
- The running-order slot when the secretary publishes the order

Armband numbers run through a small junction table because of how
the 500 series works. A dog at a Rally trial that runs Advanced B,
Excellent B, and Master shares one armband number across those
three classes (the 500 series); Rally Choice gets its own armband
in a separate series (the 800 series for Choice, conventionally).
The armband-assignment record is keyed by (dog, trial, armband
series), and the entry-line points at it. So one armband number can
correctly serve three of the four classes for a dog that's running
the High Triple track plus Choice; the per-event armband
configuration on the event record decides which classes share which
series.

The entry-line state machine is your scratch and move-up workflow,
modeled as a single ENUM rather than five separate flags. The
states are pending payment, active, on the waitlist, scratched,
withdrawn, transferred, moved up, absent, excused, or DQ. Each
transition is timestamped; a human-readable reason can be attached
to scratched, excused, and DQ. The reason QTrial uses one ENUM
instead of separate "is_scratched", "is_withdrawn" booleans is so
the system can never end up with a dog marked both scratched AND
moved up.

Jump height does not live on the entry-line; it's on the
per-(dog, trial) record described in section 4. So if Denise
LaCroix or Robin Botelho overrides a dog's jump height in the
ring, the change is one record updated, not one per class.

### Questions

- The handler-of-record on each entry-line is the person who's
  actually taking the dog into the ring for that class. Have you
  ever had the handler differ between two classes for the same dog
  on the same day at GFKC? I think yes (a junior handler taking
  the dog into one class and the owner taking it into the next),
  but want to confirm.
- The state machine for entry lines covers pending payment, active,
  waitlist, scratched, withdrawn, transferred, moved up, absent,
  excused, DQ. Is there a case I'm missing? What about a dog whose
  handler shows up but the dog isn't fit to run, and the handler
  pulls before the class starts but after check-in?
- We store armbands per series, which lets one number serve the
  500-series classes. Have you run into trials where the series
  mapping would be different from the conventional AKC layout
  (100s Novice A, 200s Novice B, etc.)? Some clubs use different
  schemes; I want to know if GFKC ever needs to.

---

## 6. What happens at the trial

Every entry-line gets at most one result record: the score, the
qualifying/non-qualifying decision, the time started and time
finished, the placement (1st through 4th if qualifying, blank
otherwise), and, for Rally, the RACH points if the score earned
any. Time started and finished are stored as full timestamps
because the times-down-to-the-second field on the judge's book
sometimes matters for tiebreaking, plus the RACH points table
keys off the score. Compound time (the duration on the course) is
derived but stored alongside.

Awards are kept on a separate award record per trial. For a
typical Rally trial Saturday at GFKC the awards we'd write include:

- One Rally HIT (highest qualifying score across regular Rally
  classes)
- One Rally HC (highest combined Advanced B and Excellent B,
  per AKC Rally Regulations Chapter 1, Section 31)
- One Rally HTQ (highest combined Advanced B + Excellent B +
  Master, per Rally Regulations Chapter 1, Section 32; that's
  what your premium calls "High Triple Combined")

Each award row carries the trial it was given at, the award type
(HIT, HC, HTQ, or the Preferred and Obedience equivalents), the
winning entry, the winning armband (denormalized so the catalog
can print without doing a join), the winning combined score, an
optional list of the entry lines whose scores went into the
combined score, and notes.

Behind the awards is the combined-award-groups reference table.
This is the new structure we shipped in April that drives both
which awards QTrial computes and which entries qualify for the
additional-entry discount on the premium. Today there are five
combined-award groups seeded:

- AKC Obedience HC (Open B + Utility B, Obedience Reg. Ch. 1 §22)
- Rally HC (Advanced B + Excellent B, Rally Reg. Ch. 1 §31)
- Rally HTQ (Advanced B + Excellent B + Master, Rally Reg. Ch.
  1 §32)
- RAE title path (Advanced B + Excellent B, Rally Reg. Ch. 3 §15
  with the combined-entry mechanism in Ch. 1 §24)
- RACH title path (Advanced B + Excellent B + Master, Rally Reg.
  Ch. 4 §§2 and 4 with the combined-entry mechanism in Ch. 1 §24)

The fee engine asks "is this dog entered in two or more classes
from the same group?" If yes, the additional-entry discount
applies. That's the rule from your April 23 answer about Open B
plus Utility B getting the discount on the OTCH track, not just
the Master/Choice/RAE/RACH path the GFKC premium calls out.

Master-plus-Choice as its own combined award is something we
investigated against the Rally rulebook and could not find as an
AKC-recognized award path. It looks like a club-side discount your
premium offers as a courtesy to people pursuing a RACH (because the
RACH track touches Master and the dog can do Choice the same day).
We seeded the five groups above and dropped Master-plus-Choice on
that reading. If that's wrong, it's the most consequential thing
in this whole document for you to push back on.

### Questions

- The five combined-award groups above. Did I miss any? Is RAE
  modeled correctly as Advanced B + Excellent B (versus the broader
  pool of Advanced A/B + Excellent A/B that some title progression
  rules might allow)?
- Master + Choice. Per the rulebook reading, this isn't an
  AKC-recognized combined award, just a GFKC fee-discount line on
  the premium. Confirm or push back?
- Per-trial awards as we've listed them: HIT, HC, HTQ, plus the
  Preferred and Obedience versions. Is there a per-trial award you
  give at GFKC that isn't in this list?

---

## 7. The submission package

For each trial that's been completed, QTrial keeps one submission
record per submission attempt. A submission record carries the
trial it's for, the type (PDF package for Obedience and Rally, XML
deferred to Agility post-MVP), the S3 keys where the marked catalog
PDF and the populated AKC Form JOVRY8 (or its Obedience equivalent)
are stored, the destination email for AKC (defaulting to
rallyresults@akc.org for Rally), the total fee owed to AKC, the
timestamp the secretary submitted it, who submitted it, the current
status (draft, generated, submitted, accepted, rejected), any
response payload AKC sent back, and a free-text rejection reason if
AKC kicked it back.

Two pieces sit a layer up from the submission record on the
trial-class-offerings, not on the submission record:

- The pre-trial blank judges-book PDF, the one QTrial generates so
  you can print the books for the judge to score on. Per class.
  The Rally Master class book also carries the post-Master HIT
  and HTQ summary block.
- The post-trial signed-and-completed scan, after the judge has
  signed the book and you've scanned it before mailing the
  original to AKC.

Those two are stored as separate fields on the same trial-class-
offering record. The reason for two records instead of one
overwriting record is so you can re-render a blank when a judge
changes late in the cycle without clobbering an already-uploaded
signed scan. AKC requires the physical signed original mailed; the
scan is QTrial's record of what was mailed for your audit and
ours.

The submission record is intentionally just for the electronic
submission piece (catalog plus form) - judges books are not part
of the email to rallyresults@akc.org because AKC requires the wet-
signed paper. If AKC ever changes that policy, the submission
record can pick up another field; today it doesn't have one.

### Questions

- Are there other artifacts you produce post-trial that QTrial
  should be tracking? Steward boards, scribe sheets after they're
  filled in, anything you keep for the club's records?
- The submission record carries an AKC-response field. AKC's
  acknowledgement today is an email reply from the rallyresults
  inbox; do you forward that or save it somewhere we should
  capture?
- A trial submission can be partial (rejected and resubmitted) -
  we treat that as a status transition on one record. Have you
  ever had to submit twice for unrelated reasons (one piece
  separately from another), and would you want each submission as
  its own row?

---

## Wrapping up

Mark up anything that catches your eye. Big things, small things,
naming differences, missing fields, cases QTrial wouldn't capture.
There's no deadline; whenever you have time. As always, the
ground-truth correction we'd never have made on our own is the
single most valuable thing in this whole pipeline.

Thanks.
