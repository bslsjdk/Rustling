//! The file-backed part of `horizon-mods.txt`, shared by the game and lightweight launchers.
//! Screen state stays in `horizon-client-render::mod_list`.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModSettings {
    /// Content mods the player turned on.
    pub enabled: BTreeSet<String>,
    /// Native mods the next build must leave out.
    pub disabled: BTreeSet<String>,
    /// Mod id → key name → key binding.
    pub keys: BTreeMap<String, BTreeMap<String, String>>,
}

impl ModSettings {
    pub fn parse(text: &str) -> Self {
        let ids = |value: &str| -> BTreeSet<String> {
            text.lines()
                .filter_map(|line| line.split_once(':'))
                .filter(|(_, value_actual)| value_actual.trim() == value)
                .filter_map(|(key, _)| key.trim().strip_prefix("mod."))
                .map(str::to_owned)
                .collect()
        };
        let (enabled, disabled) = (ids("true"), ids("false"));
        let mut keys: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for (key, value) in text.lines().filter_map(|line| line.split_once(':')) {
            // A mod id contains no dots; a key's name may.
            if let Some((id, name)) = key
                .trim()
                .strip_prefix("key.")
                .and_then(|rest| rest.split_once('.'))
            {
                keys.entry(id.to_owned())
                    .or_default()
                    .insert(name.to_owned(), value.trim().to_owned());
            }
        }
        Self {
            enabled,
            disabled,
            keys,
        }
    }

    pub fn serialize(&self) -> String {
        let on = self.enabled.iter().map(|id| format!("mod.{id}:true\n"));
        let off = self.disabled.iter().map(|id| format!("mod.{id}:false\n"));
        let keys = self.keys.iter().flat_map(|(id, keys)| {
            keys.iter()
                .map(move |(name, key)| format!("key.{id}.{name}:{key}\n"))
        });
        on.chain(off).chain(keys).collect()
    }
}
