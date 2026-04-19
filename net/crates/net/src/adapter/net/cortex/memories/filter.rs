//! Plain-data `MemoriesFilter` struct for the Prisma-ish NetDB surface.
//!
//! Mirrors the filter shape used by the Node and Python SDK bindings.
//! For fluent chaining, use the existing
//! [`super::query::MemoriesQuery`] builder instead.

use super::query::{MemoriesQuery, OrderBy};

/// Filter for `find_many` / `count_where` / `exists_where` over
/// [`super::state::MemoriesState`]. All fields are optional.
///
/// Tag predicates:
/// - `tag` — must include this exact tag.
/// - `any_tag` — must include at least one tag from the list (OR).
/// - `all_tags` — must include every tag in the list (AND).
#[derive(Debug, Clone, Default)]
pub struct MemoriesFilter {
    /// Restrict to memories from this source.
    pub source: Option<String>,
    /// Case-insensitive substring match on `content`.
    pub content_contains: Option<String>,
    /// Must include this exact tag.
    pub tag: Option<String>,
    /// Must include at least one tag from this list.
    pub any_tag: Option<Vec<String>>,
    /// Must include every tag in this list.
    pub all_tags: Option<Vec<String>>,
    /// Restrict to pinned (`Some(true)`) or unpinned (`Some(false)`).
    pub pinned: Option<bool>,
    /// `created_ns > ns`.
    pub created_after_ns: Option<u64>,
    /// `created_ns < ns`.
    pub created_before_ns: Option<u64>,
    /// `updated_ns > ns`.
    pub updated_after_ns: Option<u64>,
    /// `updated_ns < ns`.
    pub updated_before_ns: Option<u64>,
    /// Result ordering.
    pub order_by: Option<OrderBy>,
    /// Truncate results after ordering.
    pub limit: Option<usize>,
}

impl MemoriesFilter {
    pub(super) fn apply<'a>(&self, mut q: MemoriesQuery<'a>) -> MemoriesQuery<'a> {
        if let Some(s) = &self.source {
            q = q.where_source(s.clone());
        }
        if let Some(s) = &self.content_contains {
            q = q.content_contains(s.clone());
        }
        if let Some(t) = &self.tag {
            q = q.where_tag(t.clone());
        }
        if let Some(tags) = &self.any_tag {
            q = q.where_any_tag(tags.clone());
        }
        if let Some(tags) = &self.all_tags {
            q = q.where_all_tags(tags.clone());
        }
        if let Some(p) = self.pinned {
            q = q.where_pinned(p);
        }
        if let Some(ns) = self.created_after_ns {
            q = q.created_after(ns);
        }
        if let Some(ns) = self.created_before_ns {
            q = q.created_before(ns);
        }
        if let Some(ns) = self.updated_after_ns {
            q = q.updated_after(ns);
        }
        if let Some(ns) = self.updated_before_ns {
            q = q.updated_before(ns);
        }
        if let Some(o) = self.order_by {
            q = q.order_by(o);
        }
        if let Some(l) = self.limit {
            q = q.limit(l);
        }
        q
    }
}
