use crate::types::connection::{EmptyEdgeFields, QueryOperation};
use crate::{Connection, Context, Cursor, DataSource, FieldResult};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::pin::Pin;

pub struct StreamDataSource<'a, T: Send + Sync + 'a> {
    stream: Pin<Box<dyn Stream<Item = (Cursor, EmptyEdgeFields, T)> + Send + Sync + 'a>>,
}

impl<'a, T: Send + Sync> StreamDataSource<'a, T> {
    pub fn new<S, F, Fut>(stream: S, mut f: F) -> Self
    where
        S: Stream<Item = T> + Send + Sync + 'a,
        F: FnMut(&T) -> Fut + Send + Sync + 'a,
        Fut: Future<Output = Cursor> + Send + Sync + 'a,
    {
        let stream = stream.then(move |element: T| {
            f(&element).map(move |cursor| (cursor, EmptyEdgeFields, element))
        });

        StreamDataSource {
            stream: Box::pin(stream),
        }
    }
}

use std::collections::VecDeque;
struct State<E: Send + Sync> {
    has_seen_before: bool,
    has_prev_page: bool,
    has_next_page: bool,
    edges: VecDeque<(Cursor, EmptyEdgeFields, E)>,
}

#[async_trait::async_trait]
impl<'a, T: Send + Sync + 'a> DataSource for StreamDataSource<'a, T> {
    type Element = T;
    type EdgeFieldsObj = EmptyEdgeFields;

    async fn query_operation(
        &mut self,
        _ctx: &Context<'_>,
        operation: &QueryOperation,
    ) -> FieldResult<Connection<Self::Element, Self::EdgeFieldsObj>> {
        let stream = std::mem::replace(&mut self.stream, Box::pin(futures::stream::empty()));

        let state = State::<Self::Element> {
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
