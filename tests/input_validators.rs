use async_graphql::validators::{
    Email, IntEqual, IntGreaterThan, IntLessThan, IntNonZero, IntRange, ListMaxLength,
    ListMinLength, StringMaxLength, StringMinLength, MAC,
};
use async_graphql::*;

#[async_std::test]
pub async fn test_input_validator_string_min_length() {
    struct QueryRoot;

    #[InputObject]
    struct InputMaxLength {
        #[field(validator(StringMinLength(length = "6")))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(StringMinLength(length = "6")))] _id: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputMaxLength) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let test_cases = [
        "abc",
        "acbce",
        "abcdef",
        "abcdefghi",
        "abcdefghijkl",
        "abcdefghijklmnop",
    ];

    let validator_length = 6;
    for case in &test_cases {
        let field_query = format!("{{fieldParameter(id: \"{}\")}}", case);
        let object_query = format!("{{inputObject(input: {{id: \"{}\"}})}}", case);
        let case_length = case.len();

        if case_length < validator_length {
            let should_fail_msg = format!(
                "StringMinValue case {} should have failed, but did not",
                case
            );

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value length is {}, must be greater than or equal to {}",
                case_length, validator_length
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value length is {}, must be greater than or equal to {}",
                case_length, validator_length
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with StringMinLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with StringMinLength",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_string_max_length() {
    struct QueryRoot;

    #[InputObject]
    struct InputMaxLength {
        #[field(validator(StringMaxLength(length = "6")))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(StringMaxLength(length = "6")))] _id: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputMaxLength) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let test_cases = [
        "abc",
        "acbce",
        "abcdef",
        "abcdefghi",
        "abcdefghijkl",
        "abcdefghijklmnop",
    ];

    let validator_length = 6;
    for case in &test_cases {
        let field_query = format!("{{fieldParameter(id: \"{}\")}}", case);
        let object_query = format!("{{inputObject(input: {{id: \"{}\"}})}}", case);
        let case_length = case.len();

        if case_length > validator_length {
            let should_fail_msg = format!(
                "StringMaxValue case {} should have failed, but did not",
                case
            );

            let field_error_msg = format!("Invalid value for argument \"id\", the value length is {}, must be less than or equal to {}", case_length, validator_length);
            let object_error_msg = format!("Invalid value for argument \"input.id\", the value length is {}, must be less than or equal to {}", case_length, validator_length);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with StringMaxLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with StringMaxLength",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_string_email() {
    struct QueryRoot;

    #[InputObject]
    struct InputEmail {
        #[field(validator(Email))]
        pub email: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(&self, #[arg(validator(Email))] _email: String) -> bool {
            true
        }

        async fn input_object(&self, _input: InputEmail) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    // Source https://gist.github.com/cjaoude/fd9910626629b53c4d25
    let test_cases = [
        // Invalid emails
        ("plainaddress", true),
        // ("#@%^%#$@#$@#.com", true),
        ("@example.com", true),
        ("Joe Smith <email@example.com>", true),
        ("email.example.com", true),
        // ("email@example@example.com", true),
        // (".email@example.com", true),
        // ("email.@example.com", true),
        // ("email..email@example.com", true),
        ("あいうえお@example.com", true),
        // ("email@example.com (Joe Smith)", true),
        // ("email@example", true),
        // ("email@-example.com", true),
        // ("email@example.web", true),
        // ("email@111.222.333.44444", true),
        // ("email@example..com", true),
        // ("Abc..123@example.com", true),
        // Valid Emails
        ("email@example.com", false),
        ("firstname.lastname@example.com", false),
        ("email@subdomain.example.com", false),
        ("firstname+lastname@example.com", false),
        ("email@123.123.123.123", false),
        ("email@[123.123.123.123]", false),
        // This returns parsing error
        // (r#""email"@example.com"#, false),
        ("1234567890@example.com", false),
        ("email@example-one.com", false),
        ("_______@example.com", false),
        ("email@example.name", false),
        ("email@example.museum", false),
        ("email@example.co.jp", false),
        ("firstname-lastname@example.com", false),
    ];

    for (case, should_fail) in &test_cases {
        let field_query = format!("{{fieldParameter(email: \"{}\")}}", case);
        let object_query = format!("{{inputObject(input: {{email: \"{}\"}})}}", case);

        if *should_fail {
            let should_fail_msg = format!(
                "Email validation case {} should have failed, but did not",
                case
            );
            let field_error_msg =
                format!("Invalid value for argument \"email\", invalid email format");
            let object_error_msg =
                format!("Invalid value for argument \"input.email\", invalid email format");

            // Testing FieldValidator
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            // Testing ObjectValidator
            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);

            // Field Paramter
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with Email",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with Email",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_string_mac() {
    struct QueryRootWithColon;
    struct QueryRootWithoutColon;

    #[InputObject]
    struct InputMACWithColon {
        #[field(validator(MAC(colon = "true")))]
        pub mac: String,
    }

    #[InputObject]
    struct InputMACWithoutColon {
        #[field(validator(MAC(colon = "false")))]
        pub mac: String,
    }

    #[Object]
    impl QueryRootWithColon {
        async fn field_parameter(
            &self,
            #[arg(validator(MAC(colon = "true")))] _mac: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputMACWithColon) -> bool {
            true
        }
    }

    #[Object]
    impl QueryRootWithoutColon {
        async fn field_parameter(
            &self,
            #[arg(validator(MAC(colon = "false")))] _mac: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputMACWithoutColon) -> bool {
            true
        }
    }

    let schema_with_colon = Schema::new(QueryRootWithColon, EmptyMutation, EmptySubscription);
    let schema_without_colon = Schema::new(QueryRootWithoutColon, EmptyMutation, EmptySubscription);

    let valid_macs = vec![
        "28:11:32:7F:82:55",
        "B3:DC:09:DE:6B:77",
        "BD:FB:D5:F2:4B:1F",
        "1E:5B:76:FF:23:04",
        "00:00:00:00:00:00",
        "2811327F8255",
        "B3DC09DE6B77",
        "BDFBD5F24B1F",
        "1E5B76FF2304",
        "000000000000",
    ];

    let invalid_macs = vec![
        "AB:CD",
        "ABCD",
        "ABCDEFHGIJKL",
        "HJ11327F8255",
        "ZZ:ZZ:ZZ:ZZ:ZZ:ZZ",
        "AB:CD:EF:GH:IJ:KL",
        "AB:CD:EF:HG:IJ:KL",
        "HJ:11:32:7F:82:55",
    ];

    for mac in invalid_macs {
        let field_query = format!("{{fieldParameter(mac: \"{}\")}}", mac);
        let object_query = format!("{{inputObject(input: {{mac: \"{}\"}})}}", mac);
        let should_fail_msg = format!(
            "MAC validation case {} should have failed, but did not",
            mac
        );
        let field_error_msg = format!("Invalid value for argument \"mac\", invalid MAC format");
        let object_error_msg =
            format!("Invalid value for argument \"input.mac\", invalid MAC format");

        assert_eq!(
            schema_without_colon
                .execute(&field_query)
                .await
                .expect_err(&should_fail_msg[..]),
            Error::Rule {
                errors: vec!(RuleError {
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    message: field_error_msg.clone()
                })
            }
        );

        // Testing ObjectValidator
        assert_eq!(
            schema_without_colon
                .execute(&object_query)
                .await
                .expect_err(&should_fail_msg[..]),
            Error::Rule {
                errors: vec!(RuleError {
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    message: object_error_msg.clone()
                })
            }
        );

        assert_eq!(
            schema_with_colon
                .execute(&field_query)
                .await
                .expect_err(&should_fail_msg[..]),
            Error::Rule {
                errors: vec!(RuleError {
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    message: field_error_msg
                })
            }
        );

        // Testing ObjectValidator
        assert_eq!(
            schema_with_colon
                .execute(&object_query)
                .await
                .expect_err(&should_fail_msg[..]),
            Error::Rule {
                errors: vec!(RuleError {
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    message: object_error_msg
                })
            }
        );
    }

    for mac in valid_macs {
        let field_query = format!("{{fieldParameter(mac: \"{}\")}}", mac);
        let object_query = format!("{{inputObject(input: {{mac: \"{}\"}})}}", mac);
        let contains_colon = mac.contains(":");
        let should_fail_msg = format!(
            "MAC validation case {} should have failed, but did not",
            mac
        );
        let field_error_msg = format!("Invalid value for argument \"mac\", invalid MAC format");
        let object_error_msg =
            format!("Invalid value for argument \"input.mac\", invalid MAC format");
        let error_msg = format!("Schema returned error with test_string = {}", mac);

        if contains_colon {
            // Field Paramter
            assert_eq!(
                schema_with_colon
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_with_colon
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_without_colon
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            // Testing ObjectValidator
            assert_eq!(
                schema_without_colon
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            assert_eq!(
                schema_without_colon
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_without_colon
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_with_colon
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            // Testing ObjectValidator
            assert_eq!(
                schema_with_colon
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_int_range() {
    struct QueryRoot;

    #[InputObject]
    struct InputIntRange {
        #[field(validator(IntRange(min = "-2", max = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(IntRange(min = "-2", max = "5")))] _id: i32,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntRange) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let min: i32 = -2;
    let max: i32 = 5;

    for case in -10..10 {
        let field_query = format!("{{fieldParameter(id: {})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {}}})}}", case);

        if case < min || case > max {
            let should_fail_msg = format!("IntRange case {} should have failed, but did not", case);

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value is {}, must be between {} and {}",
                case, min, max
            );
            let object_error_msg = format!("Invalid value for argument \"input.id\", the value is {}, must be between {} and {}", case, min, max);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with IntRange",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with IntRange",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_int_less_than() {
    struct QueryRoot;

    #[InputObject]
    struct InputIntLessThan {
        #[field(validator(IntLessThan(value = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(IntLessThan(value = "5")))] _id: i32,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntLessThan) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let max: i32 = 5;

    for case in -10..10 {
        let field_query = format!("{{fieldParameter(id: {})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {}}})}}", case);

        if case >= max {
            let should_fail_msg =
                format!("IntLessThan case {} should have failed, but did not", case);

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value is {}, must be less than {}",
                case, max
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value is {}, must be less than {}",
                case, max
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with IntLessThan",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with IntLessThan",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_int_greater_than() {
    struct QueryRoot;

    #[InputObject]
    struct InputIntGreaterThan {
        #[field(validator(IntGreaterThan(value = "3")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(IntGreaterThan(value = "3")))] _id: i32,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntGreaterThan) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let min: i32 = 3;

    for case in -10..10 {
        let field_query = format!("{{fieldParameter(id: {})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {}}})}}", case);

        if case <= min {
            let should_fail_msg = format!(
                "IntGreaterThan case {} should have failed, but did not",
                case
            );

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value is {}, must be greater than {}",
                case, min
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value is {}, must be greater than {}",
                case, min
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with IntGreaterThan",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with IntGreaterThan",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_int_nonzero() {
    struct QueryRoot;

    #[InputObject]
    struct InputIntNonZero {
        #[field(validator(IntNonZero))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(&self, #[arg(validator(IntNonZero))] _id: i32) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntNonZero) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);

    for case in -10..10 {
        let field_query = format!("{{fieldParameter(id: {})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {}}})}}", case);
        if case == 0 {
            let should_fail_msg =
                format!("IntNonZero case {} should have failed, but did not", case);

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value is {}, must be nonzero",
                case
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value is {}, must be nonzero",
                case
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with IntNonZero",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with IntNonZero",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_int_equal() {
    struct QueryRoot;

    #[InputObject]
    struct InputIntEqual {
        #[field(validator(IntEqual(value = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(&self, #[arg(validator(IntEqual(value = "5")))] _id: i32) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntEqual) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let equal_to = 5;

    for case in -10..10 {
        let field_query = format!("{{fieldParameter(id: {})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {}}})}}", case);
        if case != equal_to {
            let should_fail_msg =
                format!("IntNonZero case {} should have failed, but did not", case);

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value is {}, must be equal to {}",
                case, equal_to
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value is {}, must be equal to {}",
                case, equal_to
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {} with IntEqual",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {} with IntEqual",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_list_max_length() {
    struct QueryRoot;

    #[InputObject]
    struct InputListMaxLength {
        #[field(validator(ListMaxLength(length = "5")))]
        pub id: Vec<i32>,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(ListMaxLength(length = "5")))] _id: Vec<i32>,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputListMaxLength) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let max_length: usize = 5;
    let test_cases: Vec<Vec<i32>> = vec![
        vec![1],
        vec![1, 2, 3],
        vec![1, 2, 3, 4],
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5, 6],
        vec![1, 2, 3, 4, 5, 6, 7, 8],
    ];

    for case in test_cases.iter() {
        let field_query = format!("{{fieldParameter(id: {:?})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {:?}}})}}", case);
        let case_length = case.len();
        if case_length > max_length {
            let should_fail_msg = format!(
                "ListMaxLength case {:?} should have failed, but did not",
                case
            );

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value length is {}, must be less than or equal to {}",
                case_length,
                max_length
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value length is {}, must be less than or equal to {}",
                case_length, max_length
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {:?} with ListMaxLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {:?} with ListMaxLength",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_list_min_length() {
    struct QueryRoot;

    #[InputObject]
    struct InputListMinLength {
        #[field(validator(ListMinLength(length = "4")))]
        pub id: Vec<i32>,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(ListMinLength(length = "4")))] _id: Vec<i32>,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputListMinLength) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let min_length: usize = 4;
    let test_cases: Vec<Vec<i32>> = vec![
        vec![1],
        vec![1, 2, 3],
        vec![1, 2, 3, 4],
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5, 6],
        vec![1, 2, 3, 4, 5, 6, 7, 8],
    ];

    for case in test_cases.iter() {
        let field_query = format!("{{fieldParameter(id: {:?})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {:?}}})}}", case);
        let case_length = case.len();
        if case_length < min_length {
            let should_fail_msg = format!(
                "ListMinLength case {:?} should have failed, but did not",
                case
            );

            let field_error_msg = format!(
                "Invalid value for argument \"id\", the value length is {}, must be greater than or equal to {}",
                case_length,
                min_length
            );
            let object_error_msg = format!(
                "Invalid value for argument \"input.id\", the value length is {}, must be greater than or equal to {}",
                case_length, min_length
            );
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {:?} with ListMinLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {:?} with ListMinLength",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_operator_or() {
    struct QueryRoot;

    #[InputObject]
    struct InputOrValidator {
        #[field(validator(or(Email, MAC(colon = "false"))))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(or(Email, MAC(colon = "false"))))] _id: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputOrValidator) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let test_cases = [
        ("2811327F8255", false),
        ("B3DC09DE6B77", false),
        ("BDFBD5F24B1F", false),
        ("1E5B76FF2304", false),
        ("000000000000", false),
        ("email@example.com", false),
        ("firstname.lastname@example.com", false),
        ("email@subdomain.example.com", false),
        ("firstname+lastname@example.com", false),
        ("AB:CD", true),
        ("ABCD", true),
        ("ABCDEFHGIJKL", true),
        ("HJ11327F8255", true),
        ("ZZ:ZZ:ZZ:ZZ:ZZ:ZZ", true),
        ("AB:CD:EF:GH:IJ:KL", true),
        ("AB:CD:EF:HG:IJ:KL", true),
        ("HJ:11:32:7F:82:55", true),
        ("plainaddress", true),
        ("@example.com", true),
        ("Joe Smith <email@example.com>", true),
        ("email.example.com", true),
    ];

    for (case, should_fail) in &test_cases {
        let field_query = format!("{{fieldParameter(id: {:?})}}", case);
        let object_query = format!("{{inputObject(input: {{id: {:?}}})}}", case);
        if *should_fail {
            let should_fail_msg = format!(
                "OR operator case {:?} should have failed, but did not",
                case
            );

            let field_error_msg = format!("Invalid value for argument \"id\", invalid MAC format");
            let object_error_msg =
                format!("Invalid value for argument \"input.id\", invalid MAC format");
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {:?} with OR operator",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {:?} with OR operator",
                case
            );
        }
    }
}

#[async_std::test]
pub async fn test_input_validator_operator_and() {
    struct QueryRoot;

    #[InputObject]
    struct InputAndValidator {
        #[field(validator(and(Email, StringMinLength(length = "14"))))]
        pub email: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[arg(validator(and(Email, StringMinLength(length = "14"))))] _email: String,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputAndValidator) -> bool {
            true
        }
    }

    let min_length = 14;
    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let test_cases = [
        ("2811327F8255", true, true),
        ("a@example.com", true, false),
        ("firstname.lastname@example.com", false, false),
        ("email@subdomain.example.com", false, false),
    ];

    for (case, should_fail, should_be_invalid_email) in &test_cases {
        let case_length = case.len();
        let field_query = format!("{{fieldParameter(email: {:?})}}", case);
        let object_query = format!("{{inputObject(input: {{email: {:?}}})}}", case);
        if *should_fail {
            let should_fail_msg = format!(
                "AND operator case {:?} should have failed, but did not",
                case
            );

            let field_error_msg = if *should_be_invalid_email {
                format!("Invalid value for argument \"email\", invalid email format")
            } else {
                format!("Invalid value for argument \"email\", the value length is {}, must be greater than or equal to {}", case_length, min_length)
            };

            let object_error_msg = if *should_be_invalid_email {
                format!("Invalid value for argument \"input.email\", invalid email format")
            } else {
                format!("Invalid value for argument \"input.email\", the value length is {}, must be greater than or equal to {}", case_length, min_length)
            };

            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 17
                        }),
                        message: field_error_msg
                    })
                }
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect_err(&should_fail_msg[..]),
                Error::Rule {
                    errors: vec!(RuleError {
                        locations: vec!(Pos {
                            line: 1,
                            column: 14
                        }),
                        message: object_error_msg
                    })
                }
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"fieldParameter": true}),
                "Failed to validate {:?} with AND operator",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .expect(&error_msg[..])
                    .data,
                serde_json::json!({"inputObject": true}),
                "Failed to validate {:?} with AND operator",
                case
            );
        }
    }
}
