//! This is @generated code, do not edit by hand.
//! See `graphql.pest` and `tests/codegen.rs`.
#![allow(unused_attributes)]
use super::GraphQLParser;

#[allow(non_upper_case_globals)]
const _PEST_GRAMMAR_GraphQLParser: &'static str =
  include_str!("/Users/djc/src/async-graphql/parser/src/graphql.pest");
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rule {
  EOI,
  WHITESPACE,
  COMMENT,
  line_terminator,
  executable_document,
  executable_definition,
  operation_definition,
  named_operation_definition,
  variable_definitions,
  variable_definition,
  selection_set,
  selection,
  field,
  alias,
  fragment_spread,
  inline_fragment,
  fragment_definition,
  type_condition,
  service_document,
  type_system_definition,
  schema_definition,
  operation_type_definition,
  type_definition,
  scalar_type,
  object_type,
  implements_interfaces,
  interface_type,
  fields_definition,
  field_definition,
  union_type,
  union_member_types,
  enum_type,
  enum_values,
  enum_value_definition,
  input_object_type,
  input_fields_definition,
  extend,
  directive_definition,
  directive_locations,
  directive_location,
  arguments_definition,
  input_value_definition,
  operation_type,
  default_value,
  type_,
  const_value,
  value,
  variable,
  number,
  float,
  fractional,
  exponent,
  int,
  string,
  block_string_content,
  block_string_character,
  string_content,
  string_character,
  unicode_scalar_value_hex,
  boolean,
  null,
  enum_value,
  const_list,
  list,
  const_object,
  object,
  const_object_field,
  object_field,
  const_directives,
  directives,
  const_directive,
  directive,
  const_arguments,
  arguments,
  const_argument,
  argument,
  name_start,
  name,
}
#[allow(clippy::all)]
impl ::pest::Parser<Rule> for GraphQLParser {
  fn parse<'i>(
    rule: Rule,
    input: &'i str,
  ) -> ::std::result::Result<::pest::iterators::Pairs<'i, Rule>, ::pest::error::Error<Rule>> {
    mod rules {
      pub mod hidden {
        use super::super::Rule;
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn skip(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          if state.atomicity() == ::pest::Atomicity::NonAtomic {
            state.sequence(|state| {
              state
                .repeat(|state| super::visible::WHITESPACE(state))
                .and_then(|state| {
                  state.repeat(|state| {
                    state.sequence(|state| {
                      super::visible::COMMENT(state)
                        .and_then(|state| state.repeat(|state| super::visible::WHITESPACE(state)))
                    })
                  })
                })
            })
          } else {
            Ok(state)
          }
        }
      }
      pub mod visible {
        use super::super::Rule;
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn WHITESPACE(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.atomic(::pest::Atomicity::Atomic, |state| {
            state
              .match_string(" ")
              .or_else(|state| state.match_string(","))
              .or_else(|state| state.match_string("\t"))
              .or_else(|state| state.match_string("\u{feff}"))
              .or_else(|state| self::line_terminator(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn COMMENT(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.atomic(::pest::Atomicity::Atomic, |state| {
            state.sequence(|state| {
              state.match_string("#").and_then(|state| {
                state.repeat(|state| {
                  state.sequence(|state| {
                    state
                      .lookahead(false, |state| self::line_terminator(state))
                      .and_then(|state| self::ANY(state))
                  })
                })
              })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn line_terminator(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::line_terminator, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state
                .match_string("\r\n")
                .or_else(|state| state.match_string("\r"))
                .or_else(|state| state.match_string("\n"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn executable_document(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::executable_document, |state| {
            state.sequence(|state| {
              self::SOI(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::executable_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::executable_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::executable_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::EOI(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn executable_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::executable_definition, |state| {
            self::operation_definition(state).or_else(|state| self::fragment_definition(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn operation_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::operation_definition, |state| {
            self::named_operation_definition(state).or_else(|state| self::selection_set(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn named_operation_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::named_operation_definition, |state| {
            state.sequence(|state| {
              self::operation_type(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::name(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::variable_definitions(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::selection_set(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn variable_definitions(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::variable_definitions, |state| {
            state.sequence(|state| {
              state
                .match_string("(")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::variable_definition(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state)
                              .and_then(|state| self::variable_definition(state))
                          })
                        })
                      })
                    })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(")"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn variable_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::variable_definition, |state| {
            state.sequence(|state| {
              self::variable(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::type_(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::default_value(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn selection_set(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::selection_set, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::selection(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::selection(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::selection(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn selection(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::selection, |state| {
            self::field(state)
              .or_else(|state| self::inline_fragment(state))
              .or_else(|state| self::fragment_spread(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn field(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::field, |state| {
            state.sequence(|state| {
              state
                .optional(|state| self::alias(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::arguments(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::selection_set(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn alias(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::alias, |state| {
            state.sequence(|state| {
              self::name(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn fragment_spread(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::fragment_spread, |state| {
            state.sequence(|state| {
              state
                .match_string("...")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn inline_fragment(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::inline_fragment, |state| {
            state.sequence(|state| {
              state
                .match_string("...")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::type_condition(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::selection_set(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn fragment_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::fragment_definition, |state| {
            state.sequence(|state| {
              state
                .match_string("fragment")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::type_condition(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::directives(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::selection_set(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn type_condition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::type_condition, |state| {
            state.sequence(|state| {
              state
                .match_string("on")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn service_document(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::service_document, |state| {
            state.sequence(|state| {
              self::SOI(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::type_system_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::type_system_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::type_system_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::EOI(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn type_system_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::type_system_definition, |state| {
            self::schema_definition(state)
              .or_else(|state| self::type_definition(state))
              .or_else(|state| self::directive_definition(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn schema_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::schema_definition, |state| {
            state
              .sequence(|state| {
                state
                  .match_string("schema")
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("{"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| {
                    state.sequence(|state| {
                      self::operation_type_definition(state)
                        .and_then(|state| super::hidden::skip(state))
                        .and_then(|state| {
                          state.sequence(|state| {
                            state.optional(|state| {
                              self::operation_type_definition(state).and_then(|state| {
                                state.repeat(|state| {
                                  state.sequence(|state| {
                                    super::hidden::skip(state)
                                      .and_then(|state| self::operation_type_definition(state))
                                  })
                                })
                              })
                            })
                          })
                        })
                    })
                  })
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("}"))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("schema"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::const_directives(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| state.match_string("{"))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| {
                              state.sequence(|state| {
                                self::operation_type_definition(state)
                                  .and_then(|state| super::hidden::skip(state))
                                  .and_then(|state| {
                                    state.sequence(|state| {
                                      state.optional(|state| {
                                        self::operation_type_definition(state).and_then(|state| {
                                          state.repeat(|state| {
                                            state.sequence(|state| {
                                              super::hidden::skip(state).and_then(|state| {
                                                self::operation_type_definition(state)
                                              })
                                            })
                                          })
                                        })
                                      })
                                    })
                                  })
                              })
                            })
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| state.match_string("}"))
                        })
                        .or_else(|state| self::const_directives(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn operation_type_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::operation_type_definition, |state| {
            state.sequence(|state| {
              self::operation_type(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn type_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::type_definition, |state| {
            self::scalar_type(state)
              .or_else(|state| self::object_type(state))
              .or_else(|state| self::interface_type(state))
              .or_else(|state| self::union_type(state))
              .or_else(|state| self::enum_type(state))
              .or_else(|state| self::input_object_type(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn scalar_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::scalar_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("scalar"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("scalar"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::const_directives(state))
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn object_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::object_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("type"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::implements_interfaces(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::fields_definition(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("type"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::implements_interfaces(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| {
                              state
                                .sequence(|state| {
                                  state
                                    .optional(|state| self::const_directives(state))
                                    .and_then(|state| super::hidden::skip(state))
                                    .and_then(|state| self::fields_definition(state))
                                })
                                .or_else(|state| self::const_directives(state))
                            })
                        })
                        .or_else(|state| self::implements_interfaces(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn implements_interfaces(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::implements_interfaces, |state| {
            state.sequence(|state| {
              state
                .match_string("implements")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| state.match_string("&")))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      state
                        .sequence(|state| {
                          state
                            .match_string("&")
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::name(state))
                        })
                        .and_then(|state| {
                          state.repeat(|state| {
                            state.sequence(|state| {
                              super::hidden::skip(state).and_then(|state| {
                                state.sequence(|state| {
                                  state
                                    .match_string("&")
                                    .and_then(|state| super::hidden::skip(state))
                                    .and_then(|state| self::name(state))
                                })
                              })
                            })
                          })
                        })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn interface_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::interface_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("interface"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::implements_interfaces(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::fields_definition(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("interface"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.optional(|state| self::implements_interfaces(state)))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::const_directives(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::fields_definition(state))
                        })
                        .or_else(|state| self::const_directives(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn fields_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::fields_definition, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::field_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::field_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::field_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn field_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::field_definition, |state| {
            state.sequence(|state| {
              state
                .optional(|state| self::string(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::arguments_definition(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::type_(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::const_directives(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn union_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::union_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("union"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::union_member_types(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("union"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::const_directives(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::union_member_types(state))
                        })
                        .or_else(|state| self::const_directives(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn union_member_types(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::union_member_types, |state| {
            state.sequence(|state| {
              state
                .match_string("=")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| state.match_string("|")))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      state
                        .sequence(|state| {
                          state
                            .match_string("|")
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::name(state))
                        })
                        .and_then(|state| {
                          state.repeat(|state| {
                            state.sequence(|state| {
                              super::hidden::skip(state).and_then(|state| {
                                state.sequence(|state| {
                                  state
                                    .match_string("|")
                                    .and_then(|state| super::hidden::skip(state))
                                    .and_then(|state| self::name(state))
                                })
                              })
                            })
                          })
                        })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn enum_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::enum_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("enum"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::enum_values(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("enum"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::const_directives(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::enum_values(state))
                        })
                        .or_else(|state| self::const_directives(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn enum_values(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::enum_values, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::enum_value_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::enum_value_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::enum_value_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn enum_value_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::enum_value_definition, |state| {
            state.sequence(|state| {
              state
                .optional(|state| self::string(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::enum_value(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::const_directives(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn input_object_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::input_object_type, |state| {
            state
              .sequence(|state| {
                state
                  .optional(|state| self::string(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.match_string("input"))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::name(state))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::const_directives(state)))
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| state.optional(|state| self::input_fields_definition(state)))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  self::extend(state)
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| state.match_string("input"))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::name(state))
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .sequence(|state| {
                          state
                            .optional(|state| self::const_directives(state))
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::input_fields_definition(state))
                        })
                        .or_else(|state| self::const_directives(state))
                    })
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn input_fields_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::input_fields_definition, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::input_value_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::input_value_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::input_value_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn extend(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::extend, |state| state.match_string("extend"))
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn directive_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::directive_definition, |state| {
            state.sequence(|state| {
              state
                .optional(|state| self::string(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("directive"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("@"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::arguments_definition(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("on"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::directive_locations(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn directive_locations(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::directive_locations, |state| {
            state.sequence(|state| {
              state
                .optional(|state| state.match_string("|"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::directive_location(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      state
                        .sequence(|state| {
                          state
                            .match_string("|")
                            .and_then(|state| super::hidden::skip(state))
                            .and_then(|state| self::directive_location(state))
                        })
                        .and_then(|state| {
                          state.repeat(|state| {
                            state.sequence(|state| {
                              super::hidden::skip(state).and_then(|state| {
                                state.sequence(|state| {
                                  state
                                    .match_string("|")
                                    .and_then(|state| super::hidden::skip(state))
                                    .and_then(|state| self::directive_location(state))
                                })
                              })
                            })
                          })
                        })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn directive_location(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::directive_location, |state| {
            state
              .match_string("QUERY")
              .or_else(|state| state.match_string("MUTATION"))
              .or_else(|state| state.match_string("SUBSCRIPTION"))
              .or_else(|state| state.match_string("FIELD_DEFINITION"))
              .or_else(|state| state.match_string("FIELD"))
              .or_else(|state| state.match_string("FRAGMENT_DEFINITION"))
              .or_else(|state| state.match_string("FRAGMENT_SPREAD"))
              .or_else(|state| state.match_string("INLINE_FRAGMENT"))
              .or_else(|state| state.match_string("VARIABLE_DEFINITION"))
              .or_else(|state| state.match_string("SCHEMA"))
              .or_else(|state| state.match_string("SCALAR"))
              .or_else(|state| state.match_string("OBJECT"))
              .or_else(|state| state.match_string("ARGUMENT_DEFINITION"))
              .or_else(|state| state.match_string("INTERFACE"))
              .or_else(|state| state.match_string("UNION"))
              .or_else(|state| state.match_string("ENUM_VALUE"))
              .or_else(|state| state.match_string("ENUM"))
              .or_else(|state| state.match_string("INPUT_OBJECT"))
              .or_else(|state| state.match_string("INPUT_FIELD_DEFINITION"))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn arguments_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::arguments_definition, |state| {
            state.sequence(|state| {
              state
                .match_string("(")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::input_value_definition(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::input_value_definition(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::input_value_definition(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(")"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn input_value_definition(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::input_value_definition, |state| {
            state.sequence(|state| {
              state
                .optional(|state| self::string(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::type_(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::default_value(state)))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::const_directives(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn operation_type(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::operation_type, |state| {
            state
              .match_string("query")
              .or_else(|state| state.match_string("mutation"))
              .or_else(|state| state.match_string("subscription"))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn default_value(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::default_value, |state| {
            state.sequence(|state| {
              state
                .match_string("=")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::const_value(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn type_(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::type_, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state.sequence(|state| {
                self::name(state)
                  .or_else(|state| {
                    state.sequence(|state| {
                      state
                        .match_string("[")
                        .and_then(|state| self::type_(state))
                        .and_then(|state| state.match_string("]"))
                    })
                  })
                  .and_then(|state| state.optional(|state| state.match_string("!")))
              })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_value(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_value, |state| {
            self::number(state)
              .or_else(|state| self::string(state))
              .or_else(|state| self::boolean(state))
              .or_else(|state| self::null(state))
              .or_else(|state| self::enum_value(state))
              .or_else(|state| self::const_list(state))
              .or_else(|state| self::const_object(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn value(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::value, |state| {
            self::variable(state)
              .or_else(|state| self::number(state))
              .or_else(|state| self::string(state))
              .or_else(|state| self::boolean(state))
              .or_else(|state| self::null(state))
              .or_else(|state| self::enum_value(state))
              .or_else(|state| self::list(state))
              .or_else(|state| self::object(state))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn variable(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::variable, |state| {
            state.sequence(|state| {
              state
                .match_string("$")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn number(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::number, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state.sequence(|state| {
                self::float(state)
                  .or_else(|state| self::int(state))
                  .and_then(|state| state.lookahead(false, |state| self::name_start(state)))
              })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn float(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::float, |state| {
            state.sequence(|state| {
              self::int(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state
                    .sequence(|state| {
                      self::fractional(state)
                        .and_then(|state| super::hidden::skip(state))
                        .and_then(|state| self::exponent(state))
                    })
                    .or_else(|state| self::fractional(state))
                    .or_else(|state| self::exponent(state))
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn fractional(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::fractional, |state| {
            state.sequence(|state| {
              state
                .match_string(".")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_DIGIT(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::ASCII_DIGIT(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::ASCII_DIGIT(state))
                          })
                        })
                      })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn exponent(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::exponent, |state| {
            state.sequence(|state| {
              state
                .match_string("E")
                .or_else(|state| state.match_string("e"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.optional(|state| {
                    state
                      .match_string("+")
                      .or_else(|state| state.match_string("-"))
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_DIGIT(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::ASCII_DIGIT(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::ASCII_DIGIT(state))
                          })
                        })
                      })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn int(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::int, |state| {
            state.sequence(|state| {
              state
                .optional(|state| state.match_string("-"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.match_string("0").or_else(|state| {
                    state.sequence(|state| {
                      self::ASCII_NONZERO_DIGIT(state)
                        .and_then(|state| super::hidden::skip(state))
                        .and_then(|state| {
                          state.sequence(|state| {
                            state.optional(|state| {
                              self::ASCII_DIGIT(state).and_then(|state| {
                                state.repeat(|state| {
                                  state.sequence(|state| {
                                    super::hidden::skip(state)
                                      .and_then(|state| self::ASCII_DIGIT(state))
                                  })
                                })
                              })
                            })
                          })
                        })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn string(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.atomic(::pest::Atomicity::CompoundAtomic, |state| {
            state.rule(Rule::string, |state| {
              state
                .sequence(|state| {
                  state
                    .match_string("\"\"\"")
                    .and_then(|state| self::block_string_content(state))
                    .and_then(|state| state.match_string("\"\"\""))
                })
                .or_else(|state| {
                  state.sequence(|state| {
                    state
                      .match_string("\"")
                      .and_then(|state| self::string_content(state))
                      .and_then(|state| state.match_string("\""))
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn block_string_content(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::block_string_content, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state.repeat(|state| self::block_string_character(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn block_string_character(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::block_string_character, |state| {
            state
              .sequence(|state| {
                state
                  .lookahead(false, |state| {
                    state
                      .match_string("\"\"\"")
                      .or_else(|state| state.match_string("\\\"\"\""))
                  })
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::ANY(state))
              })
              .or_else(|state| state.match_string("\\\"\"\""))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn string_content(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::string_content, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state.repeat(|state| self::string_character(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn string_character(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::string_character, |state| {
            state
              .sequence(|state| {
                state
                  .lookahead(false, |state| {
                    state
                      .match_string("\"")
                      .or_else(|state| state.match_string("\\"))
                      .or_else(|state| self::line_terminator(state))
                  })
                  .and_then(|state| super::hidden::skip(state))
                  .and_then(|state| self::ANY(state))
              })
              .or_else(|state| {
                state.sequence(|state| {
                  state
                    .match_string("\\")
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| {
                      state
                        .match_string("\"")
                        .or_else(|state| state.match_string("\\"))
                        .or_else(|state| state.match_string("/"))
                        .or_else(|state| state.match_string("b"))
                        .or_else(|state| state.match_string("f"))
                        .or_else(|state| state.match_string("n"))
                        .or_else(|state| state.match_string("r"))
                        .or_else(|state| state.match_string("t"))
                    })
                })
              })
              .or_else(|state| {
                state.sequence(|state| {
                  state
                    .match_string("\\u")
                    .and_then(|state| super::hidden::skip(state))
                    .and_then(|state| self::unicode_scalar_value_hex(state))
                })
              })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn unicode_scalar_value_hex(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::unicode_scalar_value_hex, |state| {
            state.sequence(|state| {
              state
                .lookahead(false, |state| {
                  state.sequence(|state| {
                    state
                      .match_insensitive("d")
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state
                          .match_range('8'..'9')
                          .or_else(|state| state.match_range('a'..'f'))
                          .or_else(|state| state.match_range('A'..'F'))
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_HEX_DIGIT(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_HEX_DIGIT(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_HEX_DIGIT(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::ASCII_HEX_DIGIT(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn boolean(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::boolean, |state| {
            state
              .match_string("true")
              .or_else(|state| state.match_string("false"))
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn null(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::null, |state| state.match_string("null"))
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn enum_value(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.atomic(::pest::Atomicity::CompoundAtomic, |state| {
            state.rule(Rule::enum_value, |state| {
              state.sequence(|state| {
                state
                  .lookahead(false, |state| {
                    self::boolean(state).or_else(|state| self::null(state))
                  })
                  .and_then(|state| self::name(state))
              })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_list(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_list, |state| {
            state.sequence(|state| {
              state
                .match_string("[")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::const_value(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::const_value(state))
                          })
                        })
                      })
                    })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("]"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn list(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::list, |state| {
            state.sequence(|state| {
              state
                .match_string("[")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::value(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::value(state))
                          })
                        })
                      })
                    })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("]"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_object(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_object, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::const_object_field(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state)
                              .and_then(|state| self::const_object_field(state))
                          })
                        })
                      })
                    })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn object(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::object, |state| {
            state.sequence(|state| {
              state
                .match_string("{")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::object_field(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::object_field(state))
                          })
                        })
                      })
                    })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string("}"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_object_field(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_object_field, |state| {
            state.sequence(|state| {
              self::name(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::const_value(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn object_field(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::object_field, |state| {
            state.sequence(|state| {
              self::name(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::value(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_directives(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_directives, |state| {
            state.sequence(|state| {
              self::const_directive(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::const_directive(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state)
                              .and_then(|state| self::const_directive(state))
                          })
                        })
                      })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn directives(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::directives, |state| {
            state.sequence(|state| {
              self::directive(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    state.optional(|state| {
                      self::directive(state).and_then(|state| {
                        state.repeat(|state| {
                          state.sequence(|state| {
                            super::hidden::skip(state).and_then(|state| self::directive(state))
                          })
                        })
                      })
                    })
                  })
                })
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_directive(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_directive, |state| {
            state.sequence(|state| {
              state
                .match_string("@")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::const_arguments(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn directive(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::directive, |state| {
            state.sequence(|state| {
              state
                .match_string("@")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::name(state))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.optional(|state| self::arguments(state)))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_arguments(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_arguments, |state| {
            state.sequence(|state| {
              state
                .match_string("(")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::const_argument(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::const_argument(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state)
                                    .and_then(|state| self::const_argument(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(")"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn arguments(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::arguments, |state| {
            state.sequence(|state| {
              state
                .match_string("(")
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| {
                  state.sequence(|state| {
                    self::argument(state)
                      .and_then(|state| super::hidden::skip(state))
                      .and_then(|state| {
                        state.sequence(|state| {
                          state.optional(|state| {
                            self::argument(state).and_then(|state| {
                              state.repeat(|state| {
                                state.sequence(|state| {
                                  super::hidden::skip(state).and_then(|state| self::argument(state))
                                })
                              })
                            })
                          })
                        })
                      })
                  })
                })
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(")"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn const_argument(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::const_argument, |state| {
            state.sequence(|state| {
              self::name(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::const_value(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn argument(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::argument, |state| {
            state.sequence(|state| {
              self::name(state)
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| state.match_string(":"))
                .and_then(|state| super::hidden::skip(state))
                .and_then(|state| self::value(state))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn name_start(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::name_start, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              self::ASCII_ALPHA(state).or_else(|state| state.match_string("_"))
            })
          })
        }
        #[inline]
        #[allow(non_snake_case, unused_variables)]
        pub fn name(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::name, |state| {
            state.atomic(::pest::Atomicity::Atomic, |state| {
              state.sequence(|state| {
                self::name_start(state).and_then(|state| {
                  state.repeat(|state| {
                    self::ASCII_ALPHA(state)
                      .or_else(|state| self::ASCII_DIGIT(state))
                      .or_else(|state| state.match_string("_"))
                  })
                })
              })
            })
          })
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn ANY(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.skip(1)
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn EOI(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.rule(Rule::EOI, |state| state.end_of_input())
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn SOI(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.start_of_input()
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn ASCII_DIGIT(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.match_range('0'..'9')
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn ASCII_NONZERO_DIGIT(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state.match_range('1'..'9')
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn ASCII_HEX_DIGIT(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state
            .match_range('0'..'9')
            .or_else(|state| state.match_range('a'..'f'))
            .or_else(|state| state.match_range('A'..'F'))
        }
        #[inline]
        #[allow(dead_code, non_snake_case, unused_variables)]
        pub fn ASCII_ALPHA(
          state: Box<::pest::ParserState<Rule>>,
        ) -> ::pest::ParseResult<Box<::pest::ParserState<Rule>>> {
          state
            .match_range('a'..'z')
            .or_else(|state| state.match_range('A'..'Z'))
        }
      }
      pub use self::visible::*;
    }
    ::pest::state(input, |state| match rule {
      Rule::WHITESPACE => rules::WHITESPACE(state),
      Rule::COMMENT => rules::COMMENT(state),
      Rule::line_terminator => rules::line_terminator(state),
      Rule::executable_document => rules::executable_document(state),
      Rule::executable_definition => rules::executable_definition(state),
      Rule::operation_definition => rules::operation_definition(state),
      Rule::named_operation_definition => rules::named_operation_definition(state),
      Rule::variable_definitions => rules::variable_definitions(state),
      Rule::variable_definition => rules::variable_definition(state),
      Rule::selection_set => rules::selection_set(state),
      Rule::selection => rules::selection(state),
      Rule::field => rules::field(state),
      Rule::alias => rules::alias(state),
      Rule::fragment_spread => rules::fragment_spread(state),
      Rule::inline_fragment => rules::inline_fragment(state),
      Rule::fragment_definition => rules::fragment_definition(state),
      Rule::type_condition => rules::type_condition(state),
      Rule::service_document => rules::service_document(state),
      Rule::type_system_definition => rules::type_system_definition(state),
      Rule::schema_definition => rules::schema_definition(state),
      Rule::operation_type_definition => rules::operation_type_definition(state),
      Rule::type_definition => rules::type_definition(state),
      Rule::scalar_type => rules::scalar_type(state),
      Rule::object_type => rules::object_type(state),
      Rule::implements_interfaces => rules::implements_interfaces(state),
      Rule::interface_type => rules::interface_type(state),
      Rule::fields_definition => rules::fields_definition(state),
      Rule::field_definition => rules::field_definition(state),
      Rule::union_type => rules::union_type(state),
      Rule::union_member_types => rules::union_member_types(state),
      Rule::enum_type => rules::enum_type(state),
      Rule::enum_values => rules::enum_values(state),
      Rule::enum_value_definition => rules::enum_value_definition(state),
      Rule::input_object_type => rules::input_object_type(state),
      Rule::input_fields_definition => rules::input_fields_definition(state),
      Rule::extend => rules::extend(state),
      Rule::directive_definition => rules::directive_definition(state),
      Rule::directive_locations => rules::directive_locations(state),
      Rule::directive_location => rules::directive_location(state),
      Rule::arguments_definition => rules::arguments_definition(state),
      Rule::input_value_definition => rules::input_value_definition(state),
      Rule::operation_type => rules::operation_type(state),
      Rule::default_value => rules::default_value(state),
      Rule::type_ => rules::type_(state),
      Rule::const_value => rules::const_value(state),
      Rule::value => rules::value(state),
      Rule::variable => rules::variable(state),
      Rule::number => rules::number(state),
      Rule::float => rules::float(state),
      Rule::fractional => rules::fractional(state),
      Rule::exponent => rules::exponent(state),
      Rule::int => rules::int(state),
      Rule::string => rules::string(state),
      Rule::block_string_content => rules::block_string_content(state),
      Rule::block_string_character => rules::block_string_character(state),
      Rule::string_content => rules::string_content(state),
      Rule::string_character => rules::string_character(state),
      Rule::unicode_scalar_value_hex => rules::unicode_scalar_value_hex(state),
      Rule::boolean => rules::boolean(state),
      Rule::null => rules::null(state),
      Rule::enum_value => rules::enum_value(state),
      Rule::const_list => rules::const_list(state),
      Rule::list => rules::list(state),
      Rule::const_object => rules::const_object(state),
      Rule::object => rules::object(state),
      Rule::const_object_field => rules::const_object_field(state),
      Rule::object_field => rules::object_field(state),
      Rule::const_directives => rules::const_directives(state),
      Rule::directives => rules::directives(state),
      Rule::const_directive => rules::const_directive(state),
      Rule::directive => rules::directive(state),
      Rule::const_arguments => rules::const_arguments(state),
      Rule::arguments => rules::arguments(state),
      Rule::const_argument => rules::const_argument(state),
      Rule::argument => rules::argument(state),
      Rule::name_start => rules::name_start(state),
      Rule::name => rules::name(state),
      Rule::EOI => rules::EOI(state),
    })
  }
}
