use crate::types::connection::{EmptyEdgeFields, QueryOperation};
use crate::{Connection, DataSource, Result};
use byteorder::{ReadBytesExt, BE};

#[async_trait::async_trait]
impl<'a, T: Sync> DataSource for &'a [T] {
    type Element = &'a T;
    type EdgeFieldsObj = EmptyEdgeFields;

    async fn query_operation(
        &self,
        operation: &QueryOperation<'_>,
    ) -> Result<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let (start, end) = match operation {
            QueryOperation::Forward { after, limit } => {
                let start = after
                    .and_then(|after| base64::decode(after).ok())
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| (idx + 1) as usize)
                    .unwrap_or(0);
                let end = (start + *limit).min(self.len());
                (start, end)
            }
            QueryOperation::Backward { before, limit } => {
                let end = before
                    .and_then(|before| base64::decode(before).ok())
                    .and_then(|data| data.as_slice().read_u32::<BE>().ok())
                    .map(|idx| idx as usize)
                    .unwrap_or(self.len());
                let start = if end < *limit { 0 } else { end - *limit };
                (start, end)
            }
        };

        let mut nodes = Vec::with_capacity(end - start);
        for (idx, item) in self[start..end].iter().enumerate() {
            nodes.push((
                base64::encode((idx as u32).to_be_bytes()),
                EmptyEdgeFields,
                item,
            ));
        }

        Ok(Connection::new(None, start > 0, end < self.len(), nodes))
    }
}
