use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Input, Select};
use minus::{page_all, Pager};
use owo_colors::OwoColorize;
use std::fmt::Write;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct SearchItemResponse {
    title: String,
    snippet: String,
}

#[derive(Deserialize, Debug)]
struct QueryResponse {
    search: Vec<SearchItemResponse>,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    query: QueryResponse,
}

#[derive(Deserialize, Debug)]
struct SummaryPageResponse {
    extract: String,
}

#[derive(Deserialize, Debug)]
struct SummaryQueryResponse {
    pages: HashMap<String, SummaryPageResponse>,
}

#[derive(Deserialize, Debug)]
struct SummaryResponse {
    query: SummaryQueryResponse,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter search term")
        .interact_text()?;

    let body = client
        .get("https://simple.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("list", "search"),
            ("srsearch", &input),
        ])
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    let queries = body.query.search;

    let pretty_printed_queries: Vec<String> = queries
        .iter()
        .map(|query| {
            let dissolved = dissolve::strip_html_tags(&query.snippet).join("");
            format!("{}\n{}\n", query.title.bold(), dissolved)
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your flavor")
        .default(0)
        .items(&pretty_printed_queries[..])
        .max_length(2)
        .interact()?;

    let chosen_query = &queries[selection];

    let body = client
        .get("https://simple.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("prop", "extracts"),
            ("explaintext", ""),
            // ("exintro", ""),
            ("redirects", "1"),
            ("titles", &chosen_query.title),
        ])
        .send()
        .await?
        .json::<SummaryResponse>()
        .await?;

    let summary = &body.query.pages.values().next().unwrap().extract;

    let mut pager = Pager::new();

    writeln!(pager, "{}", dissolve::strip_html_tags(summary).join(""))?;

    page_all(pager)?;

    Ok(())
}
