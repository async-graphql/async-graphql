use async_graphql::validators::{
    Email, InputValueValidator, IntEqual, IntGreaterThan, IntLessThan, IntNonZero, IntRange,
    ListMaxLength, ListMinLength, StringMaxLength, StringMinLength, MAC,
};
use async_graphql::*;

#[tokio::test]
pub async fn test_input_validator_string_min_length() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputMaxLength {
        #[graphql(validator(StringMinLength(length = "6")))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(StringMinLength(length = "6")))] _id: String,
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
                    .into_result()
                    .expect_err(&should_fail_msg),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with StringMinLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with StringMinLength",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_string_max_length() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputMaxLength {
        #[graphql(validator(StringMaxLength(length = "6")))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(StringMaxLength(length = "6")))] _id: String,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with StringMaxLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with StringMaxLength",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_string_email() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputEmail {
        #[graphql(validator(Email))]
        pub email: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(&self, #[graphql(validator(Email))] _email: String) -> bool {
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
                "Invalid value for argument \"email\", invalid email format".to_owned();
            let object_error_msg =
                "Invalid value for argument \"input.email\", invalid email format".to_owned();

            // Testing FieldValidator
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            // Testing ObjectValidator
            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);

            // Field Paramter
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with Email",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with Email",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_string_mac() {
    struct QueryRootWithColon;
    struct QueryRootWithoutColon;

    #[derive(InputObject)]
    struct InputMACWithColon {
        #[graphql(validator(MAC(colon = "true")))]
        pub mac: String,
    }

    #[derive(InputObject)]
    struct InputMACWithoutColon {
        #[graphql(validator(MAC(colon = "false")))]
        pub mac: String,
    }

    #[Object]
    impl QueryRootWithColon {
        async fn field_parameter(
            &self,
            #[graphql(validator(MAC(colon = "true")))] _mac: String,
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
            #[graphql(validator(MAC(colon = "false")))] _mac: String,
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
        let field_error_msg = "Invalid value for argument \"mac\", invalid MAC format".to_owned();
        let object_error_msg =
            "Invalid value for argument \"input.mac\", invalid MAC format".to_owned();

        assert_eq!(
            schema_without_colon
                .execute(&field_query)
                .await
                .into_result()
                .expect_err(&should_fail_msg[..]),
            vec![ServerError {
                message: field_error_msg.clone(),
                locations: vec!(Pos {
                    line: 1,
                    column: 17
                }),
                path: Vec::new(),
                extensions: None,
            }]
        );

        // Testing ObjectValidator
        assert_eq!(
            schema_without_colon
                .execute(&object_query)
                .await
                .into_result()
                .expect_err(&should_fail_msg[..]),
            vec![ServerError {
                message: object_error_msg.clone(),
                locations: vec!(Pos {
                    line: 1,
                    column: 14
                }),
                path: Vec::new(),
                extensions: None,
            }]
        );

        assert_eq!(
            schema_with_colon
                .execute(&field_query)
                .await
                .into_result()
                .expect_err(&should_fail_msg[..]),
            vec![ServerError {
                message: field_error_msg,
                locations: vec!(Pos {
                    line: 1,
                    column: 17
                }),
                path: Vec::new(),
                extensions: None,
            }]
        );

        // Testing ObjectValidator
        assert_eq!(
            schema_with_colon
                .execute(&object_query)
                .await
                .into_result()
                .expect_err(&should_fail_msg[..]),
            vec![ServerError {
                message: object_error_msg,
                locations: vec!(Pos {
                    line: 1,
                    column: 14
                }),
                path: Vec::new(),
                extensions: None,
            }]
        );
    }

    for mac in valid_macs {
        let field_query = format!("{{fieldParameter(mac: \"{}\")}}", mac);
        let object_query = format!("{{inputObject(input: {{mac: \"{}\"}})}}", mac);
        let contains_colon = mac.contains(':');
        let should_fail_msg = format!(
            "MAC validation case {} should have failed, but did not",
            mac
        );
        let field_error_msg = "Invalid value for argument \"mac\", invalid MAC format".to_owned();
        let object_error_msg =
            "Invalid value for argument \"input.mac\", invalid MAC format".to_owned();
        let error_msg = format!("Schema returned error with test_string = {}", mac);

        if contains_colon {
            // Field Paramter
            assert_eq!(
                schema_with_colon
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_with_colon
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_without_colon
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            // Testing ObjectValidator
            assert_eq!(
                schema_without_colon
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            assert_eq!(
                schema_without_colon
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_without_colon
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with MAC",
                mac
            );

            assert_eq!(
                schema_with_colon
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            // Testing ObjectValidator
            assert_eq!(
                schema_with_colon
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_int_range() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputIntRange {
        #[graphql(validator(IntRange(min = "-2", max = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(IntRange(min = "-2", max = "5")))] _id: i32,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with IntRange",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with IntRange",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_int_less_than() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputIntLessThan {
        #[graphql(validator(IntLessThan(value = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(IntLessThan(value = "5")))] _id: i32,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with IntLessThan",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with IntLessThan",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_int_greater_than() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputIntGreaterThan {
        #[graphql(validator(IntGreaterThan(value = "3")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(IntGreaterThan(value = "3")))] _id: i32,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with IntGreaterThan",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with IntGreaterThan",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_int_nonzero() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputIntNonZero {
        #[graphql(validator(IntNonZero))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(&self, #[graphql(validator(IntNonZero))] _id: i32) -> bool {
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with IntNonZero",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with IntNonZero",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_int_equal() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputIntEqual {
        #[graphql(validator(IntEqual(value = "5")))]
        pub id: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(IntEqual(value = "5")))] _id: i32,
        ) -> bool {
            true
        }

        async fn input_object(&self, _input: InputIntEqual) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let equal_to = 5;

    for case in -10i32..10 {
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with IntEqual",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with IntEqual",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_list_max_length() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputListMaxLength {
        #[graphql(validator(ListMaxLength(length = "5")))]
        pub id: Vec<i32>,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(ListMaxLength(length = "5")))] _id: Vec<i32>,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {:?} with ListMaxLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {:?} with ListMaxLength",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_list_min_length() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputListMinLength {
        #[graphql(validator(ListMinLength(length = "4")))]
        pub id: Vec<i32>,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(ListMinLength(length = "4")))] _id: Vec<i32>,
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
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {:?} with ListMinLength",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {:?} with ListMinLength",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_operator_or() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputOrValidator {
        #[graphql(validator(or(Email, MAC(colon = "false"))))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(or(Email, MAC(colon = "false"))))] _id: String,
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

            let field_error_msg =
                "Invalid value for argument \"id\", invalid MAC format".to_owned();
            let object_error_msg =
                "Invalid value for argument \"input.id\", invalid MAC format".to_owned();
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {:?} with OR operator",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {:?} with OR operator",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_operator_and() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputAndValidator {
        #[graphql(validator(and(Email, StringMinLength(length = "14"))))]
        pub email: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(and(Email, StringMinLength(length = "14"))))] _email: String,
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
                "Invalid value for argument \"email\", invalid email format".to_owned()
            } else {
                format!("Invalid value for argument \"email\", the value length is {}, must be greater than or equal to {}", case_length, min_length)
            };

            let object_error_msg = if *should_be_invalid_email {
                "Invalid value for argument \"input.email\", invalid email format".to_owned()
            } else {
                format!("Invalid value for argument \"input.email\", the value length is {}, must be greater than or equal to {}", case_length, min_length)
            };

            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 17
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 14
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test case = {:?}", case);
            assert_eq!(
                schema
                    .execute(&field_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {:?} with AND operator",
                case
            );

            assert_eq!(
                schema
                    .execute(&object_query)
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {:?} with AND operator",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_input_validator_variable() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputMaxLength {
        #[graphql(validator(StringMinLength(length = "6")))]
        pub id: String,
    }

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(StringMinLength(length = "6")))] _id: String,
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
        let mut variables = Variables::default();
        variables.insert(Name::new("id"), Value::String(case.to_string()));

        let field_query = "query($id: String!) {fieldParameter(id: $id)}";
        let object_query = "query($id: String!) {inputObject(input: {id: $id})}";
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
                    .execute(Request::new(field_query).variables(variables.clone()))
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: field_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 37
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );

            assert_eq!(
                schema
                    .execute(Request::new(object_query).variables(variables.clone()))
                    .await
                    .into_result()
                    .expect_err(&should_fail_msg[..]),
                vec![ServerError {
                    message: object_error_msg,
                    locations: vec!(Pos {
                        line: 1,
                        column: 34
                    }),
                    path: Vec::new(),
                    extensions: None,
                }]
            );
        } else {
            let error_msg = format!("Schema returned error with test_string = {}", case);
            assert_eq!(
                schema
                    .execute(Request::new(field_query).variables(variables.clone()))
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"fieldParameter": true}),
                "Failed to validate {} with StringMinLength",
                case
            );

            assert_eq!(
                schema
                    .execute(Request::new(object_query).variables(variables.clone()))
                    .await
                    .into_result()
                    .expect(&error_msg[..])
                    .data,
                value!({"inputObject": true}),
                "Failed to validate {} with StringMinLength",
                case
            );
        }
    }
}

#[tokio::test]
pub async fn test_custom_input_validator_with_extensions() {
    pub struct IntGreaterThanZero;

    impl InputValueValidator for IntGreaterThanZero {
        fn is_valid_with_extensions(&self, value: &Value) -> Result<(), Error> {
            if let Value::Number(n) = value {
                if let Some(n) = n.as_i64() {
                    if n <= 0 {
                        let e = Error::new("Value must be greater than 0")
                            .extend_with(|_, e| e.set("code", 400));
                        return Err(e);
                    }
                }
            }
            Ok(())
        }
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn field_parameter(
            &self,
            #[graphql(validator(IntGreaterThanZero))] _id: i32,
        ) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);

    let case = 0;
    let field_query = format!("{{fieldParameter(id: {})}}", case);
    let should_fail_msg = "IntGreaterThanZero must failed, but not";
    let field_error_msg = "Invalid value for argument \"id\", Value must be greater than 0";

    let mut error_extensions = ErrorExtensionValues::default();
    error_extensions.set("code", 400);

    assert_eq!(
        schema
            .execute(&field_query)
            .await
            .into_result()
            .expect_err(should_fail_msg),
        vec![ServerError {
            message: field_error_msg.into(),
            locations: vec!(Pos {
                line: 1,
                column: 17
            }),
            path: Vec::new(),
            extensions: Some(error_extensions),
        }]
    );
}

#[tokio::test]
pub async fn test_input_validator_list() {
    struct QueryRoot;

    #[derive(InputObject)]
    struct InputEmail {
        #[graphql(validator(list(Email)))]
        pub emails: Vec<String>,
    }

    #[Object]
    impl QueryRoot {
        async fn value(&self, #[graphql(validator(list(Email)))] _emails: Vec<String>) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(
                r#"
                {
                    value(
                        emails: [
                            "a@a.com",
                            "b@abc.com",
                        ]
                    )
                }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({"value": true})
    );

    assert_eq!(
        schema
            .execute(
                r#"
                {
                    value(
                        emails: [
                            "123456",
                        ]
                    )
                }"#
            )
            .await
            .into_result()
            .unwrap_err()[0]
            .message,
        "Invalid value for argument \"emails\", invalid email format"
    );
}
