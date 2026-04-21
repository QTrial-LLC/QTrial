# QTrial - Domain Glossary

**Status:** Draft v0.1
**Last updated:** 2026-04-19
**Purpose:** Precise definitions of dog-sport and AKC terminology used throughout QTrial. LLMs and new contributors consistently guess these terms wrong if they are not explicitly defined. This document is authoritative; if code, comments, or other docs use a term differently, they are wrong and must be corrected.

---

## The event hierarchy

**Event**
A single approved AKC gathering at a specific location over one or more consecutive days, hosted by one club. An event has one AKC event number *per sport per day per trial* (see Trial below). "Glens Falls Kennel Club Obedience and Rally Trials, November 14-15, 2025" is one event.

**Day**
A calendar date on which the event is being held. An event can be one day or span multiple days (typically 1-3).

**Trial**
A single competition unit within a day for a single sport. A day can contain multiple trials of the same sport, most commonly two: a morning (AM) trial and an afternoon (PM) trial. Each trial has its own AKC event number, its own judge assignments, its own entry limits, its own entry fees, and its own awards (HIT, HC, etc.). This three-level hierarchy (event → day → trial) is a critical data modeling concern.

**Sport** (also called "Event type" in some AKC documents)
The overall discipline. Examples: Obedience, Rally, Agility, Scent Work, Conformation, FastCAT, Barn Hunt, Lure Coursing, Herding, Tracking, Field Trial. A single event can host trials for multiple sports concurrently (e.g., Obedience trial and Rally trial on the same day, same location, different rings).

**Cluster**
An informal term for a series of events held at the same site over several consecutive days, often by different clubs. "The St. Louis cluster" might run for a full week with different clubs hosting different days. QTrial does not need first-class cluster modeling in MVP; clusters are just sequential events from the system's perspective.

## Classes, levels, and divisions

**Class**
A specific competition category within a sport at a specific level. Examples in Obedience: "Novice A", "Novice B", "Open A", "Open B", "Utility A", "Utility B", "Beginner Novice", "Graduate Novice", "Versatility", "Veterans Obedience", "Brace Obedience", "Team Obedience". In Rally: "Rally Novice A", "Rally Novice B", "Rally Intermediate", "Rally Advanced A/B", "Rally Excellent A/B", "Rally Master", "Rally Pairs".

**Level**
The regulatory category of a class. AKC categorizes classes as Regular, Optional Titling, Preferred, or Nonregular. Each has different eligibility rules and titling implications.

**A vs B classes**
Within many classes (Novice, Open, Utility, Rally Novice, Rally Advanced, Rally Excellent), the "A" version is for dogs and/or handlers who have not yet earned the title at that level, and the "B" version is for everyone else. The distinction affects eligibility. A dog cannot be entered in both A and B at the same level on the same day; moving between A and B mid-event is a "transfer" (see below).

**Regular class**
A class at which an AKC title can be earned through normal competition. Novice, Open, Utility are Regular Obedience classes.

**Optional Titling class**
A class that awards an AKC title but is not one of the main progression classes. Beginner Novice, Graduate Novice, Versatility are Optional Titling classes in Obedience.

**Preferred class**
A class that mirrors a Regular class but offers modifications (typically jump height reductions) for dogs that benefit from them. Earns a different title than the Regular class.

**Nonregular class**
A class that does not award AKC titles but is offered for variety, team competition, or fun. Veterans, Sub-Novice, Wildcard, Brace, Team, Rally Pairs are nonregular.

## Qualifying, scoring, and titles

**Q (Qualifying score)**
A score that meets the minimum standard for the class. In Obedience, a Q requires at least 170 out of 200 with no exercise scored below 50% of its value. In Rally, a Q requires at least 70 out of 100. A dog earning a Q "qualified" at that class in that trial.

**NQ (Non-qualifying score)**
A score that fails the qualifying threshold. An NQ dog ran the class but did not qualify.

**Leg**
One qualifying score toward a title. Most AKC Obedience titles require three legs under three different judges (or different judges for at least two of the three, depending on the title). Each Q in a relevant class earns one leg.

**Title**
A credential earned by completing a defined sequence of qualifying performances. Obedience titles include CD (Companion Dog), CDX (Companion Dog Excellent), UD (Utility Dog), UDX (Utility Dog Excellent), OM (Obedience Master), OGM (Obedience Grand Master), BN (Beginner Novice), GN (Graduate Novice), GO (Graduate Open), VER (Versatility). Rally titles include RN (Rally Novice), RA (Rally Advanced), RE (Rally Excellent), RAE (Rally Advanced Excellent), RM (Rally Master), RAE2/RAE3/... (repeated Rally Advanced Excellent).

**Championship titles**
OTCH (Obedience Trial Champion) requires advanced qualifying scores and accumulated OTCH points. RCH (Rally Champion) has similar structure. These are prefix titles - they appear before the dog's registered name.

**Prefix title vs suffix title**
AKC titles are rendered either as prefixes before the registered name (CH, GCH, OTCH, MACH, CT, FC, AFC, etc.) or as suffixes after the registered name (CD, CDX, UD, UDX, BN, RN, RA, RE, etc.). A fully titled dog's name is rendered as "PrefixTitles RegisteredName, SuffixTitles" - e.g., "GCH OTCH Kensington's Moonlight Sonata UDX RAE". This formatting matters for catalogs.

**HIT (High in Trial)**
The top overall Obedience score across all regular classes at a trial. One HIT is awarded per trial.

**HC (High Combined)**
Awarded to the dog with the highest combined Open B and Utility B scores at a trial, when that dog qualifies in both.

**PHIT / PHC (Preferred HIT / Preferred HC)**
The Preferred-class equivalents of HIT and HC.

**RHIT / RHC (Rally HIT / Rally HC)**
Rally-specific honors analogous to the Obedience versions.

**HTQ (Honor Team Qualifier)**
A Rally-specific recognition awarded to qualifying team entries.

**Run-off**
When two or more dogs tie for HIT, HC, or a class placement, they may be required to repeat an exercise to determine the winner.

**Placement**
The finishing order within a class for qualifying dogs: 1st, 2nd, 3rd, 4th. Most clubs award ribbons for the first four placements. Placement is typically only meaningful for qualifying dogs.

## Entries, exhibitors, and handling

**Entry**
A single dog's registration in a single class at a single trial. A dog entered in Novice B and Open A on Saturday AM has two entries.

**Exhibitor**
The person entering the dog. Not always the legal owner.

**Owner**
The person or people on the dog's AKC registration. May not be the exhibitor.

**Co-owner**
A secondary name on the AKC registration, printed in catalogs alongside the primary owner.

**Handler**
The person actually taking the dog into the ring on trial day. Can be the owner, an exhibitor who isn't the owner, or a professional handler.

**Junior Handler**
A handler under the age of 18. Juniors often have reduced entry fees and may enter via a "Junior Showmanship number" or junior handler program.

**Senior Handler**
In some contexts, a handler over a specified age; some clubs offer Senior Handler classes.

**Catalog order**
The order in which dogs are printed in the event catalog. Typically alphabetical by owner or by breed/class/armband.

**Armband**
The numbered band worn by the handler during competition, identifying the dog for the judge and ring crew. Armbands are assigned before the trial and printed for the secretary to distribute. Armband numbering schemes vary (sequential per trial, sequential per event, per class, or across all classes).

**Jump height**
The height of the jump the dog must clear. In Obedience, jump heights are determined by the dog's shoulder height and are typically 4, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36 inches. In Rally, heights are 4, 8, 12, or 16 inches. The dog's jump height is registered with AKC (jump height card) or measured at the first trial.

## Trial-day operations

**Judge's book**
The paper (or electronic) record the judge uses to track scores during the trial. Contains one page per class with pre-printed dog entries (armband, call name, handler, placement placeholders).

**Scribe sheet**
The sheet used by a scorekeeper (scribe) to record exercise-by-exercise scoring during Obedience. Separate from the judge's book.

**Catalog**
The master printed document for the trial listing every entry with full information: armband, breed, registered name with titles, sire, dam, owner, handler. Distributed to exhibitors and judges, archived for AKC records.

**Premium list**
The pre-trial document advertising the trial: dates, location, judges, classes offered, entry fees, closing date, mailing address. Mailed and posted before entries open.

**Judging schedule / Running order**
The sequence in which classes will be judged and the order in which dogs will run within each class.

**Running order within a class**
The sequence of dogs within one class. Can be sorted short-to-tall by jump height (common in Obedience), tall-to-short, randomized, or manually ordered by the secretary.

**Ring**
The physical space where a class is judged. A trial may use multiple rings concurrently (especially Rally).

**Ring steward / gate steward**
Volunteers who manage the flow of dogs at the ring gate - calling dogs, managing the queue, assisting the judge.

## Entry lifecycle events

**Opening date**
The date and time entries begin being accepted.

**Closing date**
The date and time entries stop being accepted (except for day-of move-ups and transfers, which have their own rules).

**Waitlist**
When a class or trial is full, additional entries go on a waitlist. If an entered dog withdraws before the closing date (or sometimes after, depending on club rules), the next waitlisted dog takes its place.

**Move-up**
An exhibitor's written request to move an entered dog up to the next class because the dog earned the title for its current class after the trial's closing date but before the trial begins. For example: a dog earned CD yesterday, and the exhibitor requests move-up from Novice B to Open A for this weekend. Move-ups follow strict AKC rules about timing and documentation.

**Transfer**
A change of a dog's class entry, typically within the same sport at the same level (e.g., Novice A to Novice B). Allowed under specific conditions per AKC Obedience/Rally regulations. Written request required, must be received before a specified deadline (commonly 30 minutes before the start of the trial in question).

**Bitches in season**
AKC allows a full refund (less a small processing fee, typically $5 per class) for a bitch in season, provided written notice is given before the start of the trial. This is a special-case refund category.

**Scratch**
An exhibitor's decision not to run a dog that was entered. A scratched dog may still appear in the catalog but does not compete. Refund policies vary.

**Withdraw**
Similar to scratch but typically indicates the exhibitor removed the entry before the trial began, often during waitlist processing or before closing.

**Absent**
Dog was entered but did not appear at ringside when called.

**Excused**
Dog appeared but the judge excused it from the ring without a qualifying performance (for safety, handler behavior, dog behavior, etc.).

**DQ (Disqualified)**
A more severe version of excusal. The judge disqualifies the dog, typically for aggression or a specific rules violation.

## Teams, braces, and pairs

**Brace**
Two dogs entered and handled together by one handler. Common in Obedience Brace class.

**Team (Obedience Team)**
Four dogs entered as a team, typically judged with modified exercises. Team competition has its own scoring system.

**Rally Pairs**
Two dogs and two handlers completing a Rally course together.

**Rally Team**
Multiple dogs and handlers completing a Rally course together.

**Rally T Challenge / Rally T Challenge Team**
Specialty Rally variants with team and individual versions.

## People and administration

**Trial Secretary**
The volunteer (or paid individual) responsible for running the trial's administrative side: entries, money, paperwork, communication, AKC reporting. Our primary user.

**Trial Chair / Trial Chairperson**
The club's designated lead for the event, often different from the secretary. Handles day-of operations, decisions, and disputes.

**Club**
The AKC-affiliated or AKC-licensed dog club hosting the event. Clubs have member status, license status, or an AKC club number (4-digit).

**Event Chair**
Sometimes synonymous with Trial Chair.

**Superintendent**
A paid professional who runs large events for clubs. Large conformation shows use superintendents. Smaller trials use volunteer trial secretaries. QTrial's primary market is the trial secretary segment, not the superintendent segment.

## AKC-specific identifiers and codes

**AKC event number**
A unique number assigned by AKC to each approved event (technically each trial-day combination). Required for results submission.

**AKC club number**
A 4-digit identifier for the hosting club.

**AKC registration number / AKC #**
The dog's individual AKC identifier. Formatted as two letters followed by up to 8 digits (e.g., "DN12345678"). The letter prefix relates to the breed group - see `tblAKCGroups` in Deborah's current schema for the prefix-to-group mapping.

**PAL / ILP number**
Purebred Alternative Listing (formerly Indefinite Listing Privilege) for unregistered purebreds. Assigned to dogs whose registration is not complete but that are eligible to compete in companion events.

**Canine Partners number**
AKC's registration for mixed-breed dogs, allowing them to compete in companion events. Prefix "MA" or "MB".

**FSS number**
Foundation Stock Service for breeds not yet fully recognized.

**Judge number**
AKC's identifier for each approved judge. Required on AKC reports.

**Junior Handler number (JHN)**
Identifier for a registered Junior Handler under 18.

## Money

**Entry fee (first)**
The fee for the first class a given dog is entered in, within a trial. Typically the larger of the two fees (covers AKC recording fee and processing overhead).

**Entry fee (additional)**
The fee for each subsequent class the same dog is entered in, within the same trial. Typically lower than the first-entry fee.

**AKC recording fee**
The per-entry fee AKC charges to the club for each entry (typically $3.50 first entry, $3 additional - verify current figures). Included in the advertised entry fee.

**AKC event service fee**
A per-event fee the club pays to AKC.

**HIT, HC, and placement awards**
Ribbons, trophies, or small cash awards for placement. QTrial tracks whether they are awarded, not the physical distribution.

**Coupon / discount**
Club-issued credits against entry fees, often for members or to rectify prior refund issues.

## File and document formats QTrial must produce

This is a quick index. Details live in `REQUIREMENTS.md` and `WORKFLOWS.md`.

- **Premium list** - PDF, mailed/posted pre-trial
- **Confirmation email** - per exhibitor, per entry, upon entry submission or acceptance
- **Waitlist email** - when an entry lands on the waitlist
- **Catalog** - PDF, printed for distribution on trial day
- **Judge's book** - PDF, per judge per class, for in-ring scoring
- **Scribe sheet** - PDF, for Obedience exercise-by-exercise scoring
- **Running order** - per class per trial, updated as move-ups and scratches occur
- **Armband assignment sheet** - internal document for secretary
- **Armband cards** - printed, distributed at check-in
- **AKC Report of Trial / Results submission** - either the older printed Form 1 or the modern XML/CSV electronic submission
- **Financial report** - per event, for club accounting

## Things that are easy to get wrong

- **"Trial" vs "event"**: these are not interchangeable. An event can contain multiple trials.
- **"Class" vs "level"**: Novice is a level; Novice A and Novice B are classes within that level.
- **"Entry" vs "entrant"**: an entry is a specific registration (dog in class). An entrant could be a dog or a person depending on context. Prefer "entry" (unambiguous).
- **"Exhibitor" vs "handler" vs "owner"**: not always the same person.
- **"Q" vs "placement"**: A dog can Q and still not place. A dog can place and not Q (rare, only in specific nonregular classes).
- **"Move-up" vs "transfer"**: a move-up advances a dog to the next level (because it earned the title); a transfer changes class within the same level (A to B).
- **"Preferred" classes in Obedience** are unrelated to Preferred in Agility. Different regulatory categories.
- **"HIT" is not per-class**: it is awarded once per trial across all Regular Obedience classes.

## Pending glossary additions (needed from artifacts)

- Rally-specific course terminology (signs, station numbering, course map conventions)
- Scent Work, FastCAT, Agility, Barn Hunt terminology (once those sports are in scope)
- Regional and National Championship event-type distinctions
- UKC equivalents of all the above (for when we extend beyond AKC)

Deborah's screenshots, PDFs, and a walkthrough of one full trial will help fill in anything subtle that Robare and Claude got wrong here.
