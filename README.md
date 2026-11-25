# VIctory fork (work in progress)

A work-in-progress fork based on an earlier version of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that runs an Xbox Series title natively on Windows from files the user supplies. This fork builds on upstream `main` with shader translation fixes, 4 GB VRAM adaptations, and deep CPU↔GPU memory telemetry.

**This repository contains no game files, code or assets.** You need your own copy of the game. It is not affiliated with or endorsed by the game's developer or publisher.

## Credits

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its contributors is the base of everything here: the loader, the runtime services, the Direct3D 12 back end and the shader recompiler. Licensed GPL-3.0-or-later; so is this fork.
- **[XCL](https://github.com/ivall/xcl)** by ivall and its contributors (GPL-2.0-only) was used as a reference for some title-specific behaviors.

## Status of this fork

**Not playable yet: ~1.3 to 2.0 frames per second in-world at `1920x1080`.**
Tested on: Windows 11, **NVIDIA RTX 3050 (4 GB VRAM)**, **16 GB RAM**, using the Xbox Series X profile (`-DisplayMode 1920x1080`).

| Stage | State in this fork |
|-------|--------------------|
| Boot, menus, start screen | Works |
| Opening cutscene & saves | Works |
| First gameplay scene (interior & exterior) | Renders characters, interior walls/props, doors, ceiling, sky and sunlit exterior; some distant/exterior streaming textures remain queued and ray-traced lighting/shadows are disabled |
| Frame rate in the world (`1920x1080`) | **1.3–2.0 fps** (`543–741 ms/frame`) |
| Stability | Stable in normal runs on 4 GB VRAM + 16 GB RAM when using `VICTORY_CARD_BUFFERS_MB=384` and mip clamping |

### Measured performance & memory breakdown (`1920x1080`, RTX 3050 4 GB)

From the built-in 5-second diagnostic telemetry (`victory-gpu`):

- **GPU execution (`on the GPU`):** `82.3–147.6 ms/frame` (`3,152–6,964 draws/frame`).
- **CPU `preparing`:** `475.0–552.2 ms/frame`
  - `watch` (`GetWriteWatch` + `guard` on view memory): `~275.4 ms/frame` scanning `~42.5 GiB/frame` across `~26,900` calls (`guard`: `~197.0 ms` over `~35.4 GiB/frame` in `~10,166` calls).
  - `copy_mapped` (`ReadProcessMemory` from title RAM to `D3D12_HEAP_TYPE_CUSTOM` `WRITE_BACK` `POOL_L0`, single-threaded): `~346.6 ms/frame` across `~85,553` calls (`~522.2 MiB/frame`).
- **Relay (`copying back`):** `565.2–586.8 ms/frame` (`~69.8–83.0 MB/frame` on the background relay thread).
  - `relay piece audit`: `~464–536` pieces fetched (`~28.5–32.7 MB`), `~462–532` written back to title memory (`~20.2–21.1 MB`), and **`~99%` (`459.5–528.0` pieces)** overwritten again by the GPU on the next frame.
- **Live memory audit (`memory audit (live)`):**
  - DXGI local: `~4,894 MiB` (`733 MiB` live filled textures, `2,938 MiB` render targets / depth / UAV, `101 MiB` depth twins, `383 MiB` card buffers).
  - DXGI shared: `~5,987 MiB` (`4,700 MiB` `shadow_shared_L0`, `192 MiB` list scratch chunks, `44 MiB` texture staging).
  - Title in-place memory: `~360–388 MiB` (`77–83` windows).

### What this fork changed

Shader recompiler (`victory-shader`):
- Fixed compute shader `f133e35efed2a48a` lowering (`local && !local_any`).
- Fixed `2da4beb386509b77` CFG/lane-mask lowering (`only_edge` discarding `s_cbranch_execz` taken edges + pushing `lanes.reach[block]`).
- Fixed `02384a60fda6f2d0` Wave32 fallback for `group > 32` with LDS (`LOCAL_WAVES && LOCAL_ANY`).
- Added selective `with_wave_wide` whitelist (`369cbe906ea72764`, `2cc4c197ee29e291`, `5dc3b25986622949`, `6f225b641c5b0708`).

GPU backend & diagnostics (`victory-gpu`):
- Implemented `DepthBoundsTestEnable` (`D3D12_DEPTH_STENCIL_DESC1`) and `OMSetDepthBounds` (`ID3D12GraphicsCommandList1`, clamped `0.0..=1.0`).
- Added `VICTORY_MIP_CLAMP` to skip mip 0 on sampled-only 2D `BC1–BC7` textures `>= 1024` for 4 GB VRAM cards.
- Replaced unmapped 2D texture SRVs with `opaque_dummy_2d()` (`1x1` `R8G8B8A8_UNORM`).
- Made bindless texture staging and descriptor-mirror sweeps configurable (`VICTORY_TEXTURE_PER_FRAME_MB`, `VICTORY_TEXTURE_SPAN_MS`, `VICTORY_MIRROR_FULL_PASS_EVERY`).
- Added always-on release diagnostics: `relay piece audit per frame`, `memory audit (live)` (`KINDS`, `HOST_TEXTURES`, `depth_twins`, `shadow_shared_L0`, `card_buffers`, `inplace`, `staging`, `readback`), and `watch & copy_mapped per frame`.

### Known limitations

- **Low frame rate (`1.3–2.0 fps`):** dominated by CPU↔GPU shadow coherence (`watch`, single-threaded `ReadProcessMemory` in `copy_mapped`, and `relay` readbacks) rather than GPU shading or shader translation.
- **Missing lighting/passes:** `172–332` draws/frame are dropped without a host pipeline (ray-traced lighting/BVH passes disabled), and `~1,195–3,038` unaligned buffer views remain null due to D3D12 alignment rules.
- **Texture streaming backlog:** on 4 GB VRAM, some distant/exterior textures remain queued (`waiting` / `unmapped`).

Everything below this line is upstream VIctory's own README, unchanged; its status section describes upstream, not this fork.

---

# VIctory

Runs the title (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; VIctory supplies
the platform underneath it and converts the title's data, ahead of time where
it can and on first use where it must (shaders). See
[docs/architecture.md](docs/architecture.md).

## Status (2026-11-23)

**Not playable yet.** The title boots, runs its menus at its own 30 fps and
loads the world, where rendering fails.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | reached and runs with indirect draws; its title card renders, the scene after it is still dark; 0.5-1.8 s per frame |

Tested on one machine: Windows 11, AMD RX 6700 XT.

> **Warning.** Entering the world on a real GPU can currently hang it, and
> Windows then resets the display driver (every monitor goes black for a
> moment). To look further, use a diagnostics build with
> `VICTORY_D3D12_WARP=1`, which renders on the software adapter.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Most of the scene is still missing (untranslated shaders drop draws); frames take seconds; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title:

```powershell
cargo build --release -p victory
target\release\victory.exe run <game dir>\<the title's exe> --execute
```

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_DUMP_TARGETS=1` | with the two above, saves every render target of those frames |
| `VICTORY_D3D12_DEBUG=1` | host debug layer |
| `VICTORY_D3D12_GBV=1` | with the above, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_PIPELINE_WAIT_MS=24` | reduce the per-frame wait for background pipeline creation from the default of 500 ms. A draw can be omitted until its pipeline is ready |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
