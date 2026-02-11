use crate::models::SearchResult;
use crate::state::ServiceContext;

pub fn search(
    ctx: &ServiceContext,
    query: &str,
    channel_id: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<SearchResult, String> {
    ctx.db
        .search_messages(channel_id, query, limit.unwrap_or(20), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}
