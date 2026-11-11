//! `mcp-cli` - talk to the MCP server from the terminal (great for learning and testing).
//!
//! Examples (PowerShell):
//!   mcp-cli --server ..\dist\server\SemanticSearch.McpServer.exe info
//!   mcp-cli --server ...exe tools
//!   mcp-cli --server ...exe search "my parcel never arrived" --top-k 3
//!   mcp-cli --server ...exe record products 5
//!   mcp-cli --server ...exe call list_sources "{}"
//!
//! The server path can also come from the MCP_SERVER_COMMAND environment variable.

use std::time::Duration;

use mcp_client::{McpClient, ServerConfig};
use serde_json::{json, Value};

const USAGE: &str = "\
usage: mcp-cli [--server <exe>] [--server-arg <arg>]... [--quiet] <command>

commands:
  info                                  server name, version and protocol
  tools                                 list the tools the server offers
  search <query> [--top-k N] [--min-score X] [--source S] [--mode semantic|keyword|pattern]
  record <source> <id>                  fetch one full record
  call <tool> [json-arguments]          call any tool with raw JSON arguments";

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut server = std::env::var("MCP_SERVER_COMMAND").ok();
    let mut server_args: Vec<String> = Vec::new();
    let mut quiet = false;

    // Global options
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--server" => {
                server = Some(take_value(&mut args, i)?);
            }
            "--server-arg" => {
                server_args.push(take_value(&mut args, i)?);
            }
            "--quiet" => {
                quiet = true;
                args.remove(i);
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => i += 1,
        }
    }

    let command = server.ok_or("no server given: use --server <exe> or set MCP_SERVER_COMMAND")?;
    if args.is_empty() {
        println!("{USAGE}");
        return Ok(());
    }

    let mut config = ServerConfig::new(command);
    config.args = server_args;
    config.forward_stderr = !quiet;
    config.request_timeout = Duration::from_secs(60);
    let client = McpClient::new(config);

    let sub = args.remove(0);
    let output: Value = match sub.as_str() {
        "info" => client.server_info().await?,
        "tools" => serde_json::to_value(client.list_tools().await?)?,
        "search" => {
            let mut top_k = 5i64;
            let mut min_score: Option<f64> = None;
            let mut source: Option<String> = None;
            let mut mode = "semantic".to_string();
            let mut words = Vec::new();
            let mut j = 0;
            while j < args.len() {
                match args[j].as_str() {
                    "--top-k" => top_k = take_value(&mut args, j)?.parse()?,
                    "--min-score" => min_score = Some(take_value(&mut args, j)?.parse()?),
                    "--source" => source = Some(take_value(&mut args, j)?),
                    "--mode" => mode = take_value(&mut args, j)?,
                    _ => {
                        words.push(args[j].clone());
                        j += 1;
                    }
                }
            }
            if words.is_empty() {
                return Err("search needs a query".into());
            }
            let mut a = json!({ "query": words.join(" "), "topK": top_k, "mode": mode });
            if let Some(m) = min_score {
                a["minScore"] = json!(m);
            }
            if let Some(s) = source {
                a["source"] = json!(s);
            }
            let result = client.call_tool("semantic_search", a).await?;
            print_search(&result);
            client.shutdown().await;
            return Ok(());
        }
        "record" => {
            if args.len() < 2 {
                return Err("usage: record <source> <id>".into());
            }
            let id: i64 = args[1].parse()?;
            client
                .call_tool("get_record", json!({ "source": args[0], "id": id }))
                .await?
        }
        "call" => {
            let tool = args.first().ok_or("usage: call <tool> [json]")?.clone();
            let raw = args.get(1).cloned().unwrap_or_else(|| "{}".into());
            let a: Value = serde_json::from_str(&raw)?;
            client.call_tool(&tool, a).await?
        }
        other => return Err(format!("unknown command '{other}'\n\n{USAGE}").into()),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    client.shutdown().await;
    Ok(())
}

/// Removes `args[i]` (the flag) and the value after it, returning the value.
fn take_value(args: &mut Vec<String>, i: usize) -> Result<String, String> {
    if i + 1 >= args.len() {
        return Err(format!("{} needs a value", args[i]));
    }
    let v = args.remove(i + 1);
    args.remove(i);
    Ok(v)
}

fn print_search(result: &Value) {
    let empty = vec![];
    let results = result["results"].as_array().unwrap_or(&empty);
    println!(
        "\nquery: {}   mode: {}   method: {}   {} ms",
        result["query"].as_str().unwrap_or(""),
        result["mode"].as_str().unwrap_or(""),
        result["method"].as_str().unwrap_or(""),
        result["took_ms"]
    );
    if results.is_empty() {
        println!("no results above the minimum score");
    }
    if let Some(c) = result["query_concepts"]
        .as_array()
        .filter(|c| !c.is_empty())
    {
        let labels: Vec<&str> = c.iter().filter_map(|x| x.as_str()).collect();
        println!("understood as: {}", labels.join(", "));
    }
    for (n, r) in results.iter().enumerate() {
        let score = r["score"].as_f64().unwrap_or(0.0);
        let bar = "#".repeat((score.clamp(0.0, 1.0) * 20.0).round() as usize);
        let list = |key: &str| {
            r[key]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default()
        };
        println!(
            "\n{:>2}. [{}/{}] {}\n    score {:.3} {:<20} concepts: [{}]  words: [{}]\n    {}",
            n + 1,
            r["source"].as_str().unwrap_or(""),
            r["id"],
            r["title"].as_str().unwrap_or(""),
            score,
            bar,
            list("matched_concepts"),
            list("matched_words"),
            r["snippet"].as_str().unwrap_or("")
        );
    }
    println!();
}
