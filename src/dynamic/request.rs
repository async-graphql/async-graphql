use crate::{Request, dynamic::FieldValue};

/// GraphQL request for dynamic schema.
pub struct DynamicRequest {
    pub(crate) inner: Request,
    pub(crate) root_value: FieldValue<'static>,
}

/// A trait for [`DynamicRequest`]
pub trait DynamicRequestExt {
    /// Specify the root value for the request
    fn root_value(self, value: FieldValue<'static>) -> DynamicRequest;
}

impl<T: Into<Request>> DynamicRequestExt for T {
    fn root_value(self, value: FieldValue<'static>) -> DynamicRequest {
        DynamicRequest {
            inner: self.into(),
            root_value: value,
        }
    }
}

impl<T: Into<Request>> From<T> for DynamicRequest {
    fn from(req: T) -> Self {
        Self {
            inner: req.into(),
            root_value: FieldValue::NULL,
        }
    }
}
