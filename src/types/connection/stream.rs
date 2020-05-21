use crate::types::connection::{EmptyEdgeFields, QueryOperation};
use crate::{Connection, Context, Cursor, DataSource, FieldResult, ObjectType};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct StreamDataSource<'a, T, E = EmptyEdgeFields>
where
    T: Send + Sync + 'a,
    E: ObjectType + Send + Sync + 'a,
{
    stream: Pin<Box<dyn Stream<Item = (Cursor, E, T)> + Send + Sync + 'a>>,
}

impl<'a, S, T, E> From<S> for StreamDataSource<'a, T, E>
where
    S: Stream<Item = (Cursor, E, T)> + Send + Sync + 'a,
    T: Send + Sync + 'a,
    E: ObjectType + Send + Sync + 'a,
{
    fn from(stream: S) -> Self {
        StreamDataSource {
            stream: Box::pin(stream),
        }
    }
}

use std::collections::VecDeque;
struct State<T, E = EmptyEdgeFields>
where
    T: Send + Sync,
    E: ObjectType + Send + Sync,
{
    has_seen_before: bool,
    has_prev_page: bool,
    has_next_page: bool,
    edges: VecDeque<(Cursor, E, T)>,
}

#[async_trait::async_trait]
impl<'a, T, E> DataSource for StreamDataSource<'a, T, E>
where
    T: Send + Sync + 'a,
    E: ObjectType + Send + Sync + 'a,
{
    type Element = T;
    type EdgeFieldsObj = E;

    async fn query_operation(
        &mut self,
        _ctx: &Context<'_>,
        operation: &QueryOperation,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let stream = std::mem::replace(&mut self.stream, Box::pin(futures::stream::empty()));

        let state = State::<Self::Element, Self::EdgeFieldsObj> {
            has_seen_before: false,
            has_prev_page: false,
            has_next_page: false,
            edges: VecDeque::new(),
        };

        let state = stream
            .fold(state, move |mut state, (cursor, fields, node)| {
                if state.has_seen_before {
                    return futures::future::ready(state);
                }

                match operation {
                    QueryOperation::After { after }
                    | QueryOperation::Between { after, .. }
                    | QueryOperation::FirstAfter { after, .. }
                    | QueryOperation::FirstBetween { after, .. }
                    | QueryOperation::LastAfter { after, .. }
                    | QueryOperation::LastBetween { after, .. } => {
                        if *after == cursor {
                            state.has_prev_page = true;
                            state.has_next_page = false;
                            state.edges.clear();

                            return futures::future::ready(state);
                        }
                    }
                    _ => {}
                }

                match operation {
                    QueryOperation::Before { before }
                    | QueryOperation::Between { before, .. }
                    | QueryOperation::FirstBefore { before, .. }
                    | QueryOperation::FirstBetween { before, .. }
                    | QueryOperation::LastBefore { before, .. }
                    | QueryOperation::LastBetween { before, .. } => {
                        if *before == cursor {
                            state.has_seen_before = true;
                            state.has_next_page = true;

                            return futures::future::ready(state);
                        }
                    }
                    _ => {}
                }

                match operation {
                    QueryOperation::First { limit }
                    | QueryOperation::FirstAfter { limit, .. }
                    | QueryOperation::FirstBefore { limit, .. }
                    | QueryOperation::FirstBetween { limit, .. } => {
                        if state.edges.len() < *limit {
                            state.edges.push_back((cursor, fields, node))
                        } else {
                            state.has_next_page = true;
                        }
                    }

                    QueryOperation::Last { limit }
                    | QueryOperation::LastAfter { limit, .. }
                    | QueryOperation::LastBefore { limit, .. }
                    | QueryOperation::LastBetween { limit, .. } => {
                        if state.edges.len() >= *limit {
                            state.has_prev_page = true;
                            state.edges.pop_front();
                        }
                        state.edges.push_back((cursor, fields, node));
                    }

                    _ => {
                        state.edges.push_back((cursor, fields, node));
                    }
                }

                futures::future::ready(state)
            })
            .await;

        Ok(Connection::new(
            None,
            state.has_prev_page,
            state.has_next_page,
            state.edges.into(),
        ))
    }
}
