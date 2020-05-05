#![allow(missing_docs)]

use crate::types::connection::cursor::Cursor;
use async_graphql_derive::SimpleObject;

/// Information about pagination in a connection
#[SimpleObject(internal, desc = "Information about pagination in a connection.")]
pub struct PageInfo {
    #[field(desc = "When paginating backwards, are there more items?")]
    pub has_previous_page: bool,

    #[field(desc = "When paginating forwards, are there more items?")]
    pub has_next_page: bool,

    #[field(desc = "When paginating backwards, the cursor to continue.")]
    pub start_cursor: Option<Cursor>,

    #[field(desc = "When paginating forwards, the cursor to continue.")]
    pub end_cursor: Option<Cursor>,
}
