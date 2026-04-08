use std::str::FromStr;

use itertools::Itertools;
use taskchampion::{
    chrono::{DateTime, Duration, Local, Utc},
    Annotation, Status, Tag, Task,
};
use uuid::Uuid;

// urgency coefficient
const NEXT_TAG: f64 = 15.0;
const DUE: f64 = 12.0;
const BLOCKING_OTHERS: f64 = 8.0;
const PRIORITY_HIGH: f64 = 6.0;
const PRIORITY_MEDIUM: f64 = 3.9;
const PRIORITY_LOW: f64 = -2.0;
const TAG_FI: f64 = 4.0;
const ACTIVE_STARTED: f64 = 4.0;
const TASK_AGE: f64 = 2.0;
const PROJECT_ASSIGNED: f64 = 1.0;
const TAGS_COUNT: f64 = 1.0;
const ANNOTATIONS_COUNT: f64 = 1.0;
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
    pub uuid: Uuid,
    pub status: Status,
    pub description: String,
    pub due: Option<String>,
    pub due_status: TaskDueStatus,
    pub annotations: Vec<Annotation>,

    pub is_blocked: bool,
    pub is_blocking: bool,

    pub tags: String,
    pub deps: String,

    pub priority: String,
    pub urgency: f64,
}

impl TaskDto {
    pub fn from(id: usize, task: Task, deps: Vec<usize>) -> Self {
        let next_urg = Tag::from_str("next")
            .map(|tag| if task.has_tag(&tag) { NEXT_TAG } else { 0. })
            .unwrap_or(0.);
        let fi_urg = Tag::from_str("fi")
            .map(|tag| if task.has_tag(&tag) { TAG_FI } else { 0. })
            .unwrap_or(0.);

        let due_urg = task.get_due().map(Self::due_urgency).unwrap_or_default();
        let due_status = task
            .get_due()
            .map(Self::due_status)
            .unwrap_or(TaskDueStatus::Not);
        let due = task.get_due().map(Self::due);
        let blocking_urg = if task.is_blocking() {
            BLOCKING_OTHERS
        } else {
            0.
        };
        let pri_urg = match task.get_priority() {
            "h" => PRIORITY_HIGH,
            "m" => PRIORITY_MEDIUM,
            "l" => PRIORITY_LOW,
            _ => 0.,
        };
        let act_urg = if task.is_active() { ACTIVE_STARTED } else { 0. };

        let age_urg = task
            .get_entry()
            .map(|age| (Utc::now() - age).num_days().clamp(0, 365) as f64)
            .map(|age_days| (age_days / 365.0) * TASK_AGE)
            .unwrap_or_default();

        let proj_urg = task
            .get_user_defined_attribute("project")
            .map(|_| PROJECT_ASSIGNED)
            .unwrap_or_default();
        let wait_urg = if task.is_waiting() { WAITING } else { 0. };
        let block_urg = if task.is_blocked() { BLOCKED } else { 0. };

        let user_tags: Vec<_> = task.get_tags().filter(|tag| tag.is_user()).collect();
        let tags_urg = match user_tags.len() {
            0 => 0.0,
            1 => 0.8,
            2 => 0.9,
            _ => 1.0,
        } * TAGS_COUNT;

        let annotations = task
            .get_annotations()
            .sorted_by(|x, y| y.entry.cmp(&x.entry))
            .collect::<Vec<_>>();

        let annote_urg = match annotations.len() {
            0 => 0.0,
            1 => 0.5,
            2 => 0.7,
            _ => 1.0,
        } * ANNOTATIONS_COUNT;

        Self {
            id,
            uuid: task.get_uuid(),
            status: task.get_status(),
            description: task.get_description().to_owned(),
            priority: task.get_priority().to_owned(),
            is_blocked: task.is_blocked(),
            is_blocking: task.is_blocking(),
            tags: user_tags.iter().join(" "),
            deps: deps.iter().join(" "),
            annotations,
            due,
            due_status,
            urgency: next_urg
                + fi_urg
                + due_urg
                + blocking_urg
                + pri_urg
                + act_urg
                + age_urg
                + proj_urg
                + tags_urg
                + annote_urg
                + wait_urg
                + block_urg,
        }
    }

    fn due_urgency(due: DateTime<Utc>) -> f64 {
        // days_overdue: positive = overdue, negative = future
        let days_overdue = (Utc::now() - due).num_seconds() as f64 / 86_400.0;
        let term = ((days_overdue + 14.0) * 0.8 / 21.0) + 0.2;
        term.clamp(0.2, 1.0) * DUE
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
