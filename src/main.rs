use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Select};
use minus::{page_all, Pager};
use owo_colors::OwoColorize;
use std::fmt::Write;

use serde::Deserialize;

use clap::Parser;

/// Grab info from wikipedia
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The search query to give Wikipedia
    query: String,
}

#[derive(Deserialize)]
struct SearchItemResponse {
    title: String,
    snippet: String,
}

#[derive(Deserialize)]
struct QueryResponse {
    search: Vec<SearchItemResponse>,
}

#[derive(Deserialize)]
struct SearchResponse {
    query: QueryResponse,
}

#[derive(Deserialize)]
struct SummaryPageResponse {
    extract: String,
}

#[derive(Deserialize)]
struct SummaryQueryResponse {
    pages: HashMap<String, SummaryPageResponse>,
}

#[derive(Deserialize)]
struct SummaryResponse {
    query: SummaryQueryResponse,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let body = client
        .get("https://simple.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("list", "search"),
            ("srsearch", &args.query),
        ])
        .send()
        .await.expect("Could not send search query")
        .json::<SearchResponse>()
        .await.expect("Could not parse JSON to SearchResponse");

    let queries = body.query.search;

    let pretty_printed_queries: Vec<String> = queries
        .iter()
        .map(|query| {
            let dissolved = dissolve::strip_html_tags(&query.snippet).join("");
            format!(
                "{}\n{}\n",
                if (&query.title).to_ascii_lowercase() == args.query.to_ascii_lowercase() {
                    format!("{} (exact match!)", query.title.bold())
                } else {
                    query.title.bold().to_string()
                },
                dissolved
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your definition")
        .default(0)
        .items(&pretty_printed_queries[..])
        .max_length(2)
        .interact().expect("Could not make terminal interactive to pick search result");

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
        .await.expect("Could not send HTTP Request")
        .json::<SummaryResponse>()
        .await.expect("Could not deserialize JSON");

    let summary = &body.query.pages.values().next().unwrap().extract;

    let mut pager = Pager::new();

    writeln!(pager, "{}", dissolve::strip_html_tags(summary).join("")).expect("Could not write wikipedia page to pager.");

    page_all(pager).expect("Could not create pager");
}
