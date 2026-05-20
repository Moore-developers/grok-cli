pub fn estimate_text_cost_micro_usd(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
) -> Option<i64> {
    let normalized = model.trim().to_ascii_lowercase();

    let (input_per_million, output_per_million) = match normalized.as_str() {
        // xAI official docs, verified 2026-05-20:
        // grok-4.20-0309-reasoning: $1.25 / 1M input, $0.20 / 1M cached input, $2.50 / 1M output
        // grok-4.20-0309-non-reasoning: same pricing
        // grok-4.3: $1.25 / 1M input, $2.50 / 1M output
        "grok-4.20-reasoning"
        | "grok-4.20-0309-reasoning"
        | "grok-4.20-0309"
        | "grok-4.20"
        | "grok-4.20-beta"
        | "grok-4.20-beta-0309"
        | "grok-4.20-beta-0309-reasoning"
        | "grok-4.20-beta-reasoning"
        | "grok-4.3"
        | "grok-4.3-latest"
        | "grok-latest" => (1_250_000_i64, 2_500_000_i64),
        "grok-4.20-0309-non-reasoning"
        | "grok-4.20-non-reasoning"
        | "grok-4.20-beta-0309-non-reasoning"
        | "grok-4.20-beta-non-reasoning" => (1_250_000_i64, 2_500_000_i64),
        _ => return None,
    };

    let input_cost = (input_tokens as i128 * input_per_million as i128) / 1_000_000_i128;
    let output_cost = (output_tokens as i128 * output_per_million as i128) / 1_000_000_i128;
    let total = input_cost + output_cost;
    i64::try_from(total).ok()
}

pub fn default_context_window_tokens(model: &str) -> Option<u64> {
    let normalized = model.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "grok-4.20-reasoning"
        | "grok-4.20-0309-reasoning"
        | "grok-4.20-0309"
        | "grok-4.20"
        | "grok-4.20-beta"
        | "grok-4.20-beta-0309"
        | "grok-4.20-beta-0309-reasoning"
        | "grok-4.20-beta-reasoning"
        | "grok-4.20-0309-non-reasoning"
        | "grok-4.20-non-reasoning"
        | "grok-4.20-beta-0309-non-reasoning"
        | "grok-4.20-beta-non-reasoning" => Some(2_000_000),
        "grok-4.3" | "grok-4.3-latest" | "grok-latest" => Some(1_000_000),
        _ => None,
    }
}
