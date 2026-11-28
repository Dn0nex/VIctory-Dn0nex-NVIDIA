//! Double-click start: the executable (`victory.exe`, `VIctory-Dn0nex.exe`) opened with no arguments, from the
//! folder of the user's own dump of the title (or with `VICTORY_GAME_DIR`
//! set), starts the title with the settings `Play-VIctory.cmd` gives.
//!
//! It runs itself again as `victory run <title> --console series-s
//! --execute`, with those settings in the environment (each only where the
//! user has not set it) and what is logged going to `victory-run.log` beside
//! the executable, as the launcher did.

use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

/// The title's executable in a dump.
const TITLE: &str = "game_scarlett_master.exe";

/// Whether the executable was opened with nothing after its name.
pub(crate) fn wanted() -> bool {
    std::env::args_os().len() == 1
}

/// The settings of `Play-VIctory.cmd`, in its order: the first one of a
/// name wins, and one the user set wins over all.
fn settings(threads: usize, ram_gb: u64) -> Vec<(&'static str, &'static str)> {
    let mut all = vec![
        ("VICTORY_DISPLAY_MODE", "1280x720"),
        ("VICTORY_WINDOW_MAX", "1280x720"),
        ("VICTORY_TEXTURE_ROOM_PERCENT", "82"),
        ("VICTORY_CARD_ROOM_FOR_BUFFERS", "1"),
        ("VICTORY_CARD_ROOM_FOR_TARGETS_MB", "2048"),
        ("VICTORY_RIPE_BLOCKS", "2"),
        ("VICTORY_LEVELS_MIN_KB", "512"),
        ("VICTORY_ASYNC_SUBMIT", "1"),
        ("VICTORY_WIDE_MERGE", "1"),
        ("VICTORY_WIDE_SPIN", "0"),
        ("VICTORY_AMBIENT_AIR", "1.5"),
        ("VICTORY_TEXTURE_NO_WAIT", "1"),
        ("VICTORY_RELAY", "2"),
        ("VICTORY_WIDE_BACK", "1"),
        ("VICTORY_CARD_REHASH_RUNS", "1"),
        ("VICTORY_JOIN_COPIES", "1"),
        ("VICTORY_INDIRECT_WINDOW", "1"),
        ("VICTORY_RELAY_COPY", "1"),
        ("VICTORY_CARD_STEPS", "2"),
        ("VICTORY_INDIRECT_BATCH", "2"),
        ("VICTORY_MAP_THREADS", "1"),
    ];
    // 8 cores / 16 threads: more list, landing, copy and view threads, and
    // small buffers into the card too.
    if threads >= 14 {
        all.extend([
            ("VICTORY_THIN_THREADS", "6"),
            ("VICTORY_LAND_THREADS", "6"),
            ("VICTORY_WIDE_THREADS", "6"),
            ("VICTORY_VIEW_THREADS", "6"),
            ("VICTORY_CARD_BUFFERS_MIN_KB", "64"),
            ("VICTORY_CARD_BUFFERS_MB", "1536"),
        ]);
    }
    // 32 GB of RAM: room for more title memory used in place.
    if ram_gb >= 28 {
        all.push(("VICTORY_INPLACE_MB", "4096"));
    }
    all.extend([
        ("VICTORY_CARD_BUFFERS_MB", "1024"),
        ("VICTORY_CARD_WINDOWS_MB", "1024"),
        ("VICTORY_CARD_AWAKE", "0"),
        ("VICTORY_SHOW_AFTER", "1"),
        ("VICTORY_AMBIENT", "0.8"),
        ("VICTORY_THIN", "1"),
        ("VICTORY_JOIN_PARTS", "8"),
        ("VICTORY_UPLOAD_AHEAD", "1"),
        ("VICTORY_LEVELS_ON_DEMAND", "1"),
        ("VICTORY_CARD_BUFFERS_MIN_KB", "256"),
        ("VICTORY_PLACED_ORDER", "1"),
        ("VICTORY_PIPELINE_BOOK", "1"),
        ("VICTORY_LAND_SKIP", "1"),
        ("VICTORY_THIN_THREADS", "4"),
        ("VICTORY_WIDE_THREADS", "2"),
        ("VICTORY_LAND_THREADS", "3"),
        ("VICTORY_VIEW_THREADS", "2"),
        ("VICTORY_INPLACE", "1"),
        ("VICTORY_INPLACE_MB", "2048"),
        ("VICTORY_WINDOW_KB", "256"),
        ("VICTORY_WINDOW_TAILS", "1"),
        ("VICTORY_FAST_STATS", "0"),
    ]);
    let mut seen = std::collections::HashSet::new();
    all.retain(|(name, _)| seen.insert(*name));
    all
}

/// The log filter of the launcher: warnings, and the once-a-second lines a
/// run is measured by.
const LOG: &str = "warn,victory_gpu::host=info,victory_gpu::shadow=info,victory_gpu::copystat=info,\
victory_hle::runtime=info,victory_gpu::land=info,victory_gpu::descriptor=info,victory_gpu::descriptor::host=warn,\
victory_gpu::list::thin=info,victory_gpu::queue::behind=info";

/// The PC's memory in GB, rounded as the launcher did (16 if unknown).
fn ram_gb() -> u64 {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut status = MEMORYSTATUSEX {
            dwLength: size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        // SAFETY: a struct of the size it says.
        if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
            return (status.ullTotalPhys as f64 / (1u64 << 30) as f64).round() as u64;
        }
    }
    16
}

/// The dump: `VICTORY_GAME_DIR`, else the executable's folder, the current
/// one, or a folder `game` beside the executable.
fn find_title(here: &Path) -> Option<PathBuf> {
    let mut places: Vec<PathBuf> = Vec::new();
    if let Some(dir) = std::env::var_os("VICTORY_GAME_DIR") {
        places.push(dir.into());
    }
    places.push(here.to_path_buf());
    if let Ok(dir) = std::env::current_dir() {
        places.push(dir);
    }
    places.push(here.join("game"));
    places.into_iter().map(|dir| dir.join(TITLE)).find(|title| title.is_file())
}

/// Keeps the console window open until Enter, so what was said can be read.
fn wait_for_enter() {
    print!("\nPress Enter to close this window.");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().lock().read_line(&mut String::new());
}

pub(crate) fn run() -> Result<()> {
    let me = std::env::current_exe().context("where this executable is")?;
    let here = me.parent().map(Path::to_path_buf).unwrap_or_default();
    let Some(title) = find_title(&here) else {
        let name = me.file_name().map_or_else(|| "this program".into(), |n| n.to_string_lossy().into_owned());
        println!("VIctory: {TITLE} was not found.");
        println!();
        println!("Put {name} in the folder of your own dump of the game (the folder that");
        println!("holds {TITLE}) and open it again, or set VICTORY_GAME_DIR to that folder.");
        wait_for_enter();
        std::process::exit(2);
    };
    let threads = std::thread::available_parallelism().map_or(12, |n| n.get());
    let ram = ram_gb();
    let mut child = Command::new(&me);
    child.arg("run").arg(&title).args(["--console", "series-s", "--execute"]);
    for (name, value) in settings(threads, ram) {
        if std::env::var_os(name).is_none() {
            child.env(name, value);
        }
    }
    if std::env::var_os("VICTORY_LOG").is_none() && std::env::var_os("RUST_LOG").is_none() {
        child.env("VICTORY_LOG", LOG);
    }
    // The log beside the executable (the one before kept as .previous); where
    // that folder cannot be written, the console.
    let log: PathBuf = std::env::var_os("VICTORY_RUN_LOG").map_or_else(|| here.join("victory-run.log"), PathBuf::from);
    let to_file = log.as_os_str() != "0" && {
        let mut previous = OsString::from(log.as_os_str());
        previous.push(".previous");
        let _ = std::fs::rename(&log, previous);
        match std::fs::File::create(&log) {
            Ok(file) => {
                child.stderr(Stdio::from(file));
                true
            }
            Err(_) => false,
        }
    };
    println!("VIctory: starting {} ({threads} CPU threads, {ram} GB RAM).", title.display());
    if to_file {
        println!("The log goes to {}.", log.display());
    }
    println!("The first start takes a while: shaders are prepared and cached. Keep this window open.");
    let status = child.status().context("starting the title")?;
    if !status.success() {
        println!("\nVIctory ended with {status}.");
        if to_file {
            println!("What happened is in {}.", log.display());
        }
        wait_for_enter();
    }
    std::process::exit(status.code().unwrap_or(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(all: &[(&str, &'static str)], name: &str) -> &'static str {
        all.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).unwrap()
    }

    #[test]
    fn the_launchers_order_decides() {
        let small = settings(12, 16);
        assert_eq!(value(&small, "VICTORY_THIN_THREADS"), "4");
        assert_eq!(value(&small, "VICTORY_CARD_BUFFERS_MB"), "1024");
        assert_eq!(value(&small, "VICTORY_INPLACE_MB"), "2048");
        let big = settings(16, 32);
        assert_eq!(value(&big, "VICTORY_THIN_THREADS"), "6");
        assert_eq!(value(&big, "VICTORY_CARD_BUFFERS_MB"), "1536");
        assert_eq!(value(&big, "VICTORY_CARD_BUFFERS_MIN_KB"), "64");
        assert_eq!(value(&big, "VICTORY_INPLACE_MB"), "4096");
        let names: std::collections::HashSet<_> = big.iter().map(|(n, _)| *n).collect();
        assert_eq!(names.len(), big.len());
    }
}
