use crate::types::connection::cursor::Cursor;
use async_graphql_derive::Object;

pub struct PageInfo {
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub start_cursor: Option<Cursor>,
    pub end_cursor: Option<Cursor>,
}

#[Object(internal)]
impl PageInfo {
    #[field(desc = "When paginating backwards, are there more items?")]
    async fn has_previous_page(&self) -> bool {
        self.has_previous_page
    }

    #[field(desc = "When paginating forwards, are there more items?")]
    async fn has_next_page(&self) -> bool {
        self.has_next_page
    }

    #[field(desc = "When paginating backwards, the cursor to continue.")]
    async fn start_cursor(&self) -> &Option<Cursor> {
        &self.start_cursor
    }

    #[field(desc = "When paginating forwards, the cursor to continue.")]
    async fn end_cursor(&self) -> &Option<Cursor> {
        &self.end_cursor
    }
}
