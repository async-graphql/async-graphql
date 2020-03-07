use super::StarWars;

#[async_graphql::Enum(desc = "One of the films in the Star Wars Trilogy")]
#[allow(non_camel_case_types)]
pub enum Episode {
    #[item(desc = "Released in 1977.")]
    NEWHOPE,

    #[item(desc = "Released in 1980.")]
    EMPIRE,

    #[item(desc = "Released in 1983.")]
    JEDI,
}

pub struct Human<'a> {
    starwars: &'a StarWars,
    id: usize,
}

#[async_graphql::Object(desc = "A humanoid creature in the Star Wars universe.")]
impl<'a> Human<'a> {
    #[field(desc = "The id of the human.")]
    async fn id(&self) -> &str {
        self.starwars.chars[self.id].id
    }

    #[field(desc = "The name of the human.")]
    async fn name(&self) -> &str {
        self.starwars.chars[self.id].name
    }

    #[field(desc = "The friends of the human, or an empty list if they have none.")]
    async fn friends(&self) -> Vec<Character<'a>> {
        self.starwars.chars[self.id]
            .friends
            .iter()
            .map(|id| {
                Human {
                    id: *id,
                    starwars: self.starwars,
                }
                .into()
            })
            .collect()
    }

    #[field(name = "appearsIn", desc = "Which movies they appear in.")]
    async fn appears_in(&self) -> &[Episode] {
        &self.starwars.chars[self.id].appears_in
    }

    #[field(
        name = "homePlanet",
        desc = "The home planet of the human, or null if unknown."
    )]
    async fn home_planet(&self) -> &Option<&str> {
        &self.starwars.chars[self.id].home_planet
    }
}

pub struct Droid<'a> {
    starwars: &'a StarWars,
    id: usize,
}

#[async_graphql::Object(desc = "A mechanical creature in the Star Wars universe.")]
impl<'a> Droid<'a> {
    #[field(desc = "The id of the droid.")]
    async fn id(&self) -> &str {
        self.starwars.chars[self.id].id
    }

    #[field(desc = "The name of the droid.")]
    async fn name(&self) -> &str {
        self.starwars.chars[self.id].name
    }

    #[field(desc = "The friends of the droid, or an empty list if they have none.")]
    async fn friends(&self) -> Vec<Character<'a>> {
        self.starwars.chars[self.id]
            .friends
            .iter()
            .map(|id| {
                Droid {
                    id: *id,
                    starwars: self.starwars,
                }
                .into()
            })
            .collect()
    }

    #[field(name = "appearsIn", desc = "Which movies they appear in.")]
    async fn appears_in(&self) -> &[Episode] {
        &self.starwars.chars[self.id].appears_in
    }

    #[field(name = "primaryFunction", desc = "The primary function of the droid.")]
    async fn primary_function(&self) -> &Option<&str> {
        &self.starwars.chars[self.id].primary_function
    }
}

pub struct QueryRoot(pub StarWars);

#[async_graphql::Object]
impl QueryRoot {
    #[field]
    async fn hero(
        &self,
        #[arg(
            desc = "If omitted, returns the hero of the whole saga. If provided, returns the hero of that particular episode."
        )]
        episode: Episode,
    ) -> Character<'_> {
        if episode == Episode::EMPIRE {
            Human {
                id: self.0.luke,
                starwars: &self.0,
            }
            .into()
        } else {
            Droid {
                id: self.0.artoo,
                starwars: &self.0,
            }
            .into()
        }
    }

    #[field]
    async fn human(&self, #[arg(desc = "id of the human")] id: String) -> Option<Human<'_>> {
        self.0.human(&id).map(|id| Human {
            id,
            starwars: &self.0,
        })
    }

    #[field]
    async fn droid(&self, #[arg(desc = "id of the droid")] id: String) -> Option<Droid<'_>> {
        self.0.droid(&id).map(|id| Droid {
            id,
            starwars: &self.0,
        })
    }
}

#[async_graphql::Interface(
    field(name = "id", type = "&str"),
    field(name = "name", type = "&str"),
    field(name = "friends", type = "Vec<Character<'a>>"),
    field(name = "appearsIn", method = "appears_in", type = "&[Episode]")
)]
pub struct Character<'a>(Human<'a>, Droid<'a>);
