pub struct InputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<&'static str>,
}

pub struct Field {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub args: Vec<InputValue>,
    pub ty: String,
    pub deprecation: Option<&'static str>,
}

pub struct EnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub deprecation: Option<&'static str>,
}

pub enum Type {
    Scalar {
        name: String,
        description: Option<&'static str>,
    },
    Object {
        name: &'static str,
        description: Option<&'static str>,
        fields: Vec<Field>,
    },
    Interface {
        name: &'static str,
        description: Option<&'static str>,
        fields: Vec<Field>,
        possible_types: Vec<usize>,
    },
    Union {
        name: &'static str,
        description: Option<&'static str>,
        possible_types: Vec<usize>,
    },
    Enum {
        name: &'static str,
        description: Option<&'static str>,
        enum_values: Vec<EnumValue>,
    },
    InputObject {
        name: &'static str,
        description: Option<&'static str>,
        input_fields: Vec<InputValue>,
    },
    List {
        of_type: usize,
    },
    NonNull {
        of_type: usize,
    },
}
