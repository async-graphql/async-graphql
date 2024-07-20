use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Deserializer, Serialize};

/// Extensions of a query.
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Extensions(pub HashMap<String, crate::ConstValue>);

impl<'de> Deserialize<'de> for Extensions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(
            <Option<HashMap<_, _>>>::deserialize(deserializer)?.unwrap_or_default(),
        ))
    }
}

impl Deref for Extensions {
    type Target = HashMap<String, crate::ConstValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Extensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
