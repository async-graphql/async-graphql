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
    Scalar,
    Object {
        fields: Vec<Field>,
    },
    Interface {
        fields: Vec<Field>,
        possible_types: Vec<usize>,
    },
    Union {
        possible_types: Vec<usize>,
    },
    Enum {
        enum_values: Vec<EnumValue>,
    },
    InputObject {
        input_fields: Vec<InputValue>,
    },
    List {
        of_type: usize,
    },
    NonNull {
        of_type: usize,
    },
}
