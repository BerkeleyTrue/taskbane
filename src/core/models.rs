use std::str::FromStr;

use derive_more::{Constructor, Eq, PartialEq};
use itertools::Itertools;
use serde::Serialize;
use sqlx::prelude::FromRow;
use taskchampion::{
    chrono::{DateTime, Duration, Local, Utc},
    Status, Tag, Task,
};
use tracing::info;
use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

#[derive(Debug, Clone, Serialize, FromRow, PartialEq, Eq, Constructor)]
pub struct User {
    pub id: Uuid,
    #[eq(skip)]
    pub username: String,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn with_username(&mut self, username: String) -> &mut Self {
        self.username = username;
        self
    }
}

#[derive(Debug, Clone)]
pub struct UserAuth {
    pub user_id: Uuid,
    pub passkeys: Vec<Passkey>,
    pub registration: Option<PasskeyRegistration>,
    pub authentication: Option<PasskeyAuthentication>,
}

impl UserAuth {
    pub fn new(user_id: Uuid, registration: PasskeyRegistration) -> Self {
        UserAuth {
            user_id,
            registration: Some(registration),
            authentication: None,
            passkeys: Vec::new(),
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn registration(&self) -> Option<PasskeyRegistration> {
        self.registration.clone()
    }
    pub fn authentication(&self) -> Option<PasskeyAuthentication> {
        self.authentication.clone()
    }
    pub fn passkey(&self) -> Vec<Passkey> {
        self.passkeys.clone()
    }
}

// urgency coefficient
const NEXT_TAG: f64 = 15.0;
const OVERDUE: f64 = 12.0;
const BLOCKING_OTHERS: f64 = 8.0;
const PRIORITY_HIGH: f64 = 6.0;
const PRIORITY_MEDIUM: f64 = 3.9;
const ACTIVE_STARTED: f64 = 4.0;
const TASK_AGE: f64 = 2.0;
const PROJECT_ASSIGNED: f64 = 1.0;
const WAITING: f64 = -3.0;
const BLOCKED: f64 = -5.0;

#[derive(Debug, Clone)]
pub enum TaskDueStatus {
    Due,
    DueSoon,
    DueToday,
    OverDue,
    Not,
}

#[derive(Debug, Clone)]
pub struct TaskDto {
    pub id: usize,
    pub status: Status,
    pub description: String,
    pub due: Option<String>,
    pub due_status: TaskDueStatus,

    pub is_blocked: bool,
    pub is_blocking: bool,

    pub tags: String,
    pub deps: String,

    pub priority: String,
    pub urgency: f64,
}

impl TaskDto {
    pub fn from(id: usize, value: Task, deps: Vec<usize>) -> Self {
        let next_urg = Tag::from_str("next")
            .map(|tag| if value.has_tag(&tag) { NEXT_TAG } else { 0. })
            .unwrap_or(0.);

        let due_urg = value.get_due().map(Self::due_urgency).unwrap_or_default();
        let due_status = value
            .get_due()
            .map(Self::due_status)
            .unwrap_or(TaskDueStatus::Not);
        let due = value.get_due().map(Self::due);
        let blocking_urg = if value.is_blocking() {
            BLOCKING_OTHERS
        } else {
            0.
        };
        let pri_urg = match value.get_priority() {
            "m" => PRIORITY_MEDIUM,
            "h" => PRIORITY_HIGH,
            _ => 0.,
        };
        let act_urg = if value.is_active() {
            ACTIVE_STARTED
        } else {
            0.
        };

        let age_urg = value
            .get_entry()
            .map(|age| (Utc::now() - age).num_days().clamp(0, 365) as f64)
            .map(|age_days| (age_days / 365.0) * TASK_AGE)
            .unwrap_or_default();

        let proj_urg = value
            .get_user_defined_attribute("project")
            .map(|_| PROJECT_ASSIGNED)
            .unwrap_or_default();
        let wait_urg = if value.is_waiting() { WAITING } else { 0. };
        let block_urg = if value.is_blocked() { BLOCKED } else { 0. };

        Self {
            id,
            status: value.get_status(),
            description: value.get_description().to_owned(),
            priority: value.get_priority().to_owned(),
            is_blocked: value.is_blocked(),
            is_blocking: value.is_blocking(),
            tags: value.get_tags().filter(|tag| tag.is_user()).join(" "),
            deps: deps.iter().join(" "),
            due,
            due_status,
            urgency: next_urg
                + due_urg
                + blocking_urg
                + pri_urg
                + act_urg
                + age_urg
                + proj_urg
                + wait_urg
                + block_urg,
        }
    }

    fn due_urgency(due: DateTime<Utc>) -> f64 {
        let days_until_due = (due - Utc::now()).num_days() as f64;
        // Taskwarrior uses a sigmoid-like curve clamped to [-12, 12]
        // overdue = days_until_due < 0
        if days_until_due < 0.0 {
            OVERDUE
        } else {
            // scale down for future tasks
            OVERDUE * (1.0 - (days_until_due / 14.0).min(1.0))
        }
    }

    fn due_status(due: DateTime<Utc>) -> TaskDueStatus {
        let now = Local::now();
        if due < now {
            TaskDueStatus::OverDue
        } else if due < now + Duration::hours(24) {
            TaskDueStatus::DueToday
        } else if due < now + Duration::days(7) {
            TaskDueStatus::DueSoon
        } else {
            TaskDueStatus::Due
        }
    }

    fn due(due: DateTime<Utc>) -> String {
        let delta = due.signed_duration_since(Local::now());
        let secs = delta.num_seconds().abs();

        match secs {
            s if s < 60 => format!("{}s", delta.num_seconds()),
            s if s < 3_600 => format!("{}m", delta.num_minutes()),
            s if s < 86_400 => format!("{}h", delta.num_hours()),
            s if s < 604_800 => format!("{}d", delta.num_days()),
            s if s < 2_592_000 => format!("{}w", delta.num_weeks()),
            s if s < 31_536_000 => format!("{}mo", secs / 2_592_000),
            _ => format!("{}y", secs / 31_536_000),
        }
    }
}
