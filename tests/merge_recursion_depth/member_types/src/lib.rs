//! Cross-crate Object types for testing MergedObject recursion depth.
//!
//! Each "chain" defines 3 levels of async resolver nesting:
//!   TopNN -> MidNN -> LeafNN
//!
//! When these types are merged via MergedObject in a separate crate,
//! the compiler must compute the layout of the combined async state machine
//! across crate boundaries, which amplifies monomorphization depth.

use async_graphql::*;

macro_rules! define_leaf {
    ($name:ident, $field:ident) => {
        #[derive(SimpleObject, Default, Clone)]
        pub struct $name {
            pub $field: i32,
        }
    };
}

macro_rules! define_nested {
    ($name:ident, $field:ident, $inner:ty) => {
        #[derive(Default)]
        pub struct $name {
            pub inner: $inner,
        }

        #[Object]
        impl $name {
            async fn $field(&self) -> &$inner {
                &self.inner
            }
        }
    };
}

macro_rules! define_chain {
    ($leaf:ident, $lf:ident, $mid:ident, $mf:ident, $top:ident, $tf:ident) => {
        define_leaf!($leaf, $lf);
        define_nested!($mid, $mf, $leaf);
        define_nested!($top, $tf, $mid);
    };
}

define_chain!(Leaf01, lf01, Mid01, mid01, Top01, top01);
define_chain!(Leaf02, lf02, Mid02, mid02, Top02, top02);
define_chain!(Leaf03, lf03, Mid03, mid03, Top03, top03);
define_chain!(Leaf04, lf04, Mid04, mid04, Top04, top04);
define_chain!(Leaf05, lf05, Mid05, mid05, Top05, top05);
define_chain!(Leaf06, lf06, Mid06, mid06, Top06, top06);
define_chain!(Leaf07, lf07, Mid07, mid07, Top07, top07);
define_chain!(Leaf08, lf08, Mid08, mid08, Top08, top08);
define_chain!(Leaf09, lf09, Mid09, mid09, Top09, top09);
define_chain!(Leaf10, lf10, Mid10, mid10, Top10, top10);
define_chain!(Leaf11, lf11, Mid11, mid11, Top11, top11);
define_chain!(Leaf12, lf12, Mid12, mid12, Top12, top12);
define_chain!(Leaf13, lf13, Mid13, mid13, Top13, top13);
