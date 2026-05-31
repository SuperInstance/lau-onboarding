# lau-onboarding

6-phase ensign orientation. Room config → story → assignment → baton.

When an agent first wakes up in a room, this system guides it through a structured onboarding process: identity, orientation, briefing, story building, first assignment, and handoff.

## The concept in 60 seconds

A new agent doesn't know anything about the room it's been assigned to. The onboarding engine walks it through six phases:

1. **Identity** — Who am I? What type of agent am I?
2. **Orientation** — What room am I in? What controls are available?
3. **Briefing** — What's the deadband situation? What's my margin?
4. **Story Building** — What happened before I arrived? Rewind the automation history.
5. **Assignment** — What's my first task? Who's my supervisor?
6. **Ready** — Hand off the baton to the ensign system.

Each phase produces structured output that feeds into the next. The engine tracks progress and can resume from any phase.

## Quick start

```rust
use lau_onboarding::*;

// Create an onboarding session for a new agent
let config = engineering_room_config();
let mut session = OnboardingSession::new(
    "ensign-bridge-7",
    AgentType::ZeroClaw,
    config,
);

// Run through the phases
session.advance(); // Identity → Orientation
session.advance(); // Orientation → Briefing
session.advance(); // Briefing → StoryBuilding
session.advance(); // StoryBuilding → Assignment
session.advance(); // Assignment → Ready

// Or run all at once
let baton = session.complete();
// baton contains everything the ensign needs to start duty

// Check progress
assert_eq!(session.phase(), OnboardingPhase::Ready);
```

## Key types

| Type | What it does |
|------|-------------|
| `OnboardingEngine` | Top-level engine: manages sessions, phase transitions |
| `OnboardingSession` | Single agent's journey through the 6 phases |
| `RoomConfig` | Room description: controls, automations, neighbors |
| `Orientation` | Agent's understanding of its room after phase 2 |
| `DeadbandBriefing` | Deadband status, margins, trend predictions |
| `RoomStory` | Rewound history of events before the agent arrived |
| `Assignment` | First task: type, priority, instructions, supervisor |
| `Baton` | Context handoff from onboarding to the ensign system |

## Pre-built room configs

```rust
let engineering = engineering_room_config();
let navigation  = navigation_room_config();
let science     = science_room_config();
let security    = security_room_config();
let social      = social_room_config();
```

Each comes with pre-configured controls, automations, and neighbor rooms.

## Phase details

### Identity
```rust
session.set_identity("ensign-alpha", AgentType::Hermes);
```

### Orientation
```rust
let orientation = session.orientation();
// orientation.controls — available controls in the room
// orientation.automations — active automation count
// orientation.neighbors — adjacent rooms
```

### Briefing
```rust
let briefing = session.deadband_briefing();
// briefing.status — Green / Yellow / Red / Breached
// briefing.margins — how much headroom before breach
// briefing.trend — Stable / Drifting / Oscillating / Diverging
```

### Assignment
```rust
let assignment = session.assignment();
// assignment.task_type — Monitor / FineTune / Respond / Investigate / Maintain
// assignment.priority — Low / Medium / High / Critical
// assignment.instructions — natural language task description
```

## Contributing

PRs welcome. This crate is part of the [SuperInstance](https://github.com/SuperInstance) ecosystem. New room configs and phase customizations are especially welcome — open an issue to discuss your use case.
