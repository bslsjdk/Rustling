// native_mods hook: the mod's row in Options > Mods.
fn tlm_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: crate::game::tlm_runtime::ID.into(),
        title: format!("Touhou Little Maid {}", crate::game::tlm_runtime::VERSION),
        description: crate::game::tlm_runtime::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
