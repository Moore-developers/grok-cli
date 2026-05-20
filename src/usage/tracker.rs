use std::path::Path;

use crate::app::AppContext;
use crate::error::AppError;

use super::model::UsageDelta;

pub fn record_usage(
    ctx: &AppContext,
    auth_file: Option<&Path>,
    provider: &str,
    delta: UsageDelta,
) -> Result<(), AppError> {
    let session_store = ctx.session_store(auth_file);
    let session = session_store.ensure_active_session(None, provider)?;
    session_store.record_usage(&session.session_id, &delta)
}
