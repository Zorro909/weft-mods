# weft-mods

Mods for **Weft** (submodule `Weft/`): a Rust/Vulkan voxel sandbox engine. The game itself is a mod (`mods/base`) built only on the public SDK. Mods are `cdylib` crates behind a C ABI, loaded from `run/mods/<id>/` or zips.

Mods must be built against the exact submodule commit (ABI is hash-checked): `weft_sdk = { path = "../Weft/crates/sdk" }` (adjust depth).

## Where to look (all under `Weft/`)

| What | Where |
|---|---|
| Modding cheat sheet (start here) | `docs/modding.md` |
| Topic guides | `docs/{worldgen,machines,models,ui,tooltips,textures,ctm,shaders,resourcepacks}.md` |
| Mod template (every registration kind, commented) | `examples/examplemod/` |
| SDK API (what mods call) | `crates/sdk/src/` — `content.rs` Registry + block/item builders, `game.rs` World, inventories, entities, menus, `extend.rs` events/interceptors, `client.rs`/`ui.rs` client side, `terrain.rs` worldgen |
| Base game: real-world usage | `mods/base/src/` |
| Base game typed API (events, components, tabs) | `mods/base/api/src/lib.rs` (crate `base_api`) |
| Extending/replacing base without engine changes | `mods/testmod/` |
| Automated tests (`weft --test all`) | `mods/testkit/` |
| ABI boundary | `crates/abi/src/lib.rs` |
| Engine internals (only to understand behavior) | `crates/engine/src/` |
