//! The three standard uses of a model.

use std::sync::LazyLock;

use nyaaai_contract::shapes::model::{Ability, ModelUse};

/// Needs nothing of a model; leaves out hidden families.
pub static CHAT: LazyLock<ModelUse> = LazyLock::new(|| ModelUse {
    name: "chat".to_owned(),
    requires: Vec::new(),
    hides: true,
});

/// Needs the schema ability.
pub static COMPACTION: LazyLock<ModelUse> = LazyLock::new(|| ModelUse {
    name: "compaction".to_owned(),
    requires: vec![Ability::Schema],
    hides: false,
});

pub static ANALYSIS: LazyLock<ModelUse> = LazyLock::new(|| ModelUse {
    name: "analysis".to_owned(),
    requires: vec![Ability::Schema],
    hides: false,
});

/// In the order chat, compaction, analysis.
pub static STANDARD_USES: LazyLock<[ModelUse; 3]> =
    LazyLock::new(|| [CHAT.clone(), COMPACTION.clone(), ANALYSIS.clone()]);
