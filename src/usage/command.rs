use std::path::PathBuf;

use crate::app::AppContext;
use crate::args::UsageOptions;
use crate::cli::CommandResult;
use crate::error::CommandError;
use crate::output;

use super::model::{
    LocalUsageSummary, RateLimitsData, UsageBreakdown, UsageCategorySummary, UsageCommandData,
    UsageEventSummary,
};
use super::pricing;

pub fn execute(ctx: &AppContext, opts: UsageOptions) -> CommandResult {
    let command = "usage";
    let session_store = ctx
        .session_store_with_override(opts.common.auth_file.as_deref(), opts.session_db.as_deref());

    let provider_hint = resolve_provider_hint(ctx, opts.common.auth_file.as_deref());
    let session = session_store
        .ensure_active_session(opts.session_id.as_deref(), &provider_hint)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let session_summary = session_store
        .build_session_summary(&session)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let event_summaries = session_store
        .usage_event_summaries(&session.session_id)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let rate_limits = session_store
        .latest_rate_limits(&session.session_id)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?
        .unwrap_or_else(|| RateLimitsData {
            available: false,
            ..RateLimitsData::default()
        });

    let local_usage = LocalUsageSummary {
        input_tokens: session.input_tokens,
        output_tokens: session.output_tokens,
        cache_read_tokens: session.cache_read_tokens,
        cache_write_tokens: session.cache_write_tokens,
        reasoning_tokens: session.reasoning_tokens,
        total_tokens: session.input_tokens
            + session.output_tokens
            + session.cache_read_tokens
            + session.cache_write_tokens
            + session.reasoning_tokens,
        estimated_cost_usd: Some(session.estimated_cost_micro_usd as f64 / 1_000_000.0),
        pricing_status: "estimated".to_string(),
        pricing_source: "bundled_xai_table".to_string(),
        last_model: session.active_model.clone(),
        context_window_tokens: session.context_window_tokens.or_else(|| {
            session
                .active_model
                .as_deref()
                .and_then(pricing::default_context_window_tokens)
        }),
        history_turns: session_summary.tracked_command_count,
        compression_count: session.compression_count,
        has_unflushed_tracker_data: false,
    };

    let data = UsageCommandData {
        provider: session.provider.clone(),
        session: session_summary,
        local_usage,
        breakdown: build_usage_breakdown(&event_summaries),
        recent_rate_limits: rate_limits,
    };

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        print_human(&data, session_store.path);
    }

    Ok(())
}

fn resolve_provider_hint(ctx: &AppContext, auth_file: Option<&std::path::Path>) -> String {
    match auth_file {
        Some(path) => ctx
            .state_store
            .load_valid_state(path)
            .map(|state| state.provider)
            .unwrap_or_else(|_| "xai-oauth".to_string()),
        None => {
            let path = ctx.state_store.resolve_path(None);
            ctx.state_store
                .load_valid_state(&path)
                .map(|state| state.provider)
                .unwrap_or_else(|_| "xai-oauth".to_string())
        }
    }
}

fn print_human(data: &UsageCommandData, session_store_path: PathBuf) {
    let context_line = format_context_line(
        data.local_usage.total_tokens,
        data.local_usage.context_window_tokens,
    );
    let cost_line = data
        .local_usage
        .estimated_cost_usd
        .map(|cost| format!("${cost:.2} (this session)"))
        .unwrap_or_else(|| "n/a".to_string());
    let duration_line = format_duration(data.session.duration_seconds);

    println!("Session Usage");
    println!(
        "├─ Input tokens:     {}",
        format_number(data.local_usage.input_tokens)
    );
    println!(
        "├─ Output tokens:    {}",
        format_number(data.local_usage.output_tokens)
    );
    println!(
        "├─ Total tokens:     {}",
        format_number(data.local_usage.total_tokens)
    );
    println!("├─ Estimated cost:   {cost_line}");
    println!("├─ Duration:         {duration_line}");
    println!("└─ Context:          {context_line}");
    println!();

    println!("Usage Breakdown");
    print_category_block("Text", &data.breakdown.text, true);
    print_category_block("Image", &data.breakdown.image, false);
    print_category_block("Video", &data.breakdown.video, false);
    print_category_block("Audio", &data.breakdown.audio, false);

    if !data.session.models.is_empty() {
        println!();
        println!("Session metadata");
        println!("├─ Session ID:       {}", data.session.session_id);
        println!("├─ Requests:         {}", data.session.request_count);
        println!(
            "├─ Tracked commands: {}",
            data.session.tracked_command_count
        );
        println!("├─ Models:           {}", data.session.models.join(", "));
        println!("└─ Session store:    {}", session_store_path.display());
    }
}

fn build_usage_breakdown(events: &[UsageEventSummary]) -> UsageBreakdown {
    let mut breakdown = UsageBreakdown::default();
    for event in events {
        let target = match category_key(&event.command) {
            "text" => &mut breakdown.text,
            "image" => &mut breakdown.image,
            "video" => &mut breakdown.video,
            "audio" => &mut breakdown.audio,
            _ => continue,
        };
        target.request_count += event.request_count;
        target.input_tokens += event.input_tokens;
        target.output_tokens += event.output_tokens;
        target.cache_read_tokens += event.cache_read_tokens;
        target.cache_write_tokens += event.cache_write_tokens;
        target.reasoning_tokens += event.reasoning_tokens;
        let current = target.estimated_cost_usd.unwrap_or(0.0);
        target.estimated_cost_usd =
            Some(current + (event.estimated_cost_micro_usd as f64 / 1_000_000.0));
        target.commands.push(event.command.clone());
    }

    for category in [
        &mut breakdown.text,
        &mut breakdown.image,
        &mut breakdown.video,
        &mut breakdown.audio,
    ] {
        category.commands.sort();
        category.commands.dedup();
        if category.request_count == 0 {
            category.estimated_cost_usd = None;
        }
    }

    breakdown
}

fn category_key(command: &str) -> &'static str {
    match command {
        "chat" | "search" | "task chat" | "task x-search" => "text",
        "image" | "task image-gen" => "image",
        "video" | "task video-gen" => "video",
        "tts" | "stt" | "task tts" | "task stt" => "audio",
        _ => "other",
    }
}

fn print_category_block(label: &str, category: &UsageCategorySummary, _first: bool) {
    println!("├─ {label}");
    println!("│   ├─ Requests:       {}", category.request_count);
    println!(
        "│   ├─ Commands:       {}",
        if category.commands.is_empty() {
            "none".to_string()
        } else {
            category.commands.join(", ")
        }
    );
    println!(
        "│   ├─ Input tokens:   {}",
        format_token_count(category.input_tokens)
    );
    println!(
        "│   ├─ Output tokens:  {}",
        format_token_count(category.output_tokens)
    );
    println!(
        "│   ├─ Total tokens:   {}",
        format_token_count(
            category.input_tokens
                + category.output_tokens
                + category.cache_read_tokens
                + category.cache_write_tokens
                + category.reasoning_tokens,
        )
    );
    println!(
        "│   ├─ Reasoning:      {}",
        format_token_count(category.reasoning_tokens)
    );
    println!(
        "│   └─ Estimated cost: {}",
        category
            .estimated_cost_usd
            .map(|cost| format!("${cost:.2}"))
            .unwrap_or_else(|| "n/a".to_string())
    );
}

fn format_number(value: u64) -> String {
    format_token_count(value)
}

fn format_token_count(value: u64) -> String {
    if value >= 1_000_000_000 {
        format_compact_metric(value, 1_000_000_000.0, "B")
    } else if value >= 1_000_000 {
        format_compact_metric(value, 1_000_000.0, "M")
    } else if value >= 1_000 {
        format_compact_metric(value, 1_000.0, "K")
    } else {
        value.to_string()
    }
}

fn format_compact_metric(value: u64, divisor: f64, suffix: &str) -> String {
    let scaled = value as f64 / divisor;
    if scaled >= 100.0 {
        format!("{scaled:.0}{suffix}")
    } else if scaled >= 10.0 {
        format!("{scaled:.1}{suffix}")
    } else {
        format!("{scaled:.2}{suffix}")
    }
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{hours}h {mins:02}m {secs:02}s")
    } else {
        format!("{mins}m {secs:02}s")
    }
}

fn format_context_line(total_tokens: u64, context_window_tokens: Option<u64>) -> String {
    match context_window_tokens {
        Some(limit) if limit > 0 => {
            let pct = (total_tokens as f64 / limit as f64) * 100.0;
            format!(
                "{} / {} tokens ({pct:.1}%)",
                format_number(total_tokens),
                compact_token_limit(limit),
            )
        }
        _ => format!("{} tokens", format_number(total_tokens)),
    }
}

fn compact_token_limit(limit: u64) -> String {
    format_token_count(limit)
}
