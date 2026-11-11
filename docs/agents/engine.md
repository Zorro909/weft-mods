# Engine architecture

How the engine, the server and mods connect, and where engine features plug in. It applies to work inside `Weft/`. Paths without a crate are under `Weft/crates/engine/src`.

## Map

| Path | Role |
|---|---|
| `Weft/crates/abi` | The C boundary between engine and mods. |
| `Weft/crates/sdk` | The safe API mods use. Base uses exactly this and nothing private. |
| `Weft/crates/engine` | Client, server, networking, UI, registries. |
| `Weft/crates/render`, `Weft/backends/` | Backend-agnostic rendering, and its backends: `vulkan` and `wgpu`. |
| `Weft/crates/noise` | Noise shared by CPU and GPU. |
| `Weft/crates/app` | The binary. |
| `Weft/mods/base` | The base game. `Weft/mods/base/api` holds the types other mods depend on. |
| `Weft/mods/testkit`, `Weft/mods/testmod` | In-game tests, and a mod that extends and replaces base. |

## Shape

- **ABI.** `Weft/crates/abi` is `repr(C)` and `no_std`. `ABI_VERSION` hashes its two files, `lib.rs` and `data.rs`, so it stays those two, and any edit to either (a comment too) makes built mods refuse to load. That is intended: mods are rebuilt with the engine.
- **Client and server.** Single player still runs a server thread. Client and server only see channel endpoints (`net`); a transport bridges them.
- **Registries.** Mods fill them in parallel, then they are frozen into a global (`content()`), with ids assigned by sorted name. Content declares behaviour through builder calls, and the engine runs it (ticks, spreading, liquids: `server/dim.rs`, `server/liquid.rs`).
- **Threads.** Parallel work stays deterministic: results are applied in list order. The pools in `lib.rs` protect frame time: work queued by the thousand goes to `background()` or `meshing()`, and the global rayon pool is for work something waits on.
- **Rendering.** `Weft/crates/render` is plain data (`Cmd`, `*Desc`, id newtypes) and one `trait Backend`. Only crates under `Weft/backends/` talk to a graphics API, and only `Weft/crates/app` names a backend.
- **Noise.** `Weft/crates/noise` is integer-only so CPU and GPU agree. `Weft/crates/engine/assets/shaders/terrain.wgsl` is a line-by-line copy, and testkit's `gpu_terrain` checks they match.

## Where things plug in

- **Mod API** (only after agreeing the change with your user, see `AGENTS.md`): a field on `Host` (or a `*Desc` field plus its `defaults!` entry) in `Weft/crates/abi`, the `unsafe extern "C" fn` in `host.rs`, a safe wrapper in `Weft/crates/sdk`, and the matching guide in `Weft/docs/`.
- **Engine command**: a row in `COMMANDS` (`server/builtin.rs`) or `CLIENT_COMMANDS` (`app.rs`). Gameplay commands belong in a mod (`r.command(..)`).
- **Engine text**: keys in every file under `Weft/crates/engine/assets/lang/`, looked up by key (`say("tp.nowhere", ..)`).
- **Network message**: a variant in a `messages!` block in `net/mod.rs`, handled in `client/receive.rs`, or in `server/actions.rs` with `handle` plus an arm in `allowed` for who may send it (its `_ => true` accepts anything left out).
- **Screen**: `impl Screen`, plus an arm in `screens::named`.
- **Effect a dimension asks of the server**: a `DimEvent` variant, handled in `Server::dim_events`.
