use std::{fmt::format, vec};

use crate::errors::api_errors::TasksError;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TaskData {
    pub content: String,
    pub is_deleted: bool,
    pub project_id: String,
    pub labels: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct TasksRequestData {
    pub results: Vec<TaskData>,
    pub next_cursor: Option<String>,
}

pub async fn fetch_all_tasks(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<(), TasksError> {
    let mut cursor: Option<String> = None;
    let mut response_data: Vec<TasksRequestData> = Vec::new();

    loop {
        let mut request = client
            .get(base_url)
            .header("Authorization", format!("Bearer {api_key}"));

        if let Some(ref c) = cursor {
            request = request.query(&[("cursor", c)]);
        }

        let response = request
            .send()
            .await?
            .error_for_status()?
            .json::<TasksRequestData>()
            .await?;

        cursor = response.next_cursor.clone();
        response_data.push(response);

        if cursor.is_none() {
            break;
        }
    }

    print!("{:?}", response_data);

    Ok(())
}

pub async fn fetch_tasks_due_today(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<TasksRequestData>, TasksError> {
    let mut cursor: Option<String> = None;
    let mut response_data: Vec<TasksRequestData> = Vec::new();

    loop {
        let mut request = client
            .get(format!("{base_url}/filter"))
            .header("Authorization", format!("Bearer {api_key}"));

        let mut query = vec![("query", "today")];

        if let Some(ref c) = cursor {
            request = request.query(&[("cursor", c)]);
            query.push(("cursor", c));
        }

        let request = request.query(&query);

        let response = request
            .send()
            .await?
            .error_for_status()?
            .json::<TasksRequestData>()
            .await?;

        cursor = response.next_cursor.clone();
        response_data.push(response);

        if cursor.is_none() {
            break;
        }
    }

    print!("{:?}", response_data);

    Ok(response_data)
}
