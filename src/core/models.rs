use std::str::FromStr;

use derive_more::{Constructor, Eq, PartialEq};
use serde::Serialize;
use sqlx::prelude::FromRow;
use taskchampion::{
    chrono::{DateTime, Utc},
    Status, Tag, Task,
};
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
pub struct TaskDto {
    pub id: usize,
    pub status: Status,
    pub is_blocked: bool,
    pub is_blocking: bool,
    pub description: String,
    pub priority: String,
    pub urgency: f64,
}

impl TaskDto {
    pub fn from(id: usize, value: Task) -> Self {
        let next_urg = Tag::from_str("next")
            .map(|tag| if value.has_tag(&tag) { NEXT_TAG } else { 0. })
            .unwrap_or(0.);

        let due_urg = value.get_due().map(Self::due_urgency).unwrap_or_default();
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
}
