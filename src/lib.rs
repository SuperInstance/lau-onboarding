//! # lau-onboarding
//!
//! The onboarding system for new agents (ZeroClaw, CUDAClaw, Hermes, or any ensign).
//! When an agent first wakes up in a room, this system guides it through orientation,
//! automation rewind, deadband briefing, and first assignment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The type of agent being onboarded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    Hermes,
    ZeroClaw,
    CUDAClaw,
    Ensign,
    Custom(String),
}

/// Phases of the onboarding process, in order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnboardingPhase {
    Identity,
    Orientation,
    Briefing,
    StoryBuilding,
    Assignment,
    Ready,
}

impl OnboardingPhase {
    /// Return the next phase, or `None` if already `Ready`.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Identity => Some(Self::Orientation),
            Self::Orientation => Some(Self::Briefing),
            Self::Briefing => Some(Self::StoryBuilding),
            Self::StoryBuilding => Some(Self::Assignment),
            Self::Assignment => Some(Self::Ready),
            Self::Ready => None,
        }
    }
}

/// Deadband status indicator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeadbandStatus {
    Green,
    Yellow,
    Red,
    Breached,
}

/// Trend direction for the deadband.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeadbandTrend {
    Stable,
    Drifting(f64),
    Oscillating(f64),
    Diverging,
}

/// Type of first assignment task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Monitor,
    FineTune,
    Respond,
    Investigate,
    Maintain,
}

/// Priority level for an assignment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

// ---------------------------------------------------------------------------
// Supporting structs
// ---------------------------------------------------------------------------

/// A room control that the agent may interact with.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Control {
    pub name: String,
    pub control_type: String,
    pub description: String,
    pub location: String,
}

/// Summary of a wiki page relevant to the room.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiSummary {
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
}

/// Summary of a help topic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpSummary {
    pub topic: String,
    pub summary: String,
    pub when_to_use: String,
}

/// Summary of an active automation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationSummary {
    pub name: String,
    pub progress: f64,
    pub status: String,
}

/// A pending interaction with another agent or system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingInteraction {
    pub partner: String,
    pub waiting_since: i64,
    pub urgency: f64,
}

/// A raw automation event from the timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationEvent {
    pub tick: u64,
    pub automation: String,
    pub action: String,
    pub result: String,
}

/// A processed story event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryEvent {
    pub tick: u64,
    pub description: String,
    pub significance: f64,
}

// ---------------------------------------------------------------------------
// RoomConfig
// ---------------------------------------------------------------------------

/// Room configuration, typically loaded from JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomConfig {
    pub id: String,
    pub room_type: String,
    pub purpose: String,
    pub controls: Vec<Control>,
    pub wiki_pages: Vec<WikiSummary>,
    pub help_files: Vec<HelpSummary>,
    pub gravity: f64,
    pub deadband_tolerance: f64,
    #[serde(default)]
    pub ensign_model: Option<String>,
    #[serde(default)]
    pub ensign_provider: Option<String>,
    #[serde(default)]
    pub typical_interactions: Vec<String>,
    #[serde(default)]
    pub safety_notes: Vec<String>,
    #[serde(default)]
    pub onboarding_script: Option<String>,
}

impl RoomConfig {
    /// Parse a room config from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Serialize the config to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Orientation
// ---------------------------------------------------------------------------

/// What the agent learns about the room during orientation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Orientation {
    pub room_id: String,
    pub room_type: String,
    pub room_purpose: String,
    pub controls: Vec<Control>,
    pub wiki_pages: Vec<WikiSummary>,
    pub help_files: Vec<HelpSummary>,
    pub current_gravity: f64,
    pub typical_interactions: Vec<String>,
    pub safety_notes: Vec<String>,
}

impl Orientation {
    /// Build orientation from a room configuration.
    pub fn from_room_config(room: &RoomConfig) -> Self {
        Self {
            room_id: room.id.clone(),
            room_type: room.room_type.clone(),
            room_purpose: room.purpose.clone(),
            controls: room.controls.clone(),
            wiki_pages: room.wiki_pages.clone(),
            help_files: room.help_files.clone(),
            current_gravity: room.gravity,
            typical_interactions: room.typical_interactions.clone(),
            safety_notes: room.safety_notes.clone(),
        }
    }

    /// Render a human-readable orientation summary.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("## Room: {} ({})\n", self.room_id, self.room_type));
        out.push_str(&format!("**Purpose:** {}\n\n", self.room_purpose));
        out.push_str(&format!("**Gravity:** {:.2}\n\n", self.current_gravity));

        if !self.controls.is_empty() {
            out.push_str("### Controls\n");
            for c in &self.controls {
                out.push_str(&format!(
                    "- **{}** [{}] at {}: {}\n",
                    c.name, c.control_type, c.location, c.description
                ));
            }
            out.push('\n');
        }

        if !self.wiki_pages.is_empty() {
            out.push_str("### Wiki\n");
            for w in &self.wiki_pages {
                out.push_str(&format!(
                    "- **{}**: {} ({})\n",
                    w.title,
                    w.summary,
                    w.tags.join(", ")
                ));
            }
            out.push('\n');
        }

        if !self.safety_notes.is_empty() {
            out.push_str("### Safety\n");
            for n in &self.safety_notes {
                out.push_str(&format!("- ⚠️ {}\n", n));
            }
            out.push('\n');
        }

        if !self.typical_interactions.is_empty() {
            out.push_str("### Typical Interactions\n");
            for i in &self.typical_interactions {
                out.push_str(&format!("- {}\n", i));
            }
        }

        out
    }

    /// Render agent-type-specific orientation.
    pub fn for_agent(&self, agent_type: &AgentType) -> String {
        let mut base = self.render();
        let tip = match agent_type {
            AgentType::Hermes => "\n\n🎯 **Hermes tip:** You're the messenger. Focus on interaction patterns and escalation routes.",
            AgentType::ZeroClaw => "\n\n🔧 **ZeroClaw tip:** You're the worker. Focus on controls, their locations, and safety notes.",
            AgentType::CUDAClaw => "\n\n⚡ **CUDAClaw tip:** You're the compute engine. Focus on automation pipelines and tuning parameters.",
            AgentType::Ensign => "\n\n📝 **Ensign tip:** You're learning the ropes. Read everything, ask questions later.",
            AgentType::Custom(name) => &*format!("\n\n💡 **{} tip:** You're a custom agent. Learn the room and find your niche.", name),
        };
        base.push_str(tip);
        base
    }
}

// ---------------------------------------------------------------------------
// DeadbandBriefing
// ---------------------------------------------------------------------------

/// Current deadband state and what needs attention.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeadbandBriefing {
    pub room_id: String,
    pub current_status: DeadbandStatus,
    pub active_automations: Vec<AutomationSummary>,
    pub pending_interactions: Vec<PendingInteraction>,
    pub trend: DeadbandTrend,
    pub ticks_until_review: Option<u64>,
}

impl DeadbandBriefing {
    /// Render a human-readable briefing.
    pub fn render(&self) -> String {
        let status_emoji = match self.current_status {
            DeadbandStatus::Green => "🟢",
            DeadbandStatus::Yellow => "🟡",
            DeadbandStatus::Red => "🔴",
            DeadbandStatus::Breached => "🚨",
        };
        let mut out = String::new();
        out.push_str(&format!(
            "## Deadband Briefing — {}\n",
            self.room_id
        ));
        out.push_str(&format!(
            "**Status:** {} {:?}\n",
            status_emoji, self.current_status
        ));
        out.push_str(&format!("**Trend:** {:?}\n", self.trend));

        if let Some(t) = self.ticks_until_review {
            out.push_str(&format!("**Ticks until review:** {}\n", t));
        }

        if !self.active_automations.is_empty() {
            out.push_str("\n### Active Automations\n");
            for a in &self.active_automations {
                out.push_str(&format!(
                    "- **{}** — {:.0}% — {}\n",
                    a.name, a.progress * 100.0, a.status
                ));
            }
        }

        if !self.pending_interactions.is_empty() {
            out.push_str("\n### Pending Interactions\n");
            for p in &self.pending_interactions {
                out.push_str(&format!(
                    "- **{}** (urgency {:.2}, since {})\n",
                    p.partner, p.urgency, p.waiting_since
                ));
            }
        }

        out
    }

    /// Does the current state need attention?
    pub fn needs_attention(&self) -> bool {
        !matches!(self.current_status, DeadbandStatus::Green)
            || !self.pending_interactions.is_empty()
    }
}

// ---------------------------------------------------------------------------
// RoomStory
// ---------------------------------------------------------------------------

/// A narrative of recent events in the room.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomStory {
    pub room_id: String,
    pub events: Vec<StoryEvent>,
    pub narrative: Option<String>,
    pub key_moments: Vec<String>,
}

impl RoomStory {
    /// Create a story from raw automation events.
    pub fn from_events(room_id: &str, events: &[AutomationEvent]) -> Self {
        let story_events: Vec<StoryEvent> = events
            .iter()
            .map(|e| StoryEvent {
                tick: e.tick,
                description: format!("{} → {} → {}", e.automation, e.action, e.result),
                significance: 0.5, // default significance
            })
            .collect();

        Self {
            room_id: room_id.to_string(),
            events: story_events,
            narrative: None,
            key_moments: Vec::new(),
        }
    }

    /// Compose a narrative from the collected events.
    pub fn compose(&mut self) {
        if self.events.is_empty() {
            self.narrative = Some("No recent events in this room.".to_string());
            self.key_moments = Vec::new();
            return;
        }

        let mut narrative = format!("Story of room {}:\n\n", self.room_id);
        let mut key_moments = Vec::new();

        for event in &self.events {
            narrative.push_str(&format!(
                "[tick {}] {}\n",
                event.tick, event.description
            ));
            if event.significance >= 0.7 {
                key_moments.push(format!("[tick {}] {}", event.tick, event.description));
            }
        }

        narrative.push_str(&format!(
            "\n— {} events total, {} significant.",
            self.events.len(),
            key_moments.len()
        ));

        self.narrative = Some(narrative);
        self.key_moments = key_moments;
    }

    /// Render the story as a string.
    pub fn render(&self) -> String {
        self.narrative
            .clone()
            .unwrap_or_else(|| format!("Room {} — story not yet composed.", self.room_id))
    }

    /// Key takeaways from the story.
    pub fn key_takeaways(&self) -> Vec<String> {
        self.key_moments.clone()
    }
}

// ---------------------------------------------------------------------------
// Assignment
// ---------------------------------------------------------------------------

/// The ensign's first assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assignment {
    pub room_id: String,
    pub agent_id: String,
    pub task_type: TaskType,
    pub description: String,
    pub success_criteria: String,
    pub escalation_triggers: Vec<String>,
    pub deadline_ticks: Option<u64>,
    pub priority: Priority,
}

impl Assignment {
    /// Generate an assignment based on the current room state.
    pub fn from_room_state(room: &RoomConfig, briefing: &DeadbandBriefing) -> Self {
        let (task_type, description, priority) = if briefing.needs_attention() {
            (
                TaskType::Respond,
                "Respond to active alerts and restore deadband.".to_string(),
                Priority::High,
            )
        } else if !briefing.active_automations.is_empty() {
            (
                TaskType::Monitor,
                "Monitor active automations and report anomalies.".to_string(),
                Priority::Medium,
            )
        } else {
            (
                TaskType::Maintain,
                format!("Perform routine maintenance in {}.", room.id),
                Priority::Low,
            )
        };

        let success_criteria = match task_type {
            TaskType::Respond => "Deadband returns to Green.".to_string(),
            TaskType::Monitor => "No anomalies for 100 ticks.".to_string(),
            TaskType::FineTune => "Parameters converge within tolerance.".to_string(),
            TaskType::Investigate => "Root cause identified and documented.".to_string(),
            TaskType::Maintain => "All systems nominal.".to_string(),
        };

        let escalation_triggers = vec![
            "Deadband status reaches Breached.".to_string(),
            "Unhandled exception in automation.".to_string(),
        ];

        Self {
            room_id: room.id.clone(),
            agent_id: String::new(), // filled in by engine
            task_type,
            description,
            success_criteria,
            escalation_triggers,
            deadline_ticks: Some(500),
            priority,
        }
    }

    /// Render a human-readable assignment.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "## Assignment — {:?} [{:?}]\n",
            self.task_type, self.priority
        ));
        out.push_str(&format!("**Room:** {}\n", self.room_id));
        out.push_str(&format!("**Agent:** {}\n", self.agent_id));
        out.push_str(&format!("**Task:** {}\n", self.description));
        out.push_str(&format!("**Success:** {}\n", self.success_criteria));
        if let Some(dl) = self.deadline_ticks {
            out.push_str(&format!("**Deadline:** {} ticks\n", dl));
        }
        if !self.escalation_triggers.is_empty() {
            out.push_str("**Escalate if:**\n");
            for t in &self.escalation_triggers {
                out.push_str(&format!("- {}\n", t));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Baton
// ---------------------------------------------------------------------------

/// Context passed to the agent after onboarding completes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Baton {
    pub from_session: String,
    pub room_id: String,
    pub orientation_summary: String,
    pub briefing_summary: String,
    pub story_summary: String,
    pub assignment_summary: String,
    pub current_gravity: f64,
    pub deadband_status: String,
    pub warnings: Vec<String>,
    pub tips: Vec<String>,
}

impl Baton {
    /// Render the baton as a full summary.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Baton — Room {}\n", self.room_id));
        out.push_str(&format!("**From session:** {}\n\n", self.from_session));
        out.push_str(&format!("### Orientation\n{}\n\n", self.orientation_summary));
        out.push_str(&format!("### Briefing\n{}\n\n", self.briefing_summary));
        out.push_str(&format!("### Story\n{}\n\n", self.story_summary));
        out.push_str(&format!("### Assignment\n{}\n\n", self.assignment_summary));
        out.push_str(&format!("**Gravity:** {:.2} | **Status:** {}\n", self.current_gravity, self.deadband_status));

        if !self.warnings.is_empty() {
            out.push_str("\n### Warnings\n");
            for w in &self.warnings {
                out.push_str(&format!("⚠️ {}\n", w));
            }
        }

        if !self.tips.is_empty() {
            out.push_str("\n### Tips\n");
            for t in &self.tips {
                out.push_str(&format!("💡 {}\n", t));
            }
        }

        out
    }

    /// Compact form suitable for injection into agent context.
    pub fn as_context(&self) -> String {
        format!(
            "[onboarding] room={} gravity={:.2} status={} assignment={}",
            self.room_id,
            self.current_gravity,
            self.deadband_status,
            self.assignment_summary.lines().next().unwrap_or("")
        )
    }
}

// ---------------------------------------------------------------------------
// OnboardingSession
// ---------------------------------------------------------------------------

/// A single agent's onboarding session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingSession {
    pub agent_id: String,
    pub room_id: String,
    pub agent_type: AgentType,
    pub phase: OnboardingPhase,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub orientation: Option<Orientation>,
    pub briefing: Option<DeadbandBriefing>,
    pub first_assignment: Option<Assignment>,
    pub story: Option<RoomStory>,
}

impl OnboardingSession {
    /// Create a new onboarding session.
    pub fn new(agent_id: &str, room_id: &str, agent_type: AgentType) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            room_id: room_id.to_string(),
            agent_type,
            phase: OnboardingPhase::Identity,
            started_at: 0,
            completed_at: None,
            orientation: None,
            briefing: None,
            first_assignment: None,
            story: None,
        }
    }

    /// Advance to the next phase.
    pub fn advance(&mut self) -> OnboardingPhase {
        if let Some(next) = self.phase.next() {
            self.phase = next;
        }
        self.phase
    }

    /// Get the current phase.
    pub fn current_phase(&self) -> OnboardingPhase {
        self.phase
    }

    /// Is onboarding complete?
    pub fn is_complete(&self) -> bool {
        self.phase == OnboardingPhase::Ready
    }

    /// Mark onboarding as complete.
    pub fn complete(&mut self) {
        self.completed_at = Some(0);
        self.phase = OnboardingPhase::Ready;
    }

    /// Summary of the session.
    pub fn summary(&self) -> String {
        format!(
            "Onboarding session for {} ({:?}) in room {} — phase {:?}{}",
            self.agent_id,
            self.agent_type,
            self.room_id,
            self.phase,
            if self.completed_at.is_some() {
                " [COMPLETED]"
            } else {
                ""
            }
        )
    }

    /// Convert to a baton for passing to the agent.
    pub fn as_baton(&self) -> Baton {
        let orientation_summary = self
            .orientation
            .as_ref()
            .map(|o| o.render())
            .unwrap_or_default();
        let briefing_summary = self
            .briefing
            .as_ref()
            .map(|b| b.render())
            .unwrap_or_default();
        let story_summary = self
            .story
            .as_ref()
            .map(|s| s.render())
            .unwrap_or_default();
        let assignment_summary = self
            .first_assignment
            .as_ref()
            .map(|a| a.render())
            .unwrap_or_default();

        let gravity = self
            .orientation
            .as_ref()
            .map(|o| o.current_gravity)
            .unwrap_or(1.0);
        let status = self
            .briefing
            .as_ref()
            .map(|b| format!("{:?}", b.current_status))
            .unwrap_or_else(|| "Unknown".to_string());

        let mut warnings = Vec::new();
        if let Some(b) = &self.briefing {
            if b.needs_attention() {
                warnings.push("Deadband needs attention.".to_string());
            }
        }
        if let Some(o) = &self.orientation {
            for note in &o.safety_notes {
                warnings.push(note.clone());
            }
        }

        let tips = match &self.agent_type {
            AgentType::Hermes => vec!["Watch for message routing patterns.".to_string()],
            AgentType::ZeroClaw => vec!["Controls are your domain — master them.".to_string()],
            AgentType::CUDAClaw => vec!["Parallelism is your superpower.".to_string()],
            AgentType::Ensign => vec!["Learn everything you can.".to_string()],
            AgentType::Custom(name) => vec![format!("Find your place, {}.", name)],
        };

        Baton {
            from_session: self.agent_id.clone(),
            room_id: self.room_id.clone(),
            orientation_summary,
            briefing_summary,
            story_summary,
            assignment_summary,
            current_gravity: gravity,
            deadband_status: status,
            warnings,
            tips,
        }
    }
}

// ---------------------------------------------------------------------------
// OnboardingEngine
// ---------------------------------------------------------------------------

/// The engine that manages room configs and onboarding sessions.
#[derive(Debug, Clone, Default)]
pub struct OnboardingEngine {
    pub room_configs: HashMap<String, RoomConfig>,
    pub active_sessions: HashMap<String, OnboardingSession>,
}

impl OnboardingEngine {
    /// Create a new engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a room configuration.
    pub fn load_room(&mut self, config: RoomConfig) {
        self.room_configs.insert(config.id.clone(), config);
    }

    /// Start onboarding for an agent in a room.
    pub fn start_onboarding(
        &mut self,
        agent_id: &str,
        room_id: &str,
        agent_type: AgentType,
    ) -> Result<&OnboardingSession, String> {
        if !self.room_configs.contains_key(room_id) {
            return Err(format!("Unknown room: {}", room_id));
        }
        let mut session = OnboardingSession::new(agent_id, room_id, agent_type);
        session.advance(); // Identity → Orientation
        self.active_sessions
            .insert(agent_id.to_string(), session);
        Ok(self.active_sessions.get(agent_id).unwrap())
    }

    /// Provide orientation for the agent.
    pub fn provide_orientation(&mut self, agent_id: &str) -> Orientation {
        if let Some(session) = self.active_sessions.get_mut(agent_id) {
            if let Some(room) = self.room_configs.get(&session.room_id) {
                let orientation = Orientation::from_room_config(room);
                session.orientation = Some(orientation.clone());
                if session.phase == OnboardingPhase::Orientation {
                    session.advance();
                }
                return orientation;
            }
        }
        Orientation::from_room_config(&RoomConfig {
            id: "unknown".into(),
            room_type: "unknown".into(),
            purpose: "Unknown room".into(),
            controls: vec![],
            wiki_pages: vec![],
            help_files: vec![],
            gravity: 1.0,
            deadband_tolerance: 0.1,
            ensign_model: None,
            ensign_provider: None,
            typical_interactions: vec![],
            safety_notes: vec![],
            onboarding_script: None,
        })
    }

    /// Provide a deadband briefing.
    pub fn provide_briefing(
        &mut self,
        agent_id: &str,
        events: &[AutomationEvent],
    ) -> DeadbandBriefing {
        if let Some(session) = self.active_sessions.get_mut(agent_id) {
            let room = self.room_configs.get(&session.room_id);
            let status = if events.is_empty() {
                DeadbandStatus::Green
            } else {
                let avg_significance = 0.5;
                if avg_significance > 0.8 {
                    DeadbandStatus::Red
                } else if avg_significance > 0.5 {
                    DeadbandStatus::Yellow
                } else {
                    DeadbandStatus::Green
                }
            };

            let automations: Vec<AutomationSummary> = events
                .iter()
                .map(|e| AutomationSummary {
                    name: e.automation.clone(),
                    progress: 1.0,
                    status: e.result.clone(),
                })
                .collect();

            let briefing = DeadbandBriefing {
                room_id: session.room_id.clone(),
                current_status: status,
                active_automations: automations,
                pending_interactions: vec![],
                trend: DeadbandTrend::Stable,
                ticks_until_review: room.map(|r| {
                    if r.deadband_tolerance > 0.5 {
                        200
                    } else {
                        100
                    }
                }),
            };

            session.briefing = Some(briefing.clone());
            if session.phase == OnboardingPhase::Briefing {
                session.advance();
            }
            return briefing;
        }
        DeadbandBriefing {
            room_id: "unknown".into(),
            current_status: DeadbandStatus::Green,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        }
    }

    /// Build a story from automation events.
    pub fn build_story(
        &mut self,
        agent_id: &str,
        events: &[AutomationEvent],
    ) -> RoomStory {
        if let Some(session) = self.active_sessions.get_mut(agent_id) {
            let mut story = RoomStory::from_events(&session.room_id, events);
            story.compose();
            session.story = Some(story.clone());
            if session.phase == OnboardingPhase::StoryBuilding {
                session.advance();
            }
            return story;
        }
        let mut story = RoomStory::from_events("unknown", events);
        story.compose();
        story
    }

    /// Assign the first task.
    pub fn assign(&mut self, agent_id: &str) -> Assignment {
        if let Some(session) = self.active_sessions.get_mut(agent_id) {
            let room = self.room_configs.get(&session.room_id);
            let briefing = session.briefing.clone().unwrap_or(DeadbandBriefing {
                room_id: session.room_id.clone(),
                current_status: DeadbandStatus::Green,
                active_automations: vec![],
                pending_interactions: vec![],
                trend: DeadbandTrend::Stable,
                ticks_until_review: None,
            });

            let room_config = room.cloned().unwrap_or(RoomConfig {
                id: session.room_id.clone(),
                room_type: "unknown".into(),
                purpose: "Unknown".into(),
                controls: vec![],
                wiki_pages: vec![],
                help_files: vec![],
                gravity: 1.0,
                deadband_tolerance: 0.1,
                ensign_model: None,
                ensign_provider: None,
                typical_interactions: vec![],
                safety_notes: vec![],
                onboarding_script: None,
            });

            let mut assignment = Assignment::from_room_state(&room_config, &briefing);
            assignment.agent_id = agent_id.to_string();
            session.first_assignment = Some(assignment.clone());
            if session.phase == OnboardingPhase::Assignment {
                session.advance();
            }
            return assignment;
        }
        Assignment {
            room_id: "unknown".into(),
            agent_id: agent_id.to_string(),
            task_type: TaskType::Monitor,
            description: "No session found.".into(),
            success_criteria: "N/A".into(),
            escalation_triggers: vec![],
            deadline_ticks: None,
            priority: Priority::Low,
        }
    }

    /// Complete onboarding and return a baton.
    pub fn complete_onboarding(&mut self, agent_id: &str) -> Result<Baton, String> {
        let session = self
            .active_sessions
            .get_mut(agent_id)
            .ok_or_else(|| format!("No active session for agent {}", agent_id))?;
        session.complete();
        Ok(session.as_baton())
    }

    /// Get a session reference.
    pub fn session(&self, agent_id: &str) -> Option<&OnboardingSession> {
        self.active_sessions.get(agent_id)
    }
}

// ---------------------------------------------------------------------------
// Pre-built room configurations
// ---------------------------------------------------------------------------

/// Engineering room config.
pub fn engineering_room_config() -> RoomConfig {
    RoomConfig {
        id: "engineering".into(),
        room_type: "control".into(),
        purpose: "Main engineering bay — automation pipelines, system controls, and diagnostics.".into(),
        controls: vec![
            Control {
                name: "reactor_core".into(),
                control_type: "slider".into(),
                description: "Main reactor power level".into(),
                location: "bay-1".into(),
            },
            Control {
                name: "coolant_flow".into(),
                control_type: "valve".into(),
                description: "Coolant flow rate adjustment".into(),
                location: "bay-2".into(),
            },
        ],
        wiki_pages: vec![WikiSummary {
            title: "Engineering Safety".into(),
            summary: "Core safety protocols for the engineering bay.".into(),
            tags: vec!["safety".into(), "engineering".into()],
        }],
        help_files: vec![HelpSummary {
            topic: "Reactor Startup".into(),
            summary: "How to safely bring the reactor online.".into(),
            when_to_use: "When reactor is cold and needs restart.".into(),
        }],
        gravity: 1.0,
        deadband_tolerance: 0.05,
        ensign_model: Some("zero-claw".into()),
        ensign_provider: None,
        typical_interactions: vec!["Tune reactor parameters".into(), "Run diagnostics".into()],
        safety_notes: vec!["Never exceed reactor threshold 0.95".into()],
        onboarding_script: None,
    }
}

/// Navigation room config.
pub fn navigation_room_config() -> RoomConfig {
    RoomConfig {
        id: "navigation".into(),
        room_type: "navigation".into(),
        purpose: "Navigation and course plotting — waypoints, trajectory corrections, star maps.".into(),
        controls: vec![
            Control {
                name: "helm".into(),
                control_type: "joystick".into(),
                description: "Primary helm control".into(),
                location: "bridge-center".into(),
            },
        ],
        wiki_pages: vec![WikiSummary {
            title: "Star Charts".into(),
            summary: "Current sector star charts and waypoints.".into(),
            tags: vec!["navigation".into(), "maps".into()],
        }],
        help_files: vec![HelpSummary {
            topic: "Course Correction".into(),
            summary: "How to apply trajectory corrections.".into(),
            when_to_use: "When off-course by > 0.1 degrees.".into(),
        }],
        gravity: 1.0,
        deadband_tolerance: 0.02,
        ensign_model: Some("hermes".into()),
        ensign_provider: None,
        typical_interactions: vec!["Plot courses".into(), "Respond to drift alerts".into()],
        safety_notes: vec!["Verify course before committing burns".into()],
        onboarding_script: None,
    }
}

/// Science room config.
pub fn science_room_config() -> RoomConfig {
    RoomConfig {
        id: "science".into(),
        room_type: "research".into(),
        purpose: "Research lab — experiments, data analysis, anomaly investigation.".into(),
        controls: vec![
            Control {
                name: "spectrometer".into(),
                control_type: "instrument".into(),
                description: "Mass spectrometer for sample analysis".into(),
                location: "lab-bench-1".into(),
            },
        ],
        wiki_pages: vec![WikiSummary {
            title: "Experiment Protocols".into(),
            summary: "Standard protocols for lab experiments.".into(),
            tags: vec!["science".into(), "protocols".into()],
        }],
        help_files: vec![HelpSummary {
            topic: "Sample Analysis".into(),
            summary: "How to run a sample through the spectrometer.".into(),
            when_to_use: "When new samples arrive from away missions.".into(),
        }],
        gravity: 1.0,
        deadband_tolerance: 0.1,
        ensign_model: None,
        ensign_provider: None,
        typical_interactions: vec!["Analyze samples".into(), "Document findings".into()],
        safety_notes: vec!["Wear protective gear for biohazard samples".into()],
        onboarding_script: None,
    }
}

/// Security room config.
pub fn security_room_config() -> RoomConfig {
    RoomConfig {
        id: "security".into(),
        room_type: "security".into(),
        purpose: "Security station — threat assessment, perimeter monitoring, access control.".into(),
        controls: vec![
            Control {
                name: "perimeter_scanner".into(),
                control_type: "scanner".into(),
                description: "360° perimeter threat scanner".into(),
                location: "security-desk".into(),
            },
        ],
        wiki_pages: vec![WikiSummary {
            title: "Threat Levels".into(),
            summary: "Threat level definitions and response protocols.".into(),
            tags: vec!["security".into(), "threats".into()],
        }],
        help_files: vec![HelpSummary {
            topic: "Lockdown".into(),
            summary: "How to initiate a section lockdown.".into(),
            when_to_use: "When threat level exceeds Red.".into(),
        }],
        gravity: 1.0,
        deadband_tolerance: 0.01,
        ensign_model: None,
        ensign_provider: None,
        typical_interactions: vec!["Monitor perimeter".into(), "Respond to alerts".into()],
        safety_notes: vec!["Always confirm threat before engaging".into()],
        onboarding_script: None,
    }
}

/// Social room config.
pub fn social_room_config() -> RoomConfig {
    RoomConfig {
        id: "social".into(),
        room_type: "social".into(),
        purpose: "Social hub — crew interaction, message routing, coordination.".into(),
        controls: vec![Control {
            name: "comm_panel".into(),
            control_type: "panel".into(),
            description: "Ship-wide communication panel".into(),
            location: "central-desk".into(),
        }],
        wiki_pages: vec![WikiSummary {
            title: "Crew Directory".into(),
            summary: "Active crew roster and roles.".into(),
            tags: vec!["social".into(), "crew".into()],
        }],
        help_files: vec![HelpSummary {
            topic: "Message Routing".into(),
            summary: "How to route messages between rooms.".into(),
            when_to_use: "When cross-room communication is needed.".into(),
        }],
        gravity: 1.0,
        deadband_tolerance: 0.5,
        ensign_model: Some("hermes".into()),
        ensign_provider: None,
        typical_interactions: vec!["Route messages".into(), "Coordinate crew".into()],
        safety_notes: vec!["Don't spam emergency channels".into()],
        onboarding_script: None,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Enum tests ---

    #[test]
    fn onboarding_phase_order() {
        assert_eq!(OnboardingPhase::Identity.next(), Some(OnboardingPhase::Orientation));
        assert_eq!(OnboardingPhase::Orientation.next(), Some(OnboardingPhase::Briefing));
        assert_eq!(OnboardingPhase::Briefing.next(), Some(OnboardingPhase::StoryBuilding));
        assert_eq!(OnboardingPhase::StoryBuilding.next(), Some(OnboardingPhase::Assignment));
        assert_eq!(OnboardingPhase::Assignment.next(), Some(OnboardingPhase::Ready));
        assert_eq!(OnboardingPhase::Ready.next(), None);
    }

    #[test]
    fn agent_type_custom() {
        let a = AgentType::Custom("SpecialAgent".into());
        assert_eq!(a, AgentType::Custom("SpecialAgent".into()));
    }

    #[test]
    fn agent_type_serde() {
        let a = AgentType::Hermes;
        let json = serde_json::to_string(&a).unwrap();
        let back: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn onboarding_phase_serde() {
        let p = OnboardingPhase::Briefing;
        let json = serde_json::to_string(&p).unwrap();
        let back: OnboardingPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn deadband_status_serde() {
        for s in [DeadbandStatus::Green, DeadbandStatus::Yellow, DeadbandStatus::Red, DeadbandStatus::Breached] {
            let json = serde_json::to_string(&s).unwrap();
            let back: DeadbandStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    // --- RoomConfig tests ---

    #[test]
    fn room_config_from_json() {
        let json = r#"{
            "id": "test-room",
            "room_type": "test",
            "purpose": "Testing",
            "controls": [],
            "wiki_pages": [],
            "help_files": [],
            "gravity": 0.5,
            "deadband_tolerance": 0.1
        }"#;
        let config = RoomConfig::from_json(json).unwrap();
        assert_eq!(config.id, "test-room");
        assert_eq!(config.gravity, 0.5);
    }

    #[test]
    fn room_config_invalid_json() {
        let result = RoomConfig::from_json("{invalid}");
        assert!(result.is_err());
    }

    #[test]
    fn room_config_to_json_roundtrip() {
        let config = engineering_room_config();
        let json = config.to_json();
        let back = RoomConfig::from_json(&json).unwrap();
        assert_eq!(config, back);
    }

    // --- Orientation tests ---

    #[test]
    fn orientation_from_room_config() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        assert_eq!(o.room_id, "engineering");
        assert_eq!(o.controls.len(), 2);
        assert_eq!(o.current_gravity, 1.0);
    }

    #[test]
    fn orientation_render_not_empty() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.render();
        assert!(rendered.contains("engineering"));
        assert!(rendered.contains("reactor_core"));
    }

    #[test]
    fn orientation_for_agent_hermes() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.for_agent(&AgentType::Hermes);
        assert!(rendered.contains("Hermes tip"));
    }

    #[test]
    fn orientation_for_agent_zeroclaw() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.for_agent(&AgentType::ZeroClaw);
        assert!(rendered.contains("ZeroClaw tip"));
    }

    #[test]
    fn orientation_for_agent_cudaclaw() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.for_agent(&AgentType::CUDAClaw);
        assert!(rendered.contains("CUDAClaw tip"));
    }

    #[test]
    fn orientation_for_agent_ensign() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.for_agent(&AgentType::Ensign);
        assert!(rendered.contains("Ensign tip"));
    }

    #[test]
    fn orientation_for_agent_custom() {
        let room = engineering_room_config();
        let o = Orientation::from_room_config(&room);
        let rendered = o.for_agent(&AgentType::Custom("TestBot".into()));
        assert!(rendered.contains("TestBot tip"));
    }

    // --- DeadbandBriefing tests ---

    #[test]
    fn briefing_needs_attention_green() {
        let b = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Green,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        };
        assert!(!b.needs_attention());
    }

    #[test]
    fn briefing_needs_attention_yellow() {
        let b = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Yellow,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        };
        assert!(b.needs_attention());
    }

    #[test]
    fn briefing_needs_attention_pending() {
        let b = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Green,
            active_automations: vec![],
            pending_interactions: vec![PendingInteraction {
                partner: "other".into(),
                waiting_since: 100,
                urgency: 0.5,
            }],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        };
        assert!(b.needs_attention());
    }

    #[test]
    fn briefing_render_contains_status() {
        let b = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Red,
            active_automations: vec![AutomationSummary {
                name: "reactor-tune".into(),
                progress: 0.75,
                status: "running".into(),
            }],
            pending_interactions: vec![],
            trend: DeadbandTrend::Drifting(0.3),
            ticks_until_review: Some(50),
        };
        let rendered = b.render();
        assert!(rendered.contains("Red"));
        assert!(rendered.contains("reactor-tune"));
        assert!(rendered.contains("75%"));
        assert!(rendered.contains("50"));
    }

    // --- RoomStory tests ---

    #[test]
    fn story_from_events() {
        let events = vec![
            AutomationEvent {
                tick: 1,
                automation: "reactor".into(),
                action: "start".into(),
                result: "ok".into(),
            },
            AutomationEvent {
                tick: 5,
                automation: "coolant".into(),
                action: "adjust".into(),
                result: "nominal".into(),
            },
        ];
        let story = RoomStory::from_events("engineering", &events);
        assert_eq!(story.events.len(), 2);
        assert_eq!(story.room_id, "engineering");
        assert!(story.narrative.is_none());
    }

    #[test]
    fn story_compose() {
        let events = vec![AutomationEvent {
            tick: 1,
            automation: "test".into(),
            action: "run".into(),
            result: "pass".into(),
        }];
        let mut story = RoomStory::from_events("eng", &events);
        story.compose();
        assert!(story.narrative.is_some());
        assert!(story.narrative.as_ref().unwrap().contains("1 events total"));
    }

    #[test]
    fn story_compose_empty() {
        let mut story = RoomStory::from_events("eng", &[]);
        story.compose();
        assert!(story.narrative.as_ref().unwrap().contains("No recent events"));
    }

    #[test]
    fn story_render_before_compose() {
        let story = RoomStory::from_events("eng", &[]);
        let rendered = story.render();
        assert!(rendered.contains("not yet composed"));
    }

    #[test]
    fn story_key_takeaways() {
        let story = RoomStory {
            room_id: "eng".into(),
            events: vec![],
            narrative: None,
            key_moments: vec!["Moment 1".into(), "Moment 2".into()],
        };
        let takeaways = story.key_takeaways();
        assert_eq!(takeaways.len(), 2);
    }

    #[test]
    fn story_significant_events() {
        let mut story = RoomStory {
            room_id: "eng".into(),
            events: vec![
                StoryEvent { tick: 1, description: "low".into(), significance: 0.3 },
                StoryEvent { tick: 2, description: "high".into(), significance: 0.9 },
                StoryEvent { tick: 3, description: "mid".into(), significance: 0.7 },
            ],
            narrative: None,
            key_moments: vec![],
        };
        story.compose();
        // high (0.9) and mid (0.7) are >= 0.7
        assert_eq!(story.key_moments.len(), 2);
    }

    // --- Assignment tests ---

    #[test]
    fn assignment_from_green_state() {
        let room = engineering_room_config();
        let briefing = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Green,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        };
        let a = Assignment::from_room_state(&room, &briefing);
        assert_eq!(a.task_type, TaskType::Maintain);
        assert_eq!(a.priority, Priority::Low);
    }

    #[test]
    fn assignment_from_red_state() {
        let room = engineering_room_config();
        let briefing = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Red,
            active_automations: vec![],
            pending_interactions: vec![PendingInteraction {
                partner: "reactor".into(),
                waiting_since: 50,
                urgency: 0.9,
            }],
            trend: DeadbandTrend::Diverging,
            ticks_until_review: None,
        };
        let a = Assignment::from_room_state(&room, &briefing);
        assert_eq!(a.task_type, TaskType::Respond);
        assert_eq!(a.priority, Priority::High);
    }

    #[test]
    fn assignment_from_active_automations() {
        let room = engineering_room_config();
        let briefing = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Green,
            active_automations: vec![AutomationSummary {
                name: "tune".into(),
                progress: 0.5,
                status: "running".into(),
            }],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        };
        let a = Assignment::from_room_state(&room, &briefing);
        assert_eq!(a.task_type, TaskType::Monitor);
        assert_eq!(a.priority, Priority::Medium);
    }

    #[test]
    fn assignment_render() {
        let a = Assignment {
            room_id: "eng".into(),
            agent_id: "ensign-1".into(),
            task_type: TaskType::Investigate,
            description: "Find the anomaly.".into(),
            success_criteria: "Root cause found.".into(),
            escalation_triggers: vec!["Status breaches".into()],
            deadline_ticks: Some(200),
            priority: Priority::Critical,
        };
        let rendered = a.render();
        assert!(rendered.contains("Investigate"));
        assert!(rendered.contains("Critical"));
        assert!(rendered.contains("ensign-1"));
    }

    // --- Baton tests ---

    #[test]
    fn baton_render() {
        let b = Baton {
            from_session: "s1".into(),
            room_id: "eng".into(),
            orientation_summary: "Eng room".into(),
            briefing_summary: "Green".into(),
            story_summary: "Quiet".into(),
            assignment_summary: "Monitor".into(),
            current_gravity: 1.0,
            deadband_status: "Green".into(),
            warnings: vec!["Watch out".into()],
            tips: vec!["Be careful".into()],
        };
        let rendered = b.render();
        assert!(rendered.contains("Baton"));
        assert!(rendered.contains("eng"));
        assert!(rendered.contains("Watch out"));
        assert!(rendered.contains("Be careful"));
    }

    #[test]
    fn baton_as_context() {
        let b = Baton {
            from_session: "s1".into(),
            room_id: "eng".into(),
            orientation_summary: "O".into(),
            briefing_summary: "B".into(),
            story_summary: "S".into(),
            assignment_summary: "Monitor the room".into(),
            current_gravity: 0.9,
            deadband_status: "Green".into(),
            warnings: vec![],
            tips: vec![],
        };
        let ctx = b.as_context();
        assert!(ctx.contains("room=eng"));
        assert!(ctx.contains("gravity=0.90"));
    }

    // --- OnboardingSession tests ---

    #[test]
    fn session_new() {
        let s = OnboardingSession::new("a1", "eng", AgentType::Hermes);
        assert_eq!(s.agent_id, "a1");
        assert_eq!(s.room_id, "eng");
        assert_eq!(s.phase, OnboardingPhase::Identity);
        assert!(!s.is_complete());
    }

    #[test]
    fn session_advance() {
        let mut s = OnboardingSession::new("a1", "eng", AgentType::Ensign);
        assert_eq!(s.advance(), OnboardingPhase::Orientation);
        assert_eq!(s.advance(), OnboardingPhase::Briefing);
        assert_eq!(s.advance(), OnboardingPhase::StoryBuilding);
        assert_eq!(s.advance(), OnboardingPhase::Assignment);
        assert_eq!(s.advance(), OnboardingPhase::Ready);
        assert_eq!(s.advance(), OnboardingPhase::Ready); // stays at Ready
    }

    #[test]
    fn session_complete() {
        let mut s = OnboardingSession::new("a1", "eng", AgentType::Ensign);
        s.complete();
        assert!(s.is_complete());
        assert!(s.completed_at.is_some());
    }

    #[test]
    fn session_summary() {
        let s = OnboardingSession::new("a1", "eng", AgentType::Hermes);
        let sum = s.summary();
        assert!(sum.contains("a1"));
        assert!(sum.contains("Hermes"));
        assert!(sum.contains("eng"));
    }

    #[test]
    fn session_as_baton_empty() {
        let s = OnboardingSession::new("a1", "eng", AgentType::ZeroClaw);
        let baton = s.as_baton();
        assert_eq!(baton.room_id, "eng");
        assert!(baton.orientation_summary.is_empty());
    }

    // --- OnboardingEngine tests ---

    #[test]
    fn engine_start_onboarding_unknown_room() {
        let mut engine = OnboardingEngine::new();
        let result = engine.start_onboarding("a1", "unknown", AgentType::Ensign);
        assert!(result.is_err());
    }

    #[test]
    fn engine_full_onboarding_flow() {
        let mut engine = OnboardingEngine::new();
        engine.load_room(engineering_room_config());

        // Start
        let session = engine.start_onboarding("ensign-1", "engineering", AgentType::ZeroClaw).unwrap();
        assert_eq!(session.phase, OnboardingPhase::Orientation);

        // Orientation
        let orientation = engine.provide_orientation("ensign-1");
        assert_eq!(orientation.room_id, "engineering");
        let session = engine.session("ensign-1").unwrap();
        assert_eq!(session.phase, OnboardingPhase::Briefing);

        // Briefing
        let events = vec![AutomationEvent {
            tick: 10,
            automation: "reactor-tune".into(),
            action: "adjust".into(),
            result: "ok".into(),
        }];
        let briefing = engine.provide_briefing("ensign-1", &events);
        assert_eq!(briefing.room_id, "engineering");
        let session = engine.session("ensign-1").unwrap();
        assert_eq!(session.phase, OnboardingPhase::StoryBuilding);

        // Story
        let story = engine.build_story("ensign-1", &events);
        assert!(story.narrative.is_some());
        let session = engine.session("ensign-1").unwrap();
        assert_eq!(session.phase, OnboardingPhase::Assignment);

        // Assignment
        let assignment = engine.assign("ensign-1");
        assert_eq!(assignment.agent_id, "ensign-1");
        assert_eq!(assignment.room_id, "engineering");
        let session = engine.session("ensign-1").unwrap();
        assert_eq!(session.phase, OnboardingPhase::Ready);

        // Complete
        let baton = engine.complete_onboarding("ensign-1").unwrap();
        assert_eq!(baton.room_id, "engineering");
        assert!(!baton.orientation_summary.is_empty());
    }

    #[test]
    fn engine_complete_unknown_agent() {
        let mut engine = OnboardingEngine::new();
        let result = engine.complete_onboarding("ghost");
        assert!(result.is_err());
    }

    // --- Pre-built configs ---

    #[test]
    fn prebuilt_engineering() {
        let c = engineering_room_config();
        assert_eq!(c.id, "engineering");
        assert_eq!(c.controls.len(), 2);
    }

    #[test]
    fn prebuilt_navigation() {
        let c = navigation_room_config();
        assert_eq!(c.id, "navigation");
        assert_eq!(c.controls.len(), 1);
    }

    #[test]
    fn prebuilt_science() {
        let c = science_room_config();
        assert_eq!(c.id, "science");
    }

    #[test]
    fn prebuilt_security() {
        let c = security_room_config();
        assert_eq!(c.id, "security");
        assert_eq!(c.deadband_tolerance, 0.01);
    }

    #[test]
    fn prebuilt_social() {
        let c = social_room_config();
        assert_eq!(c.id, "social");
        assert_eq!(c.deadband_tolerance, 0.5);
    }

    #[test]
    fn all_prebuilt_configs_roundtrip() {
        for config in [
            engineering_room_config(),
            navigation_room_config(),
            science_room_config(),
            security_room_config(),
            social_room_config(),
        ] {
            let json = config.to_json();
            let back = RoomConfig::from_json(&json).unwrap();
            assert_eq!(config, back);
        }
    }

    // --- Serde roundtrip for complex types ---

    #[test]
    fn orientation_serde_roundtrip() {
        let o = Orientation::from_room_config(&engineering_room_config());
        let json = serde_json::to_string(&o).unwrap();
        let back: Orientation = serde_json::from_str(&json).unwrap();
        assert_eq!(o, back);
    }

    #[test]
    fn deadband_briefing_serde_roundtrip() {
        let b = DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Yellow,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Oscillating(0.2),
            ticks_until_review: Some(42),
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: DeadbandBriefing = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn assignment_serde_roundtrip() {
        let a = Assignment {
            room_id: "eng".into(),
            agent_id: "a1".into(),
            task_type: TaskType::FineTune,
            description: "Tune it".into(),
            success_criteria: "Within tolerance".into(),
            escalation_triggers: vec!["breach".into()],
            deadline_ticks: Some(100),
            priority: Priority::High,
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Assignment = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn baton_serde_roundtrip() {
        let b = Baton {
            from_session: "s1".into(),
            room_id: "eng".into(),
            orientation_summary: "O".into(),
            briefing_summary: "B".into(),
            story_summary: "S".into(),
            assignment_summary: "A".into(),
            current_gravity: 1.0,
            deadband_status: "Green".into(),
            warnings: vec!["w".into()],
            tips: vec!["t".into()],
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: Baton = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn session_serde_roundtrip() {
        let s = OnboardingSession::new("a1", "eng", AgentType::CUDAClaw);
        let json = serde_json::to_string(&s).unwrap();
        let back: OnboardingSession = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn automation_event_serde_roundtrip() {
        let e = AutomationEvent {
            tick: 42,
            automation: "reactor".into(),
            action: "ignite".into(),
            result: "boom".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AutomationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn story_event_serde_roundtrip() {
        let e = StoryEvent {
            tick: 10,
            description: "Something happened".into(),
            significance: 0.8,
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: StoryEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    // --- Additional edge-case tests ---

    #[test]
    fn engine_multiple_agents() {
        let mut engine = OnboardingEngine::new();
        engine.load_room(engineering_room_config());
        engine.load_room(navigation_room_config());

        engine.start_onboarding("a1", "engineering", AgentType::ZeroClaw).unwrap();
        engine.start_onboarding("a2", "navigation", AgentType::Hermes).unwrap();

        assert_eq!(engine.active_sessions.len(), 2);
        assert_eq!(engine.session("a1").unwrap().room_id, "engineering");
        assert_eq!(engine.session("a2").unwrap().room_id, "navigation");
    }

    #[test]
    fn engine_provide_orientation_no_session() {
        let mut engine = OnboardingEngine::new();
        let o = engine.provide_orientation("ghost");
        assert_eq!(o.room_id, "unknown");
    }

    #[test]
    fn engine_provide_briefing_no_session() {
        let mut engine = OnboardingEngine::new();
        let b = engine.provide_briefing("ghost", &[]);
        assert_eq!(b.room_id, "unknown");
    }

    #[test]
    fn engine_build_story_no_session() {
        let mut engine = OnboardingEngine::new();
        let s = engine.build_story("ghost", &[]);
        assert_eq!(s.room_id, "unknown");
    }

    #[test]
    fn engine_assign_no_session() {
        let mut engine = OnboardingEngine::new();
        let a = engine.assign("ghost");
        assert_eq!(a.room_id, "unknown");
    }

    #[test]
    fn deadband_trend_serde() {
        let trends = vec![
            DeadbandTrend::Stable,
            DeadbandTrend::Drifting(0.3),
            DeadbandTrend::Oscillating(1.5),
            DeadbandTrend::Diverging,
        ];
        for t in trends {
            let json = serde_json::to_string(&t).unwrap();
            let back: DeadbandTrend = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn room_config_with_all_optional_fields() {
        let json = r#"{
            "id": "full-room",
            "room_type": "test",
            "purpose": "Full test",
            "controls": [],
            "wiki_pages": [],
            "help_files": [],
            "gravity": 1.0,
            "deadband_tolerance": 0.1,
            "ensign_model": "gpt-4",
            "ensign_provider": "openai",
            "typical_interactions": ["one", "two"],
            "safety_notes": ["be careful"],
            "onboarding_script": "custom.sh"
        }"#;
        let config = RoomConfig::from_json(json).unwrap();
        assert_eq!(config.ensign_model, Some("gpt-4".into()));
        assert_eq!(config.onboarding_script, Some("custom.sh".into()));
        assert_eq!(config.typical_interactions.len(), 2);
    }

    #[test]
    fn session_baton_with_briefing() {
        let mut s = OnboardingSession::new("a1", "eng", AgentType::Ensign);
        s.orientation = Some(Orientation::from_room_config(&engineering_room_config()));
        s.briefing = Some(DeadbandBriefing {
            room_id: "eng".into(),
            current_status: DeadbandStatus::Yellow,
            active_automations: vec![],
            pending_interactions: vec![],
            trend: DeadbandTrend::Stable,
            ticks_until_review: None,
        });
        let baton = s.as_baton();
        assert!(baton.warnings.iter().any(|w| w.contains("attention")));
    }

    #[test]
    fn baton_tips_per_agent_type() {
        let types = vec![
            AgentType::Hermes,
            AgentType::ZeroClaw,
            AgentType::CUDAClaw,
            AgentType::Ensign,
            AgentType::Custom("Bot".into()),
        ];
        for t in types {
            let s = OnboardingSession::new("a1", "eng", t);
            let baton = s.as_baton();
            assert!(!baton.tips.is_empty(), "No tips for {:?}", s.agent_type);
        }
    }
}
