# lau-onboarding

Agent onboarding system — guides new ensigns (ZeroClaw, CUDAClaw, Hermes, or custom agents) through room orientation, automation rewind, deadband briefing, and first assignment.

## Usage

```rust
use lau_onboarding::*;

let mut engine = OnboardingEngine::new();
engine.load_room(engineering_room_config());

// Start onboarding
engine.start_onboarding("ensign-1", "engineering", AgentType::ZeroClaw)?;

// Walk through phases
let orientation = engine.provide_orientation("ensign-1");
let briefing = engine.provide_briefing("ensign-1", &events);
let story = engine.build_story("ensign-1", &events);
let assignment = engine.assign("ensign-1");

// Complete and get the baton
let baton = engine.complete_onboarding("ensign-1")?;
```

## Architecture

- **OnboardingSession** — tracks a single agent's progress through phases
- **Orientation** — what the agent learns about the room
- **DeadbandBriefing** — current state and what needs attention
- **RoomStory** — narrative of recent automation events
- **Assignment** — the ensign's first task
- **Baton** — context passed to the agent after onboarding
- **OnboardingEngine** — manages rooms and sessions

## Pre-built Rooms

- `engineering_room_config()` — automation pipelines and reactor controls
- `navigation_room_config()` — course plotting and trajectory
- `science_room_config()` — research lab and analysis
- `security_room_config()` — threat assessment and perimeter
- `social_room_config()` — crew interaction and messaging
