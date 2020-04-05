#![allow(unused_variables)]
#![allow(dead_code)]

use crate::validation::visitor::{visit, Visitor, VisitorContext};
use crate::*;
use graphql_parser::parse_query;

#[InputObject(internal)]
struct TestInput {
    id: i32,
    name: String,
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
    #[field]
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn nickname(&self) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn bark_volume(&self) -> Option<i32> {
        unimplemented!()
    }

    #[field]
    async fn barks(&self) -> Option<bool> {
        unimplemented!()
    }

    #[field]
    async fn does_know_command(&self, dog_command: Option<DogCommand>) -> Option<bool> {
        unimplemented!()
    }

    #[field]
    async fn is_housetrained(&self, #[arg(default = "true")] at_other_homes: bool) -> Option<bool> {
        unimplemented!()
    }

    #[field]
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
    #[field]
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn nickname(&self) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn meows(&self) -> Option<bool> {
        unimplemented!()
    }

    #[field]
    async fn meow_volume(&self) -> Option<i32> {
        unimplemented!()
    }

    #[field]
    async fn fur_color(&self) -> Option<FurColor> {
        unimplemented!()
    }
}

#[Union(internal)]
struct CatOrDog(Cat, Dog);

struct Human;

#[Object(internal)]
impl Human {
    #[field]
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn pets(&self) -> Option<Vec<Option<Pet>>> {
        unimplemented!()
    }

    #[field]
    async fn relatives(&self) -> Option<Vec<Human>> {
        unimplemented!()
    }

    #[field]
    async fn iq(&self) -> Option<i32> {
        unimplemented!()
    }
}

struct Alien;

#[Object(internal)]
impl Alien {
    #[field]
    async fn name(&self, surname: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn iq(&self) -> Option<i32> {
        unimplemented!()
    }

    #[field]
    async fn num_eyes(&self) -> Option<i32> {
        unimplemented!()
    }
}

#[Union(internal)]
struct DogOrHuman(Dog, Human);

#[Union(internal)]
struct HumanOrAlien(Human, Alien);

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
struct Being(Dog, Cat, Human, Alien);

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
struct Pet(Dog, Cat);

#[Interface(
    internal,
    field(
        name = "name",
        type = "Option<String>",
        arg(name = "surname", type = "Option<bool>")
    )
)]
struct Canine(Dog);

#[Interface(internal, field(name = "iq", type = "Option<i32>"))]
struct Intelligent(Human, Alien);

#[InputObject(internal)]
struct ComplexInput {
    #[field]
    required_field: bool,

    #[field]
    int_field: Option<i32>,

    #[field]
    string_field: Option<String>,

    #[field]
    boolean_field: Option<bool>,

    #[field]
    string_list_field: Option<Vec<Option<String>>>,
}

struct ComplicatedArgs;

#[Object(internal)]
impl ComplicatedArgs {
    #[field]
    async fn int_arg_field(&self, int_arg: Option<i32>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn non_null_int_arg_field(&self, non_null_int_arg: i32) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn string_arg_field(&self, string_arg: Option<String>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn boolean_arg_field(&self, boolean_arg: Option<bool>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn enum_arg_field(&self, enum_arg: Option<FurColor>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn float_arg_field(&self, float_arg: Option<f64>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn id_arg_field(&self, id_arg: Option<ID>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn string_list_arg_field(
        &self,
        string_list_arg: Option<Vec<Option<String>>>,
    ) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn complex_arg_field(&self, complex_arg: Option<ComplexInput>) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn multiple_reqs(&self, req1: i32, req2: i32) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn multiple_opts(
        &self,
        #[arg(default = "0")] opt1: i32,
        #[arg(default = "0")] opt2: i32,
    ) -> Option<String> {
        unimplemented!()
    }

    #[field]
    async fn multiple_opt_and_req(
        &self,
        req1: i32,
        req2: i32,
        #[arg(default = "0")] opt1: i32,
        #[arg(default = "0")] opt2: i32,
    ) -> Option<String> {
        unimplemented!()
    }
}

pub struct QueryRoot;

#[Object(internal)]
impl QueryRoot {
    #[field]
    async fn human(&self, id: Option<ID>) -> Option<Human> {
        unimplemented!()
    }

    #[field]
    async fn alien(&self) -> Option<Alien> {
        unimplemented!()
    }

    #[field]
    async fn dog(&self) -> Option<Dog> {
        unimplemented!()
    }

    #[field]
    async fn cat(&self) -> Option<Cat> {
        unimplemented!()
    }

    #[field]
    async fn pet(&self) -> Option<Pet> {
        unimplemented!()
    }

    #[field]
    async fn being(&self) -> Option<Being> {
        unimplemented!()
    }

    #[field]
    async fn intelligent(&self) -> Option<Intelligent> {
        unimplemented!()
    }

    #[field]
    async fn cat_or_dog(&self) -> Option<CatOrDog> {
        unimplemented!()
    }

    #[field]
    async fn dog_or_human(&self) -> Option<DogOrHuman> {
        unimplemented!()
    }

    #[field]
    async fn human_or_alien(&self) -> Option<HumanOrAlien> {
        unimplemented!()
    }

    #[field]
    async fn complicated_args(&self) -> Option<ComplicatedArgs> {
        unimplemented!()
    }
}

pub struct MutationRoot;

#[Object(internal)]
impl MutationRoot {
    #[field]
    async fn test_input(
        &self,
        #[arg(default = r#"{id: 423, name: "foo"}"#)] input: TestInput,
    ) -> i32 {
        unimplemented!()
    }
}

pub struct SubscriptionRoot;

#[Subscription(internal)]
impl SubscriptionRoot {}

pub fn expect_passes_rule<'a, V, F>(factory: F, query_source: &str)
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    expect_passes_rule_with_schema(
        QueryRoot,
        MutationRoot,
        SubscriptionRoot,
        factory,
        query_source,
    );
}

pub fn expect_fails_rule<'a, V, F>(factory: F, query_source: &str)
where
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    expect_fails_rule_with_schema(
        QueryRoot,
        MutationRoot,
        SubscriptionRoot,
        factory,
        query_source,
    );
}

pub fn validate<'a, Query, Mutation, Subscription, V, F>(
    query: Query,
    mutation: Mutation,
    subscription: Subscription,
    factory: F,
    query_source: &str,
) -> Result<()>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    let schema = Schema::new(query, mutation, subscription);
    let registry = &schema.0.registry;
    let doc = parse_query(query_source).expect("Parse error");
    let mut ctx = VisitorContext::new(
        unsafe { ::std::mem::transmute(&schema.0.registry) },
        unsafe { ::std::mem::transmute(&doc) },
    );
    let mut visitor = factory();
    visit(&mut visitor, &mut ctx, unsafe {
        ::std::mem::transmute(&doc)
    });
    if !ctx.errors.is_empty() {
        return Err(Error::Rule { errors: ctx.errors });
    }
    Ok(())
}

pub fn expect_passes_rule_with_schema<'a, Query, Mutation, Subscription, V, F>(
    query: Query,
    mutation: Mutation,
    subscription: Subscription,
    factory: F,
    query_source: &str,
) where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    if let Err(err) = validate(query, mutation, subscription, factory, query_source) {
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

pub fn expect_fails_rule_with_schema<'a, Query, Mutation, Subscription, V, F>(
    query: Query,
    mutation: Mutation,
    subscription: Subscription,
    factory: F,
    query_source: &str,
) where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
    V: Visitor<'a> + 'a,
    F: Fn() -> V,
{
    if let Ok(_) = validate(query, mutation, subscription, factory, query_source) {
        panic!("Expected rule to fail, but no errors were found");
    }
}
