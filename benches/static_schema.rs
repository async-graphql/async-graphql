use std::collections::HashMap;

use async_graphql::{
    Context, Enum, Error, Interface, Object, OutputType, Result,
    connection::{Connection, Edge, query},
    *,
};
use criterion::{Criterion, criterion_group, criterion_main};
use slab::Slab;

pub struct StarWarsChar {
    id: &'static str,
    name: &'static str,
    is_human: bool,
    friends: Vec<usize>,
    appears_in: Vec<Episode>,
    home_planet: Option<&'static str>,
    primary_function: Option<&'static str>,
}

pub struct StarWars {
    luke: usize,
    artoo: usize,
    chars: Slab<StarWarsChar>,
    chars_by_id: HashMap<&'static str, usize>,
}

impl StarWars {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut chars = Slab::new();

        let luke = chars.insert(StarWarsChar {
            id: "1000",
            name: "Luke Skywalker",
            is_human: true,
            friends: vec![],
            appears_in: vec![],
            home_planet: Some("Tatooine"),
            primary_function: None,
        });

        let vader = chars.insert(StarWarsChar {
            id: "1001",
            name: "Anakin Skywalker",
            is_human: true,
            friends: vec![],
            appears_in: vec![],
            home_planet: Some("Tatooine"),
            primary_function: None,
        });

        let han = chars.insert(StarWarsChar {
            id: "1002",
            name: "Han Solo",
            is_human: true,
            friends: vec![],
            appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
            home_planet: None,
            primary_function: None,
        });

        let leia = chars.insert(StarWarsChar {
            id: "1003",
            name: "Leia Organa",
            is_human: true,
            friends: vec![],
            appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
            home_planet: Some("Alderaa"),
            primary_function: None,
        });

        let tarkin = chars.insert(StarWarsChar {
            id: "1004",
            name: "Wilhuff Tarkin",
            is_human: true,
            friends: vec![],
            appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
            home_planet: None,
            primary_function: None,
        });

        let threepio = chars.insert(StarWarsChar {
            id: "2000",
            name: "C-3PO",
            is_human: false,
            friends: vec![],
            appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
            home_planet: None,
            primary_function: Some("Protocol"),
        });

        let artoo = chars.insert(StarWarsChar {
            id: "2001",
            name: "R2-D2",
            is_human: false,
            friends: vec![],
            appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
            home_planet: None,
            primary_function: Some("Astromech"),
        });

        chars[luke].friends = vec![han, leia, threepio, artoo];
        chars[vader].friends = vec![tarkin];
        chars[han].friends = vec![luke, leia, artoo];
        chars[leia].friends = vec![luke, han, threepio, artoo];
        chars[tarkin].friends = vec![vader];
        chars[threepio].friends = vec![luke, han, leia, artoo];
        chars[artoo].friends = vec![luke, han, leia];

        let chars_by_id = chars.iter().map(|(idx, ch)| (ch.id, idx)).collect();
        Self {
            luke,
            artoo,
            chars,
            chars_by_id,
        }
    }

    pub fn human(&self, id: &str) -> Option<&StarWarsChar> {
        self.chars_by_id
            .get(id)
            .copied()
            .map(|idx| self.chars.get(idx).unwrap())
            .filter(|ch| ch.is_human)
    }

    pub fn droid(&self, id: &str) -> Option<&StarWarsChar> {
        self.chars_by_id
            .get(id)
            .copied()
            .map(|idx| self.chars.get(idx).unwrap())
            .filter(|ch| !ch.is_human)
    }

    pub fn humans(&self) -> Vec<&StarWarsChar> {
        self.chars
            .iter()
            .filter(|(_, ch)| ch.is_human)
            .map(|(_, ch)| ch)
            .collect()
    }

    pub fn droids(&self) -> Vec<&StarWarsChar> {
        self.chars
            .iter()
            .filter(|(_, ch)| !ch.is_human)
            .map(|(_, ch)| ch)
            .collect()
    }

    pub fn friends(&self, ch: &StarWarsChar) -> Vec<&StarWarsChar> {
        ch.friends
            .iter()
            .copied()
            .filter_map(|id| self.chars.get(id))
            .collect()
    }
}

/// One of the films in the Star Wars Trilogy
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Episode {
    /// Released in 1977.
    NewHope,

    /// Released in 1980.
    Empire,

    /// Released in 1983.
    Jedi,
}

pub struct Human<'a>(&'a StarWarsChar);

/// A humanoid creature in the Star Wars universe.
#[Object]
impl Human<'_> {
    /// The id of the human.
    async fn id(&self) -> &str {
        self.0.id
    }

    /// The name of the human.
    async fn name(&self) -> &str {
        self.0.name
    }

    /// The friends of the human, or an empty list if they have none.
    async fn friends<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Character<'ctx>> {
        let star_wars = ctx.data_unchecked::<StarWars>();
        star_wars
            .friends(self.0)
            .into_iter()
            .map(|ch| {
                if ch.is_human {
                    Human(ch).into()
                } else {
                    Droid(ch).into()
                }
            })
            .collect()
    }

    /// Which movies they appear in.
    async fn appears_in(&self) -> &[Episode] {
        &self.0.appears_in
    }

    /// The home planet of the human, or null if unknown.
    async fn home_planet(&self) -> &Option<&str> {
        &self.0.home_planet
    }
}

pub struct Droid<'a>(&'a StarWarsChar);

/// A mechanical creature in the Star Wars universe.
#[Object]
impl Droid<'_> {
    /// The id of the droid.
    async fn id(&self) -> &str {
        self.0.id
    }

    /// The name of the droid.
    async fn name(&self) -> &str {
        self.0.name
    }

    /// The friends of the droid, or an empty list if they have none.
    async fn friends<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Character<'ctx>> {
        let star_wars = ctx.data_unchecked::<StarWars>();
        star_wars
            .friends(self.0)
            .into_iter()
            .map(|ch| {
                if ch.is_human {
                    Human(ch).into()
                } else {
                    Droid(ch).into()
                }
            })
            .collect()
    }

    /// Which movies they appear in.
    async fn appears_in(&self) -> &[Episode] {
        &self.0.appears_in
    }

    /// The primary function of the droid.
    async fn primary_function(&self) -> &Option<&str> {
        &self.0.primary_function
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hero<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(
            desc = "If omitted, returns the hero of the whole saga. If provided, returns the hero of that particular episode."
        )]
        episode: Option<Episode>,
    ) -> Character<'a> {
        let star_wars = ctx.data_unchecked::<StarWars>();
        match episode {
            Some(episode) => {
                if episode == Episode::Empire {
                    Human(star_wars.chars.get(star_wars.luke).unwrap()).into()
                } else {
                    Droid(star_wars.chars.get(star_wars.artoo).unwrap()).into()
                }
            }
            None => Human(star_wars.chars.get(star_wars.luke).unwrap()).into(),
        }
    }

    async fn human<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "id of the human")] id: String,
    ) -> Option<Human<'a>> {
        ctx.data_unchecked::<StarWars>().human(&id).map(Human)
    }

    async fn humans<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Human<'a>>> {
        let humans = ctx.data_unchecked::<StarWars>().humans().to_vec();
        query_characters(after, before, first, last, &humans, Human).await
    }

    async fn droid<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "id of the droid")] id: String,
    ) -> Option<Droid<'a>> {
        ctx.data_unchecked::<StarWars>().droid(&id).map(Droid)
    }

    async fn droids<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Droid<'a>>> {
        let droids = ctx.data_unchecked::<StarWars>().droids().to_vec();
        query_characters(after, before, first, last, &droids, Droid).await
    }
}

#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "&str"),
    field(name = "name", ty = "&str"),
    field(name = "friends", ty = "Vec<Character<'ctx>>"),
    field(name = "appears_in", ty = "&[Episode]")
)]
pub enum Character<'a> {
    Human(Human<'a>),
    Droid(Droid<'a>),
}

async fn query_characters<'a, F, T>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    characters: &[&'a StarWarsChar],
    map_to: F,
) -> Result<Connection<usize, T>>
where
    F: Fn(&'a StarWarsChar) -> T,
    T: OutputType + OutputTypeMarker,
{
    query(
        after,
        before,
        first,
        last,
        |after, before, first, last| async move {
            let mut start = 0usize;
            let mut end = characters.len();

            if let Some(after) = after {
                if after >= characters.len() {
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

            let mut slice = &characters[start..end];

            if let Some(first) = first {
                slice = &slice[..first.min(slice.len())];
                end -= first.min(slice.len());
            } else if let Some(last) = last {
                slice = &slice[slice.len() - last.min(slice.len())..];
                start = end - last.min(slice.len());
            }

            let mut connection = Connection::new(start > 0, end < characters.len());
            connection.edges.extend(
                slice
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| Edge::new(start + idx, (map_to)(item))),
            );
            Ok::<_, Error>(connection)
        },
    )
    .await
}

pub const Q: &str = r#"
{
    humans {
        nodes {
            id
            name
            friends {
                id
                name
                friends {
                    id
                    name
                }
            }
            appearsIn
            homePlanet
        }
    }
    droids {
        nodes {
            id
            name
            friends {
                id
                name
                friends {
                    id
                    name
                }
            }
            appearsIn
            primaryFunction
        }
    }
}
"#;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Static Schema", |b| {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .data(StarWars::new())
            .finish();
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                schema.execute(Q).await.into_result().unwrap();
            });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
