macro_rules! try_query_result {
    ($res:expr) => {
        match $res {
            Ok(resp) => resp,
            Err(err) => return err.into(),
        }
    };
}
