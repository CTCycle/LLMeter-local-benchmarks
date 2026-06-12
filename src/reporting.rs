use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::Value;

use crate::results::BenchmarkRun;
use crate::utils;

#[derive(Debug, Clone)]
pub struct SummaryRow {
    pub model: String,
    pub benchmark_id: String,
    pub prompt_name: Option<String>,
    pub records: usize,
    pub errors: usize,
    pub avg_wall_time_ms: Option<f64>,
    pub avg_time_to_first_token_ms: Option<f64>,
    pub avg_tokens_per_second: Option<f64>,
    pub avg_input_tokens: Option<f64>,
    pub avg_output_tokens: Option<f64>,
    pub avg_embedding_dimensions: Option<f64>,
    pub mean_similarity: Option<f64>,
    pub avg_schema_valid: Option<f64>,
    pub avg_tool_call_valid: Option<f64>,
}

pub fn build_summary_rows(run: &BenchmarkRun) -> Vec<SummaryRow> {
    let mut grouped: HashMap<
        (String, String, Option<String>),
        Vec<&crate::benchmarks::base::BenchmarkResultRecord>,
    > = HashMap::new();

    for record in &run.results {
        let key = (
            record.model.clone(),
            record.benchmark_id.clone(),
            record.prompt_name.clone(),
        );
        grouped.entry(key).or_default().push(record);
    }

    let mut keys: Vec<_> = grouped.keys().cloned().collect();
    keys.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    let mut rows = Vec::new();
    for (model, benchmark_id, prompt_name) in keys {
        let records = grouped[&(model.clone(), benchmark_id.clone(), prompt_name.clone())].clone();
        let error_count = records.iter().filter(|r| r.error.is_some()).count();

        rows.push(SummaryRow {
            model,
            benchmark_id,
            prompt_name,
            records: records.len(),
            errors: error_count,
            avg_wall_time_ms: metric_mean(&records, "wall_time_ms"),
            avg_time_to_first_token_ms: metric_mean(&records, "time_to_first_token_ms"),
            avg_tokens_per_second: metric_mean(&records, "tokens_per_second")
                .or_else(|| metric_mean(&records, "mean_tokens_per_second")),
            avg_input_tokens: metric_mean(&records, "input_tokens"),
            avg_output_tokens: metric_mean(&records, "output_tokens"),
            avg_embedding_dimensions: metric_mean(&records, "embedding_dimensions"),
            mean_similarity: metric_mean(&records, "mean_pairwise_similarity"),
            avg_schema_valid: metric_success_ratio(&records, "schema_valid"),
            avg_tool_call_valid: metric_success_ratio(&records, "tool_call_valid"),
        });
    }
    rows
}

fn metric_mean(
    records: &[&crate::benchmarks::base::BenchmarkResultRecord],
    key: &str,
) -> Option<f64> {
    let values: Vec<f64> = records
        .iter()
        .filter_map(|r| r.metrics.get(key))
        .filter_map(|v| v.as_f64())
        .collect();
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    Some((mean * 10000.0).round() / 10000.0)
}

fn metric_success_ratio(
    records: &[&crate::benchmarks::base::BenchmarkResultRecord],
    key: &str,
) -> Option<f64> {
    let values: Vec<bool> = records
        .iter()
        .filter_map(|r| r.metrics.get(key))
        .filter_map(|v| v.as_bool())
        .collect();
    if values.is_empty() {
        return None;
    }
    let success = values.iter().filter(|&&v| v).count() as f64;
    Some((success / values.len() as f64 * 1000.0).round() / 1000.0)
}

pub fn best_throughput_row(rows: &[SummaryRow]) -> Option<&SummaryRow> {
    rows.iter()
        .filter(|r| r.avg_tokens_per_second.is_some() && r.errors == 0)
        .max_by(|a, b| {
            a.avg_tokens_per_second
                .partial_cmp(&b.avg_tokens_per_second)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn fastest_latency_row(rows: &[SummaryRow]) -> Option<&SummaryRow> {
    rows.iter()
        .filter(|r| r.avg_wall_time_ms.is_some() && r.errors == 0)
        .min_by(|a, b| {
            a.avg_wall_time_ms
                .partial_cmp(&b.avg_wall_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn render_markdown_report(run: &BenchmarkRun) -> String {
    let rows = build_summary_rows(run);
    let error_records: Vec<&crate::benchmarks::base::BenchmarkResultRecord> =
        run.results.iter().filter(|r| r.error.is_some()).collect();
    let best_tps = best_throughput_row(&rows);
    let fastest = fastest_latency_row(&rows);

    let mut lines = Vec::new();
    lines.push("# LLMeter Report".to_string());
    lines.push(String::new());
    lines.push(format!("**Run ID:** `{}`", run.run_id));
    lines.push(format!("**Created:** {}", run.created_at));
    lines.push(format!(
        "**Models:** {}",
        if run.models.is_empty() {
            "none".to_string()
        } else {
            run.models.join(", ")
        }
    ));
    lines.push(format!(
        "**Benchmarks:** {}",
        if run.benchmark_ids.is_empty() {
            "none".to_string()
        } else {
            run.benchmark_ids.join(", ")
        }
    ));
    if let Some(provider) = run.config.get("provider") {
        lines.push(format!("**Provider:** `{provider}`"));
    }
    if let Some(base_url) = run.config.get("base_url") {
        lines.push(format!("**Base URL:** `{base_url}`"));
    }
    lines.push(String::new());
    lines.push("## Executive summary".to_string());
    lines.push(String::new());
    lines.push(format!("- Records: {}", run.results.len()));
    lines.push(format!(
        "- Successful records: {}",
        run.results.len() - error_records.len()
    ));
    lines.push(format!("- Error records: {}", error_records.len()));
    if let Some(best) = best_tps {
        lines.push(format!(
            "- Best average throughput: **{} tok/s** on `{}` for `{}`",
            fmt(best.avg_tokens_per_second),
            best.model,
            best.benchmark_id
        ));
    }
    if let Some(fast) = fastest {
        lines.push(format!(
            "- Fastest average wall time: **{} ms** on `{}` for `{}`",
            fmt(fast.avg_wall_time_ms),
            fast.model,
            fast.benchmark_id
        ));
    }
    lines.push(String::new());

    lines.push("## Run configuration".to_string());
    lines.push(String::new());
    lines.push("| Setting | Value |".to_string());
    lines.push("|---|---:|".to_string());
    let mut config_keys: Vec<&String> = run.config.keys().collect();
    config_keys.sort();
    for key in config_keys {
        if let Some(value) = run.config.get(key) {
            lines.push(format!("| `{key}` | `{value}` |"));
        }
    }
    lines.push(String::new());

    lines.push("## Aggregated benchmark results".to_string());
    lines.push(String::new());
    lines.push("| Model | Benchmark | Prompt | Records | Errors | Avg wall ms | Avg TTFT ms | Avg tok/s | Input tok | Output tok | Embed dims | Similarity | Schema ok | Tool ok |".to_string());
    lines.push("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|".to_string());
    for row in &rows {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            md(&row.model),
            md(&row.benchmark_id),
            md(row.prompt_name.as_deref().unwrap_or("all")),
            row.records,
            row.errors,
            fmt(row.avg_wall_time_ms),
            fmt(row.avg_time_to_first_token_ms),
            fmt(row.avg_tokens_per_second),
            fmt(row.avg_input_tokens),
            fmt(row.avg_output_tokens),
            fmt(row.avg_embedding_dimensions),
            fmt_digits(row.mean_similarity, 4),
            fmt_ratio(row.avg_schema_valid),
            fmt_ratio(row.avg_tool_call_valid),
        ));
    }
    lines.push(String::new());

    lines.push("## Detailed records".to_string());
    lines.push(String::new());
    lines.push(
        "| Model | Benchmark | Prompt | Run | Wall ms | TTFT ms | Tok/s | Status | Preview |"
            .to_string(),
    );
    lines.push("|---|---|---|---:|---:|---:|---:|---|---|".to_string());
    for record in &run.results {
        let metrics = &record.metrics;
        let wall = metric_value_as_f64(metrics.get("wall_time_ms"));
        let ttft = metric_value_as_f64(metrics.get("time_to_first_token_ms"));
        let tps = metric_value_as_f64(metrics.get("tokens_per_second"))
            .or_else(|| metric_value_as_f64(metrics.get("mean_tokens_per_second")));
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            md(&record.model),
            md(&record.benchmark_id),
            md(record.prompt_name.as_deref().unwrap_or("")),
            record.run_index.map(|i| i.to_string()).unwrap_or_default(),
            fmt(wall),
            fmt(ttft),
            fmt(tps),
            if record.error.is_some() {
                "ERROR"
            } else {
                "OK"
            },
            md(record.response_preview.as_deref().unwrap_or("")),
        ));
    }
    lines.push(String::new());

    if !error_records.is_empty() {
        lines.push("## Errors".to_string());
        lines.push(String::new());
        lines.push("| Model | Benchmark | Prompt | Error |".to_string());
        lines.push("|---|---|---|---|".to_string());
        for record in &error_records {
            lines.push(format!(
                "| {} | {} | {} | {} |",
                md(&record.model),
                md(&record.benchmark_id),
                md(record.prompt_name.as_deref().unwrap_or("")),
                md(record.error.as_deref().unwrap_or("")),
            ));
        }
        lines.push(String::new());
    }

    lines.push("## Interpretation notes".to_string());
    lines.push(String::new());
    lines.push("- Wall time is measured by the CLI around the request.".to_string());
    lines.push("- Time to first token is measured only for streaming benchmark calls.".to_string());
    lines.push("- Token counts and endpoint-specific fields are reported only when the provider returns them.".to_string());
    lines.push("- Structured output, tool calling, responses, and embeddings may be unsupported by some local servers or models.".to_string());
    lines.push("- Results are local-machine specific. Compare runs from the same host for useful conclusions.".to_string());
    lines.push(String::new());

    lines.join("\n")
}

fn metric_value_as_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|v| v.as_f64())
}

pub fn render_html_report(run: &BenchmarkRun) -> String {
    let rows = build_summary_rows(run);
    let error_records: Vec<&crate::benchmarks::base::BenchmarkResultRecord> =
        run.results.iter().filter(|r| r.error.is_some()).collect();
    let best_tps = best_throughput_row(&rows);
    let fastest = fastest_latency_row(&rows);

    let mut summary_items = Vec::new();
    summary_items.push(("Records", run.results.len().to_string()));
    summary_items.push((
        "Successful records",
        (run.results.len() - error_records.len()).to_string(),
    ));
    summary_items.push(("Error records", error_records.len().to_string()));
    if let Some(best) = best_tps {
        summary_items.push((
            "Best average throughput",
            format!(
                "{} tok/s on {} for {}",
                fmt(best.avg_tokens_per_second),
                best.model,
                best.benchmark_id
            ),
        ));
    }
    if let Some(fast) = fastest {
        summary_items.push((
            "Fastest average wall time",
            format!(
                "{} ms on {} for {}",
                fmt(fast.avg_wall_time_ms),
                fast.model,
                fast.benchmark_id
            ),
        ));
    }

    let mut config_keys: Vec<&String> = run.config.keys().collect();
    config_keys.sort();
    let config_rows: String = config_keys
        .iter()
        .filter_map(|key| run.config.get(*key).map(|v| (key, v)))
        .map(|(key, value)| {
            format!(
                "<tr><th><code>{}</code></th><td><code>{}</code></td></tr>",
                escape(key),
                escape(&value.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let summary_cards: String = summary_items
        .iter()
        .map(|(label, value)| {
            format!(
                "<div class=\"card\"><span>{}</span><strong>{}</strong></div>",
                escape(label),
                escape(value)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let aggregate_rows: String = rows
        .iter()
        .map(|row| {
            format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(&row.model),
                escape(&row.benchmark_id),
                escape(row.prompt_name.as_deref().unwrap_or("all")),
                row.records,
                row.errors,
                fmt(row.avg_wall_time_ms),
                fmt(row.avg_time_to_first_token_ms),
                fmt(row.avg_tokens_per_second),
                fmt(row.avg_input_tokens),
                fmt(row.avg_output_tokens),
                fmt(row.avg_embedding_dimensions),
                fmt_digits(row.mean_similarity, 4),
                fmt_ratio(row.avg_schema_valid),
                fmt_ratio(row.avg_tool_call_valid),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let detail_rows: String = run
        .results
        .iter()
        .map(|record| {
            let metrics = &record.metrics;
            let wall = metric_value_as_f64(metrics.get("wall_time_ms"));
            let ttft = metric_value_as_f64(metrics.get("time_to_first_token_ms"));
            let tps = metric_value_as_f64(metrics.get("tokens_per_second"))
                .or_else(|| metric_value_as_f64(metrics.get("mean_tokens_per_second")));
            let status = if record.error.is_some() {
                "ERROR"
            } else {
                "OK"
            };
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(&record.model),
                escape(&record.benchmark_id),
                escape(record.prompt_name.as_deref().unwrap_or("")),
                record.run_index.map(|i| i.to_string()).unwrap_or_default(),
                fmt(wall),
                fmt(ttft),
                fmt(tps),
                status,
                escape(record.response_preview.as_deref().unwrap_or("")),
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let error_section = if error_records.is_empty() {
        String::new()
    } else {
        let error_rows: String = error_records
            .iter()
            .map(|record| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    escape(&record.model),
                    escape(&record.benchmark_id),
                    escape(record.prompt_name.as_deref().unwrap_or("")),
                    escape(record.error.as_deref().unwrap_or("")),
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            r#"<section>
  <h2>Errors</h2>
  <table>
    <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Error</th></tr></thead>
    <tbody>{error_rows}</tbody>
  </table>
</section>
"#
        )
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>LLMeter Report {run_id}</title>
  <style>
    :root {{ color-scheme: light dark; --border: color-mix(in srgb, CanvasText 20%, Canvas); --soft: color-mix(in srgb, CanvasText 8%, Canvas); }}
    body {{ font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; line-height: 1.5; background: Canvas; color: CanvasText; }}
    main {{ max-width: 1180px; margin: 0 auto; padding: 2rem; }}
    header {{ padding: 1.5rem; border: 1px solid var(--border); border-radius: 18px; background: var(--soft); }}
    h1 {{ margin: 0 0 .5rem; }}
    h2 {{ margin-top: 2rem; }}
    code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace; }}
    .meta {{ display: grid; grid-template-columns: max-content 1fr; gap: .25rem 1rem; }}
    .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: .75rem; margin: 1rem 0; }}
    .card {{ border: 1px solid var(--border); border-radius: 14px; padding: .9rem; background: var(--soft); }}
    .card span {{ display: block; opacity: .75; font-size: .9rem; }}
    .card strong {{ display: block; margin-top: .25rem; }}
    .table-wrap {{ overflow-x: auto; border: 1px solid var(--border); border-radius: 14px; }}
    table {{ width: 100%; border-collapse: collapse; font-size: .92rem; }}
    th, td {{ padding: .65rem .75rem; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }}
    th {{ background: var(--soft); position: sticky; top: 0; }}
    tr:last-child td {{ border-bottom: none; }}
    .notes li {{ margin-bottom: .35rem; }}
  </style>
</head>
<body>
<main>
<header>
  <h1>LLMeter Report</h1>
  <div class="meta">
    <strong>Run ID</strong><code>{run_id}</code>
    <strong>Created</strong><span>{created}</span>
    <strong>Models</strong><span>{models}</span>
    <strong>Benchmarks</strong><span>{benchmarks}</span>
  </div>
</header>
<section>
  <h2>Executive summary</h2>
  <div class="cards">{summary_cards}</div>
</section>
<section>
  <h2>Run configuration</h2>
  <div class="table-wrap"><table><tbody>{config_rows}</tbody></table></div>
</section>
<section>
  <h2>Aggregated benchmark results</h2>
  <div class="table-wrap">
    <table>
      <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Records</th><th>Errors</th><th>Avg wall ms</th><th>Avg TTFT ms</th><th>Avg tok/s</th><th>Input tok</th><th>Output tok</th><th>Embed dims</th><th>Similarity</th><th>Schema ok</th><th>Tool ok</th></tr></thead>
      <tbody>{aggregate_rows}</tbody>
    </table>
  </div>
</section>
<section>
  <h2>Detailed records</h2>
  <div class="table-wrap">
    <table>
      <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Run</th><th>Wall ms</th><th>TTFT ms</th><th>Tok/s</th><th>Status</th><th>Preview</th></tr></thead>
      <tbody>{detail_rows}</tbody>
    </table>
  </div>
</section>
{error_section}
<section class="notes">
  <h2>Interpretation notes</h2>
  <ul>
    <li>Wall time is measured by the CLI around the request.</li>
    <li>Time to first token is measured only for streaming benchmark calls.</li>
    <li>Token counts and endpoint-specific fields are reported only when the provider returns them.</li>
    <li>Structured output, tool calling, responses, and embeddings may be unsupported by some local servers or models.</li>
    <li>Results are local-machine specific. Compare runs from the same host for useful conclusions.</li>
  </ul>
</section>
</main>
</body>
</html>
"#,
        run_id = escape(&run.run_id),
        created = escape(&run.created_at),
        models = escape(&if run.models.is_empty() {
            "none".to_string()
        } else {
            run.models.join(", ")
        }),
        benchmarks = escape(&if run.benchmark_ids.is_empty() {
            "none".to_string()
        } else {
            run.benchmark_ids.join(", ")
        }),
        summary_cards = summary_cards,
        config_rows = config_rows,
        aggregate_rows = aggregate_rows,
        detail_rows = detail_rows,
        error_section = error_section,
    )
}

pub fn save_markdown_report(run: &BenchmarkRun, output_dir: &Path) -> anyhow::Result<PathBuf> {
    let path = output_dir.join(format!("{}.report.md", run.run_id));
    utils::ensure_dir(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;
    let content = render_markdown_report(run);
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write markdown report: {}", path.display()))?;
    Ok(path)
}

pub fn save_html_report(run: &BenchmarkRun, output_dir: &Path) -> anyhow::Result<PathBuf> {
    let path = output_dir.join(format!("{}.report.html", run.run_id));
    utils::ensure_dir(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;
    let content = render_html_report(run);
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write HTML report: {}", path.display()))?;
    Ok(path)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn fmt(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{:.2}", v),
        None => String::new(),
    }
}

fn fmt_digits(value: Option<f64>, digits: usize) -> String {
    match value {
        Some(v) => format!("{:.digits$}", v),
        None => String::new(),
    }
}

fn fmt_ratio(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{:.0}%", v * 100.0),
        None => String::new(),
    }
}

fn md(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace('\n', " ")
        .trim()
        .to_string()
}
