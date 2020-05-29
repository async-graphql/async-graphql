use crate::connection::{Connection, DataSource, Edge, EmptyFields};
use crate::{Context, FieldResult};
use async_graphql_derive::DataSource;

#[DataSource(internal)]
impl<'a, T: Send + Sync> DataSource for &'a [T] {
    type CursorType = usize;
    type NodeType = &'a T;
    type ConnectionFieldsType = EmptyFields;
    type EdgeFieldsType = EmptyFields;

    #[allow(clippy::suspicious_else_formatting)]
    async fn execute_query(
        &self,
        _ctx: &Context<'_>,
        after: Option<usize>,
        before: Option<usize>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> FieldResult<
        Connection<
            Self::CursorType,
            Self::NodeType,
            Self::ConnectionFieldsType,
            Self::EdgeFieldsType,
        >,
    > {
        let mut start = 0usize;
        let mut end = self.len();

        if let Some(after) = after {
            if after >= self.len() {
                return Ok(Connection::new(false, false));
            }
            start = after + 1;
        }

        if let Some(before) = before {
            if before == 0 {
                return Ok(Connection::new(false, false));
            }
            end = before;
        }

        let mut slice = &self[start..end];

        if let Some(first) = first {
            slice = &slice[..first.min(slice.len())];
            end -= first.min(slice.len());
        } else if let Some(last) = last {
            slice = &slice[slice.len() - last.min(slice.len())..];
            start = end - last.min(slice.len());
        }

        let mut connection = Connection::new(start > 0, end < self.len());
        connection.append(
            slice
                .iter()
                .enumerate()
                .map(|(idx, item)| Edge::new(start + idx, item)),
        );
        Ok(connection)
    }
}
