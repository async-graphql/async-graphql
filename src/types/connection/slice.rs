use crate::connection::{Connection, DataSource, EmptyEdgeFields, Record};
use crate::{Context, FieldResult, OutputValueType};

#[async_trait::async_trait]
impl<'a, T> DataSource for &'a [T]
where
    T: OutputValueType + Send + Sync + 'a,
{
    type CursorType = usize;
    type ElementType = &'a T;
    type EdgeFieldsType = EmptyEdgeFields;

    async fn execute_query(
        &self,
        _ctx: &Context<'_>,
        after: Option<usize>,
        before: Option<usize>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> FieldResult<Connection<Self::ElementType, Self::EdgeFieldsType>> {
        let mut start = 0usize;
        let mut end = self.len();

        if let Some(after) = after {
            if after >= self.len() {
                return Ok(Connection::empty());
            }
            start = after + 1;
        }
        if let Some(before) = before {
            if before == 0 {
                return Ok(Connection::empty());
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

        let connection = Connection::new_from_iter(
            slice
                .iter()
                .enumerate()
                .map(|(idx, item)| Record::new_without_edge_fields(start + idx, item)),
            start > 0,
            end < self.len(),
            Some(self.len()),
        );
        Ok(connection)
    }
}
