use async_graphql_derive::SimpleObject;

/// Information about pagination in a connection
#[SimpleObject(internal)]
pub struct PageInfo {
    /// When paginating backwards, are there more items?
    pub has_previous_page: bool,

    /// When paginating forwards, are there more items?
    pub has_next_page: bool,

    /// When paginating backwards, the cursor to continue.
    pub start_cursor: Option<String>,

    /// When paginating forwards, the cursor to continue.
    pub end_cursor: Option<String>,
}
