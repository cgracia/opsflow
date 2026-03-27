use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::PraxisConfig;

const TODOIST_BASE: &str = "https://api.todoist.com/rest/v2";

/// A Todoist task as returned by the REST API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoistTask {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub description: String,
    pub project_id: String,
    /// Priority 1 (normal) to 4 (urgent) — Todoist's scale.
    pub priority: u8,
    pub due: Option<TodoistDue>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub created_at: String,
    pub url: String,
}

impl TodoistTask {
    /// True if this task is overdue (due date is in the past).
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = &self.due {
            if let Ok(today) = chrono::NaiveDate::parse_from_str(
                &chrono::Utc::now().format("%Y-%m-%d").to_string(),
                "%Y-%m-%d",
            ) {
                if let Ok(due_date) =
                    chrono::NaiveDate::parse_from_str(&due.date, "%Y-%m-%d")
                {
                    return due_date < today;
                }
            }
        }
        false
    }

    /// True if this task is due today.
    pub fn is_due_today(&self) -> bool {
        if let Some(due) = &self.due {
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            return due.date == today;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoistDue {
    pub date: String,
    pub string: String,
    #[serde(default)]
    pub is_recurring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoistProject {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub is_inbox_project: bool,
}

#[derive(Serialize)]
struct CreateTaskRequest<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_string: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
}

pub struct TodoistClient {
    api_key: String,
    client: Client,
}

impl TodoistClient {
    /// Create a new client. Returns an error if no API key is configured.
    pub fn new(config: &PraxisConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = config
            .todoist_api_key
            .clone()
            .or_else(|| std::env::var("PRAXIS_TODOIST_API_KEY").ok())
            .ok_or("Todoist API key not configured. Set todoist_api_key in ~/.praxis/config.toml or PRAXIS_TODOIST_API_KEY env var.")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {}", e))?;

        Ok(Self {
            api_key,
            client,
        })
    }

    /// Fetch active tasks, optionally filtered by a Todoist filter expression.
    /// Example filters: "today | overdue", "p1 & today"
    pub fn fetch_tasks(
        &self,
        filter: Option<&str>,
    ) -> Result<Vec<TodoistTask>, Box<dyn std::error::Error>> {
        let mut req = self
            .client
            .get(format!("{}/tasks", TODOIST_BASE))
            .bearer_auth(&self.api_key);

        if let Some(f) = filter {
            req = req.query(&[("filter", f)]);
        }

        let response = req.send().map_err(|e| format!("Todoist request failed: {}", e))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("Todoist API error {}: {}", status, body).into());
        }
        let tasks: Vec<TodoistTask> = response
            .json()
            .map_err(|e| format!("failed to parse Todoist tasks: {}", e))?;
        Ok(tasks)
    }

    /// Create a new task in Todoist. Returns the created task.
    pub fn create_task(
        &self,
        content: &str,
        due: Option<&str>,
        priority: Option<u8>,
        project_id: Option<&str>,
    ) -> Result<TodoistTask, Box<dyn std::error::Error>> {
        let body = CreateTaskRequest {
            content,
            due_string: due,
            priority,
            project_id,
            labels: None,
        };

        let response = self
            .client
            .post(format!("{}/tasks", TODOIST_BASE))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|e| format!("Todoist request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("Todoist API error {}: {}", status, body).into());
        }
        let task: TodoistTask = response
            .json()
            .map_err(|e| format!("failed to parse created task: {}", e))?;
        Ok(task)
    }

    /// Close (complete) a task by ID.
    pub fn close_task(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(format!("{}/tasks/{}/close", TODOIST_BASE, id))
            .bearer_auth(&self.api_key)
            .send()
            .map_err(|e| format!("Todoist request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("Todoist API error {}: {}", status, body).into());
        }
        Ok(())
    }

    /// List all projects.
    pub fn fetch_projects(
        &self,
    ) -> Result<Vec<TodoistProject>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/projects", TODOIST_BASE))
            .bearer_auth(&self.api_key)
            .send()
            .map_err(|e| format!("Todoist request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("Todoist API error {}: {}", status, body).into());
        }
        let projects: Vec<TodoistProject> = response
            .json()
            .map_err(|e| format!("failed to parse Todoist projects: {}", e))?;
        Ok(projects)
    }

    /// Resolve a project name to its ID.
    pub fn resolve_project_id(
        &self,
        name: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let projects = self.fetch_projects()?;
        let lower = name.to_lowercase();
        Ok(projects
            .into_iter()
            .find(|p| p.name.to_lowercase() == lower)
            .map(|p| p.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_todoist_tasks_fixture() {
        let json = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/todoist/tasks.json"),
        )
        .unwrap();
        let tasks: Vec<TodoistTask> = serde_json::from_str(&json).unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].id, "task-001");
        assert_eq!(tasks[0].content, "Prep 1:1 with Jonas");
        assert_eq!(tasks[0].priority, 2);
        assert!(tasks[0].due.is_some());
    }

    #[test]
    fn test_todoist_task_is_due_today() {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let task = TodoistTask {
            id: "1".to_string(),
            content: "test".to_string(),
            description: String::new(),
            project_id: "p1".to_string(),
            priority: 1,
            due: Some(TodoistDue {
                date: today,
                string: "today".to_string(),
                is_recurring: false,
            }),
            labels: vec![],
            created_at: "2026-03-25T00:00:00Z".to_string(),
            url: "https://todoist.com/task/1".to_string(),
        };
        assert!(task.is_due_today());
    }

    #[test]
    fn test_todoist_task_is_overdue() {
        let task = TodoistTask {
            id: "2".to_string(),
            content: "old task".to_string(),
            description: String::new(),
            project_id: "p1".to_string(),
            priority: 1,
            due: Some(TodoistDue {
                date: "2020-01-01".to_string(),
                string: "Jan 1 2020".to_string(),
                is_recurring: false,
            }),
            labels: vec![],
            created_at: "2020-01-01T00:00:00Z".to_string(),
            url: "https://todoist.com/task/2".to_string(),
        };
        assert!(task.is_overdue());
    }

    /// Live integration test — only runs when PRAXIS_TEST_TODOIST_API_KEY is set.
    #[test]
    fn test_live_fetch_tasks() {
        if std::env::var("PRAXIS_TEST_TODOIST_API_KEY").is_err() {
            return;
        }
        let config = crate::config::load_config();
        let client = TodoistClient::new(&config).unwrap();
        let tasks = client.fetch_tasks(Some("today | overdue")).unwrap();
        println!("Live tasks: {}", tasks.len());
    }
}
