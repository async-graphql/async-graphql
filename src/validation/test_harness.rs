#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_code)]

use crate::validation::visitor::{visit, Visitor, VisitorContext};
use crate::*;
use async_graphql_parser::query::Document;
use once_cell::sync::Lazy;

#[InputObject(internal)]
struct TestInput {
    id: i32,
    name: String,
}

impl Default for TestInput {
    fn default() -> Self {
        Self {
            id: 423,
            name: "foo".to_string(),
        }
    }
}

#[Enum(internal)]
enum DogCommand {
    Sit,
    Heel,
    Down,
}

struct Dog;

#[Object(internal)]
impl Dog {
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    async fn nickname(&self) -> Option<String> {
        unimplemented!()
    }

    async fn bark_volume(&self) -> Option<i32> {
        unimplemented!()
    }

    async fn barks(&self) -> Option<bool> {
        unimplemented!()
    }

    async fn does_know_command(&self, dog_command: Option<DogCommand>) -> Option<bool> {
        unimplemented!()
    }

    async fn is_housetrained(&self, #[arg(default = true)] at_other_homes: bool) -> Option<bool> {
        unimplemented!()
    }

    async fn is_at_location(&self, x: Option<i32>, y: Option<i32>) -> Option<bool> {
        unimplemented!()
    }
}

#[Enum(internal)]
enum FurColor {
    Brown,
    Black,
    Tan,
    Spotted,
}

struct Cat;

#[Object(internal)]
impl Cat {
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    async fn nickname(&self) -> Option<String> {
        unimplemented!()
    }

    async fn meows(&self) -> Option<bool> {
        unimplemented!()
    }

    async fn meow_volume(&self) -> Option<i32> {
        unimplemented!()
    }

    async fn fur_color(&self) -> Option<FurColor> {
        unimplemented!()
    }
}

#[Union(internal)]
enum CatOrDog {
    Cat(Cat),
    Dog(Dog),
}

struct Human;

#[Object(internal)]
impl Human {
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    async fn pets(&self) -> Option<Vec<Option<Pet>>> {
        unimplemented!()
    }

    async fn relatives(&self) -> Option<Vec<Human>> {
        unimplemented!()
    }

    async fn iq(&self) -> Option<i32> {
        unimplemented!()
    }
}

struct Alien;

#[Object(internal)]
impl Alien {
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    async fn iq(&self) -> Option<i32> {
        unimplemented!()
    }

    async fn num_eyes(&self) -> Option<i32> {
        unimplemented!()
    }
}

#[Union(internal)]
enum DogOrHuman {
    Dog(Dog),
    Human(Human),
}

#[Union(internal)]
enum HumanOrAlien {
    Human(Human),
    Alien(Alien),
}

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
enum Being {
    Dog(Dog),
    Cat(Cat),
    Human(Human),
    Alien(Alien),
}

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
enum Pet {
    Dog(Dog),
    Cat(Cat),
}

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
enum Canine {
    Dog(Dog),
}

#[Interface(internal, field(name = "iq", type = "Option<i32>"))]
enum Intelligent {
    Human(Human),
    Alien(Alien),
}

#[InputObject(internal)]
struct ComplexInput {
    required_field: bool,
    int_field: Option<i32>,
    string_field: Option<String>,
    boolean_field: Option<bool>,
    string_list_field: Option<Vec<Option<String>>>,
}

struct ComplicatedArgs;

#[Object(internal)]
impl ComplicatedArgs {
    async fn int_arg_field(&self, int_arg: Option<i32>) -> Option<String> {
        unimplemented!()
    }

    async fn non_null_int_arg_field(&self, non_null_int_arg: i32) -> Option<String> {
        unimplemented!()
    }

    async fn string_arg_field(&self, string_arg: Option<String>) -> Option<String> {
        unimplemented!()
    }

    async fn boolean_arg_field(&self, boolean_arg: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    async fn enum_arg_field(&self, enum_arg: Option<FurColor>) -> Option<String> {
        unimplemented!()
    }

    async fn float_arg_field(&self, float_arg: Option<f64>) -> Option<String> {
        unimplemented!()
    }

    async fn id_arg_field(&self, id_arg: Option<ID>) -> Option<String> {
        unimplemented!()
    }

    async fn string_list_arg_field(
        &self,
        string_list_arg: Option<Vec<Option<String>>>,
    ) -> Option<String> {
        unimplemented!()
    }

    async fn complex_arg_field(&self, complex_arg: Option<ComplexInput>) -> Option<String> {
        unimplemented!()
    }

    async fn multiple_reqs(&self, req1: i32, req2: i32) -> Option<String> {
        unimplemented!()
    }

    async fn multiple_opts(
        &self,
        #[arg(default)] opt1: i32,
        #[arg(default)] opt2: i32,
    ) -> Option<String> {
        unimplemented!()
    }

    async fn multiple_opt_and_req(
        &self,
        req1: i32,
        req2: i32,
        #[arg(default)] opt1: i32,
        #[arg(default)] opt2: i32,
    ) -> Option<String> {
        unimplemented!()
    }
}

pub struct QueryRoot;

#[Object(internal)]
impl QueryRoot {
    async fn human(&self, id: Option<ID>) -> Option<Human> {
        unimplemented!()
    }

    async fn alien(&self) -> Option<Alien> {
        unimplemented!()
    }

    async fn dog(&self) -> Option<Dog> {
        unimplemented!()
    }

    async fn cat(&self) -> Option<Cat> {
        unimplemented!()
    }

    async fn pet(&self) -> Option<Pet> {
        unimplemented!()
    }

    async fn being(&self) -> Option<Being> {
        unimplemented!()
    }

    async fn intelligent(&self) -> Option<Intelligent> {
        unimplemented!()
    }

    async fn cat_or_dog(&self) -> Option<CatOrDog> {
        unimplemented!()
    }

    async fn dog_or_human(&self) -> Option<DogOrHuman> {
        unimplemented!()
    }

    async fn human_or_alien(&self) -> Option<HumanOrAlien> {
        unimplemented!()
    }

    async fn complicated_args(&self) -> Option<ComplicatedArgs> {
        unimplemented!()
    }
}

pub struct MutationRoot;

#[Object(internal)]
impl MutationRoot {
    async fn test_input(&self, #[arg(default)] input: TestInput) -> i32 {
        unimplemented!()
    }
}

static TEST_HARNESS: Lazy<Schema<QueryRoot, MutationRoot, EmptySubscription>> =
    Lazy::new(|| Schema::new(QueryRoot, MutationRoot, EmptySubscription));

pub fn validate<'a, V, F>(doc: &'a Document, factory: F) -> Result<()>
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    let schema = &*TEST_HARNESS;
    let registry = &schema.env.registry;
    let mut ctx = VisitorContext::new(registry, doc);
    let mut visitor = factory();
    visit(&mut visitor, &mut ctx, doc);
    if !ctx.errors.is_empty() {
        return Err(Error::Rule { errors: ctx.errors });
    }
    Ok(())
}

pub(crate) fn expect_passes_rule_<'a, V, F>(doc: &'a Document, factory: F)
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    if let Err(err) = validate(doc, factory) {
        if let Error::Rule { errors } = err {
            for err in errors {
                if let Some(position) = err.locations.first() {
                    print!("[{}:{}] ", position.line, position.column);
                }
                println!("{}", err.message);
            }
        }
        panic!("Expected rule to pass, but errors found");
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! expect_passes_rule {
    ($factory:expr, $query_source:literal $(,)*) => {
        let doc = crate::parser::parse_query($query_source).expect("Parse error");
        crate::validation::test_harness::expect_passes_rule_(&doc, $factory);
    };
}

pub(crate) fn expect_fails_rule_<'a, V, F>(doc: &'a Document, factory: F)
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    if let Ok(_) = validate(doc, factory) {
        panic!("Expected rule to fail, but no errors were found");
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! expect_fails_rule {
    ($factory:expr, $query_source:literal $(,)*) => {
        let doc = crate::parser::parse_query($query_source).expect("Parse error");
        crate::validation::test_harness::expect_fails_rule_(&doc, $factory);
    };
}
