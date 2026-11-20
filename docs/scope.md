# ExoNovel — Product Scope

> **Status**: Drafted from initial user brief and requested publication-format research. Open for interview refinement.

---

## 1. Product Vision & Core Concept

**ExoNovel** is a collaborative creative writing platform and application where **every contributor retains ownership of their own content** even while creating works together in shared projects.

### Core Pillars

- **Individual Content Ownership in Shared Projects**: Authors own the creative elements and contributions they author, even when collaborating inside a shared project.
- **Modular / Dimensional Sharing**: The design encourages authors to share distinct dimensions of a project independently—for example, publishing or sharing an individual character biography so other authors can discover it, take interest, and potentially include that character in their own novels.
- **Google Docs-Style Collaboration & Live Co-Editing**: Standard sharing and permission actions alongside live, simultaneous co-editing.
- **Writer-Centered Planning & Creation**: UI and workflows built around how writers actually plan, organize, and draft works (novels, screenplays, and other formats) rather than forcing generic document editing alone.

---

## 2. Platform & Technical Scope

### Target Delivery Platforms

1. **Website (Web Application)** — Initial primary release.
2. **Desktop Installations** (all three major desktop operating systems):
   - **Linux**
   - **macOS**
   - **Windows**
3. **Mobile Installations** (both major mobile operating systems):
   - **Android**
   - **iOS**

### Data Storage

- **Database**: Hosted **PostgreSQL** (`pgsql`) solution.

---

## 3. Top 5 Publication Formats & Natural Structures

Based on industry reach, creative-writing community activity, and publishing volume, the five foundational publication formats for ExoNovel—along with the natural structures writers use to **organize/plan** them and **publish** the finished work—are defined below.

### Format 1: Novels (Long-Form Fiction & Nonfiction)

Long-form narrative works representing the commercial backbone of traditional, indie, and self-publishing.

#### Natural Planning & Authoring Structure

- **Foundational Story Bible**:
  - **Characters**: Profiles, biographies, arcs, motivations, flaws, wounds, and relationship webs.
  - **Worldbuilding / Setting Bible**: Locations, timelines, lore, rules/systems (kept largely behind the scenes as an "iceberg" during drafting).
  - **Plot / Outline**: High-level roadmap (e.g., Three-Act arc, beat sheets, Snowflake method, or loose bullet outlines) mapping Inciting Incident, Midpoint, Climax, and Resolution.
- **Drafting Hierarchy**:
  - **Parts / Acts** (macro shifts in time, POV, or narrative phase)
  - **Chapters** (pacing containers, typically 1,500–5,000 words, ending on hooks or transitions)
  - **Scenes & Sequels** (atomic action/conflict units paired with character reaction/processing units)

#### Published Output Structure

- Front matter (title page, dedication, epigraph, table of contents when applicable)
- Sequenced **Parts** (optional) and **Chapters** with smoothed transitions and scene breaks
- Back matter (acknowledgments, author bio, series notes)

---

### Format 2: Articles & Creative Nonfiction (Essays, Memoirs, Feature Writing)

High-consumption nonfiction spanning narrative journalism, personal essays, cultural commentary, and memoirs.

#### Natural Planning & Authoring Structure

- **Core Angle & Purpose**: Central thesis (essays), news/value hook (articles), or underlying thematic truth (creative nonfiction/memoir).
- **Structural Outline**:
  1. **Hook / Lead**: Sensory scene lead, surprising statistic/hook lead, or thesis summary lead.
  2. **Pivot / Context**: Transition from opening hook into the core subject.
  3. **Body Sections**: Arguments, evidence, anecdotes, or narrative scenes grouped under logical signposts (`H2` / `H3` subheadings), using either an inverted pyramid (journalism), logical progression (essays), or braided scene-and-reflection arcs (creative nonfiction).
  4. **Turning Point / Synthesis**: Addressing counterarguments or delivering peak emotional/intellectual insight.
  5. **Conclusion**: Synthesis, takeaway, or resonant closing image.
- **Research & Source Notes**: Citations, interviews, fact-check notes, and reference clippings attached to sections.

#### Published Output Structure

- Headline & Deck / Subtitle
- Byline & Metadata
- Opening Lead
- Subheaded Body Sections (often paired with pull quotes, figures, or citations)
- Closing Conclusion / Call to Action

---

### Format 3: Short Stories & Flash Fiction

Compact narrative prose ranging from flash fiction (typically under 1,000 words) to traditional short stories (roughly 3,000–8,000 words), prominent in literary magazines, anthologies, and digital newsletters.

#### Natural Planning & Authoring Structure

- **Single-Conflict Anchor**: One central tension, compressed timeline, and tight cast (avoiding sprawling multi-plot exposition).
- **Focused Outline**:
  1. ***In Media Res* Hook & Grounding**: Immediate character, setting, and active pressure.
  2. **Inciting Pressure & Backstory Tease**: History implied through concrete details rather than exposition ("Iceberg Technique").
  3. **Escalation / Micro-Turning Point**: Rising friction leading to a shift in dynamic.
  4. **Climax / Flashpoint**: Decisive choice or realization.
  5. **Resonance**: Lingering image or shift rather than heavy-handed wrap-up.
- **Collection / Anthology Grouping** (optional): Grouping related stories under a shared theme or world.

#### Published Output Structure

- Title & Author Byline
- Tight, continuous prose (single scene or minimal section breaks `* * *`)
- Published individually (magazines/zines) or sequenced inside an **Anthology / Short Story Collection**

---

### Format 4: Screenplays & Scripts (Film, Television, Audio/Podcast)

Visual and auditory blueprints governed by strict industry formatting where roughly **1 script page equals 1 minute of screen time**.

#### Natural Planning & Authoring Structure

- **Pre-Script Development Documents**:
  - **Logline & Premise**: One-to-two sentence core hook.
  - **Treatment**: Present-tense prose summary (2–15+ pages) telling the story end-to-end without full dialogue.
  - **Beat Sheet & Index Cards**: Scene-by-scene cards mapped across Acts (e.g., Act I Setup, Act II Confrontation/Midpoint, Act III Resolution) or episodic A/B/C storylines.
  - **Character & Location Breakdowns**: Speaking roles, voice attributes, recurring sets/locations.
- **Script Block Elements**:
  - **Sluglines (Scene Headings)**: `[INT./EXT.] [LOCATION] - [TIME OF DAY]`
  - **Action Lines**: Present-tense, concise visual/auditory descriptions
  - **Character Cues** (including `(V.O.)`, `(O.S.)`)
  - **Dialogue** & **Parentheticals**
  - **Transitions** (used sparingly)

#### Published / Production Output Structure

- Title Page (Title, Written By, Draft Date / Contact)
- Industry-standard fixed-pitch layout (12pt Courier / Courier Prime, 1.5" left margin, 1" top/bottom/right margins)
- Sequenced numbered or unnumbered scenes ready for table reads, pitch packets, or production breakdowns

---

### Format 5: Poetry (Individual Poems, Chapbooks & Collections)

Line- and stanza-driven verse spanning structured/formal poetry (sonnets, villanelles, haikus), free verse, and spoken-word/digital forms, published individually or curated into chapbooks and full-length collections.

#### Natural Planning & Authoring Structure

- **Line & Stanza Craft**:
  - **Lines & Line Breaks**: Controlling breath, speed, *enjambment*, and *end-stopped* emphasis.
  - **Stanzas**: Structural groupings (couplets, tercets, quatrains, etc.) and intentional white space.
  - **Sound & Form Notes**: Meter (e.g., iambic pentameter), rhyme schemes, or free-verse rhythmic cadence; thematic spark $\rightarrow$ shift (*volta*) $\rightarrow$ resonance.
- **Manuscript / Collection Curation**:
  - Gathering a pool of poems and sequencing by thematic, emotional, or narrative arc.
  - Balancing visual density and white space across pages.
  - Sizing by tier: **Micro-chapbook** (~5–10 poems), **Chapbook** (~15–30 poems), or **Full-Length Collection** (~40–80+ poems).

#### Published Output Structure

- **Single Poem**: Title, dedication/epigraph (optional), exact preserved lineation, indentation, and stanza spacing.
- **Chapbook / Collection**: Title page, table of contents, optional numbered/titled sections, one poem per page start, and notes/acknowledgments.

---

## 4. Open Interview Questions (Next Steps)

- Do you want to adjust, replace, or reorder any of these **five** publication formats?
- Which of the five formats would you like to interview on **in depth** first (e.g., mapping its exact planning dimensions, reusable components, and collaboration ownership rules)?
