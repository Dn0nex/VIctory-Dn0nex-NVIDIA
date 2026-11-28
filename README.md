# VIctory (Dn0nex build)

A build of [VIctory](https://github.com/Marvisak/VIctory), the compatibility layer that runs an
Xbox Series title natively on Windows (Direct3D 12) from files the user supplies. It runs on
**any PC from 16 GB of RAM up** (more RAM, more cores and a stronger card only make it faster),
and it was tuned so that even an ordinary PC (16 GB, an 8 GB card, 6 cores) can play it, without
lowering the picture: textures, shaders, resolution and ray tracing stay as they are.

**To play: put `VIctory-Dn0nex.exe` in the folder of your game dump and double-click it.**

> **This repository and its releases contain no game files, code or assets.**
> You need your own dump of the game. This project is not affiliated with or endorsed by
> the game's developer or publisher.

*Resumen en español al final.*

## Where it stands

It runs, but it is **not smooth yet**. Tested on one mid-range PC (6-core CPU, 8 GB NVIDIA card,
16 GB RAM, 720p):

| | |
|---|---|
| Menus, videos, loading screens | 30 fps |
| Walking indoors (the prison) | 8–10 fps (it was ~6.5 before this build's changes) |
| Busy views | 6–7 fps |
| Memory | stays inside 16 GB RAM / 8 GB VRAM |

Faster PCs (more cores, 32 or 64 GB of RAM, a stronger card) do better: `VIctory-Dn0nex.exe` detects
the number of CPU threads and the amount of RAM and uses more threads and memory when there are.

Known problems (from upstream and this build):

- The first run with a new driver or build stutters while shaders and pipelines are made; it
  gets better on later runs (they are cached).
- Textures of what just came into view can appear late while the game streams.
- Some flicker of trees, plants and far buildings outdoors.
- With 16 GB the PC is short of memory: close the browser and other programs while playing.

## What is different in this build

- **No more whole-buffer copies of the game's big blocks.** Large memory blocks the game asks
  for are now allocated with Windows' write watch, so only the pages it changed are sent to the
  card. One 21 MB buffer used to be copied whole about twice a frame (~42 MB, 10–14 ms); now
  ~0.4 MB. Walking went from ~6.5 to ~8–10 fps.
- **Spare memory pools are trimmed only when idle**, instead of being freed and re-made every
  frame under memory pressure.
- **Settings built in, chosen for the PC it runs on**: windows of game memory kept on the card,
  finer windows, card-side steps instead of waits; fewer threads on 6-core CPUs, more on 8-core
  ones, more memory used in place with 32 GB or more.
- **Double-click start**: `VIctory-Dn0nex.exe` with no arguments finds the game in its own folder and
  starts it, no launcher needed.

Everything else (the loader, the runtime, the D3D12 back end, the shader recompiler, the
threaded translation, landings and card steps) comes from the projects in the credits.

## Requirements

- Windows 10 or 11, 64-bit
- A CPU with **AVX2** (Intel 4th gen / AMD Ryzen or newer); 6 cores or more
- A Direct3D 12 card with **shader model 6.6 and mesh shaders** (NVIDIA RTX 20 series or newer,
  AMD RX 6000 or newer, Intel Arc), 8 GB of VRAM, an up-to-date driver
- **16 GB of RAM or more** (32 or 64 GB run better)
- An SSD for the game files
- **Your own complete dump of the game** (the folder with `game_scarlett_master.exe`)

## How to run it

1. Download `VIctory-Dn0nex.exe` from [Releases](../../releases/latest).
2. Put it in the folder of your game dump, next to `game_scarlett_master.exe`.
3. Double-click `VIctory-Dn0nex.exe`.

That is all. The first start takes a while (shaders are prepared and cached in
`%LOCALAPPDATA%\victory\cache`); keep the console window open while you play. The log of each
run goes to `victory-run.log` next to `VIctory-Dn0nex.exe`.

Windows SmartScreen or an antivirus may warn about `VIctory-Dn0nex.exe`: it is not signed, and it
works the way a debugger does (it watches and protects memory pages of the game). Choose
"More info" → "Run anyway".

Advanced: every `VICTORY_...` switch can be set before starting it to override the built-in
value; `VICTORY_GAME_DIR` points it at a dump in another folder. `Play-VIctory.cmd` in this
repository does the same as a double-click, for those who prefer a script.

## Building from source

Install the pinned Rust toolchain (`rust-toolchain.toml`) and either the MSVC build tools with a
Windows SDK, or llvm-mingw for the `x86_64-pc-windows-gnullvm` target, then:

```bat
cargo build --release --locked -p victory
```

## Credits

This is a tree of other people's work with a little more on top.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its contributors: the
  loader, the runtime services, the Direct3D 12 back end and the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)** and the people whose pull
  requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes, cube gradients,
  sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**: the ambient
  light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over every view of
  a page (with Spexivus, joroyt and tightfall), and Joshua Odom's reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **VIctory-7900XTX** (avatarmanz) and **codec_d**: the threaded submission, relay, card steps
  and the NVIDIA residency launcher this build started from.
- This build's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory (see [LICENSE](LICENSE)). If you share the `.exe`, the GPL asks you
to share its source code too. The BCPack format tables in `crates/victory-formats` are not
covered by the licence (see `docs/asset-policy.md` in the source).

---

## Resumen en español

Versión de VIctory (capa que ejecuta un juego de Xbox Series en Windows con Direct3D 12) que
funciona **en cualquier PC desde 16 GB de RAM** (con 32 o 64 GB, más núcleos o mejor gráfica va
más rápido), ajustada para que incluso un PC normal (16 GB, gráfica de 8 GB, 6 núcleos) pueda
jugarlo, sin bajar la calidad gráfica (texturas, shaders, resolución y ray tracing intactos).

- **No incluye nada del juego**: necesitas tu propio volcado (la carpeta con
  `game_scarlett_master.exe`).
- **Rendimiento actual**: menús a 30 fps; caminando por interiores 8–10 fps (antes ~6,5); en
  zonas cargadas 6–7 fps. Todavía no va fluido. En PCs más potentes debería ir mejor.
- **Requisitos**: Windows 10/11 de 64 bits, CPU con AVX2, gráfica DirectX 12 con shader model
  6.6 y mesh shaders (NVIDIA RTX 20 o superior, AMD RX 6000 o superior, Intel Arc), 16 GB de RAM
  o más y un SSD.
- **Uso**: descarga `VIctory-Dn0nex.exe` de Releases, ponlo en la carpeta de tu volcado (junto a
  `game_scarlett_master.exe`) y ábrelo con doble clic. Nada más. La primera vez tarda porque
  prepara la caché de shaders; deja abierta la ventana de consola mientras juegas.
- Windows o el antivirus pueden avisar porque el `.exe` no está firmado: "Más información" →
  "Ejecutar de todas formas".
