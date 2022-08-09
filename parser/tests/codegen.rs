//! `pest_derive` crate has large dependency tree, and, as a build dependency,
//! it imposes these deps onto our consumers.
//!
//! To avoid that, let's just dump generated code to string into this
//! repository, and add a test that checks that the code is fresh.
