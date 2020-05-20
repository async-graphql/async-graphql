use crate::types::connection::{EmptyEdgeFields, QueryOperation};
use crate::{Connection, Context, DataSource, FieldResult};
use byteorder::{ReadBytesExt, BE};

#[async_trait::async_trait]
impl<'a, T: Sync> DataSource for &'a [T] {
    type Element = &'a T;
    type EdgeFieldsObj = EmptyEdgeFields;

    async fn query_operation(
        &mut self,
        _ctx: &Context<'_>,
        operation: &QueryOperation,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let (start, end) = match operation {
            QueryOperation::None => {
                let start = 0;
                let end = self.len();
                (start, end)
            }
            QueryOperation::After { after } => {
                let start = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end = self.len();
                (start, end)
            }
            QueryOperation::Before { before } => {
                let end = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                let start = 0;
                (start, end)
            }
            QueryOperation::Between { after, before } => {
                let start = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                (start, end)
            }
            QueryOperation::First { limit } => {
                let start = 0;
                let end = (start + *limit).min(self.len());
                (start, end)
            }
            QueryOperation::FirstAfter { after, limit } => {
                let start = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end = (start + *limit).min(self.len());
                (start, end)
            }
            QueryOperation::FirstBefore { before, limit } => {
                let end_cursor = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                let start = (end_cursor - *limit).max(0);
                let end = (start + *limit).min(end_cursor);
                (start, end)
            }
            QueryOperation::FirstBetween {
                after,
                before,
                limit,
            } => {
                let start = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end_cursor = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                let end = (start + *limit).min(end_cursor);
                (start, end)
            }
            QueryOperation::Last { limit } => {
                let end = self.len();
                let start = (end - *limit).max(0);
                (start, end)
            }
            QueryOperation::LastAfter { after, limit } => {
                let end = self.len();
                let start_cursor = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let start = (end - *limit).max(start_cursor);
                (start, end)
            }
            QueryOperation::LastBefore { before, limit } => {
                let end = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                let start = (end - *limit).max(0);
                (start, end)
            }
            QueryOperation::LastBetween {
                after,
                before,
                limit,
            } => {
                let start_cursor = base64::decode(after.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end = base64::decode(before.to_string())
                    .ok()
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or_else(|| self.len());
                let start = (end - *limit).max(start_cursor);
                (start, end)
            }
            QueryOperation::Invalid => {
                let start = 0;
                let end = 0;
                (start, end)
            }
        };

        let mut nodes = Vec::with_capacity(end - start);
        if nodes.capacity() != 0 {
            for (idx, item) in self[start..end].iter().enumerate() {
                nodes.push((
                    base64::encode((idx as u32).to_be_bytes()).into(),
                    EmptyEdgeFields,
                    item,
                ));
            }
        }

        Ok(Connection::new(None, start > 0, end < self.len(), nodes))
    }
}
