use crate::registry::SemanticNullability;

mod array;
mod btree_set;
mod hash_set;
#[cfg(feature = "hashbrown")]
mod hashbrown_hash_set;
mod linked_list;
mod slice;
mod vec;
mod vec_deque;

fn wrap_semantic_nullability_in_list(
    semantic_nullability: SemanticNullability,
) -> SemanticNullability {
    if cfg!(feature = "nullable-result") {
        match semantic_nullability {
            SemanticNullability::None => SemanticNullability::None,
            SemanticNullability::OutNonNull => SemanticNullability::InNonNull,
            SemanticNullability::InNonNull => SemanticNullability::InNonNull,
            SemanticNullability::BothNonNull => SemanticNullability::BothNonNull,
        }
    } else {
        semantic_nullability
    }
}
