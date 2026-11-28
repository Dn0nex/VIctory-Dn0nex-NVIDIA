//! What the card spends its time on, by the card's own clock, in every build
//! (`VICTORY_FAST_STATS=1`; off: nothing is stamped).
//!
//! "The card's clock" line counts the time something of ours was queued,
//! which is not the time the card worked: a queue the card cannot get on
//! with (a wait for memory over the bus, a bubble behind our CPU) counts as
//! busy there. Here timestamps around the parts of each submission say how
//! long each took on the card: the title's lists, the steps of ours done on
//! the card, the sends of copies in the card's memory and the hashes after
//! them, the compares of what shaders wrote, the fetches of what they wrote.
//! What the parts leave of the card's time between the first stamp and the
//! last is the card standing (or work of ours not stamped: tile mappings,
//! indirect batches, the keep-awake lists).
//!
//! Stamps are ends of queries in one query heap, each resolved at once into
//! its own word of a buffer the CPU reads; a word is zeroed when its query
//! is handed out, so a word not zero is a stamp the card has written. Some
//! stamps go into lists of ours as they are recorded; those between the
//! title's lists are lists made ahead, one per query, used in turn.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use windows::core::Interface;

use super::*;

/// Queries stamped into lists as they are recorded, used in turn.
const IN_LIST: u32 = 8192;
/// Queries of the lists made ahead, used in turn.
const MADE: u32 = 1024;

/// The parts of the card's time.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The title's lists (with the state preludes ahead of them).
    Title = 0,
    /// Steps of ours done on the card (`card_steps`).
    Steps = 1,
    /// Sends of the bytes of copies in the card's memory.
    Sends = 2,
    /// Hashes of the pieces just sent.
    Hashes = 3,
    /// Compares of what shaders wrote, after the title's lists.
    Compares = 4,
    /// Fetches of what shaders wrote, on the drawing queue.
    Fetches = 5,
}

const PARTS: usize = 6;

struct Stamps {
    heap: ID3D12QueryHeap,
    /// The words the queries resolve to, mapped.
    words: usize,
    /// The lists made ahead, each ending query `IN_LIST + i`.
    made: Vec<ID3D12CommandList>,
    _kept: (ID3D12CommandAllocator, ID3D12Resource),
    /// Card ticks per second.
    frequency: u64,
}

// SAFETY: free-threaded D3D12 objects, and a mapping of words of our own.
unsafe impl Send for Stamps {}
// SAFETY: as above.
unsafe impl Sync for Stamps {}

static STAMPS: OnceLock<Option<Stamps>> = OnceLock::new();
static NEXT_IN_LIST: AtomicUsize = AtomicUsize::new(0);
static NEXT_MADE: AtomicUsize = AtomicUsize::new(0);
/// Stamps handed out so far, in lists and of the lists made ahead: how old a
/// part waiting for its stamps is.
static HANDED: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

/// A part waiting for the card: its two queries and when it was handed out.
struct Waiting {
    part: Part,
    from: u32,
    to: u32,
    handed: [u64; 2],
}

static WAITING: Mutex<Vec<Waiting>> = Mutex::new(Vec::new());

/// Whether stamps are taken (`VICTORY_FAST_STATS=1`).
pub(super) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_FAST_STATS").is_some_and(|v| v == "1"))
}

fn stamps() -> Option<&'static Stamps> {
    if !on() {
        return None;
    }
    STAMPS
        .get_or_init(|| {
            let host = host()?;
            match host.make_part_stamps() {
                Ok(stamps) => Some(stamps),
                Err(e) => {
                    tracing::warn!(error = %e, "no stamps of the card's time (VICTORY_FAST_STATS)");
                    None
                }
            }
        })
        .as_ref()
}

impl Stamps {
    /// Zeroes the word of `query` before it is used again.
    fn clear(&self, query: u32) {
        // SAFETY: the mapping holds a word per query; the card wrote this one
        // thousands of stamps ago.
        unsafe { ((self.words as *mut u64).add(query as usize)).write_volatile(0) };
        HANDED[usize::from(query >= IN_LIST)].fetch_add(1, Ordering::Relaxed);
    }

    fn word(&self, query: u32) -> u64 {
        // SAFETY: as above; read only.
        unsafe { ((self.words as *const u64).add(query as usize)).read_volatile() }
    }
}

/// A stamp in a list of ours being recorded: the query it ends.
#[derive(Clone, Copy)]
pub(super) struct Stamp(u32);

/// Stamps `list` (open, of the drawing queue) here.
pub(super) fn stamp_in(list: &ID3D12GraphicsCommandList) -> Option<Stamp> {
    let stamps = stamps()?;
    let query = (NEXT_IN_LIST.fetch_add(1, Ordering::Relaxed) % IN_LIST as usize) as u32;
    stamps.clear(query);
    // SAFETY: an open list; the query and the word lie in the heap and buffer.
    unsafe {
        list.EndQuery(&stamps.heap, D3D12_QUERY_TYPE_TIMESTAMP, query);
        list.ResolveQueryData(&stamps.heap, D3D12_QUERY_TYPE_TIMESTAMP, query, 1, &stamps._kept.1, u64::from(query) * 8);
    }
    Some(Stamp(query))
}

/// A list made ahead that stamps where it is put among a submission's lists.
pub(super) fn stamp_list() -> Option<(ID3D12CommandList, Stamp)> {
    let stamps = stamps()?;
    let i = (NEXT_MADE.fetch_add(1, Ordering::Relaxed) % MADE as usize) as u32;
    stamps.clear(IN_LIST + i);
    Some((stamps.made[i as usize].clone(), Stamp(IN_LIST + i)))
}

/// `part` ran on the card from `from` to `to`.
pub(super) fn part(part: Part, from: Option<Stamp>, to: Option<Stamp>) {
    let (Some(from), Some(to)) = (from, to) else {
        return;
    };
    let handed = HANDED.each_ref().map(|h| h.load(Ordering::Relaxed));
    WAITING.lock().unwrap_or_else(|e| e.into_inner()).push(Waiting { part, from: from.0, to: to.0, handed });
}

/// Since the last line: card ticks per part, the parts' first and last tick.
static SUMS: Mutex<([u64; PARTS], u64, u64)> = Mutex::new(([0; PARTS], u64::MAX, 0));

/// Takes the parts the card has run.
fn gather() {
    let Some(stamps) = stamps() else {
        return;
    };
    let handed = HANDED.each_ref().map(|h| h.load(Ordering::Relaxed));
    let mut waiting = WAITING.lock().unwrap_or_else(|e| e.into_inner());
    let mut sums = SUMS.lock().unwrap_or_else(|e| e.into_inner());
    waiting.retain(|w| {
        // A query handed out again since (the card far behind): given up.
        if handed[0].saturating_sub(w.handed[0]) > u64::from(IN_LIST) / 2 || handed[1].saturating_sub(w.handed[1]) > u64::from(MADE) / 2 {
            return false;
        }
        let (from, to) = (stamps.word(w.from), stamps.word(w.to));
        if from == 0 || to == 0 {
            return true;
        }
        if to >= from {
            sums.0[w.part as usize] += to - from;
            sums.1 = sums.1.min(from);
            sums.2 = sums.2.max(to);
        }
        false
    });
}

/// For the frame-rate line: the card's time per frame by part, and the page
/// faults of the process per frame.
pub(super) fn line(frames: f64) {
    static FAULTS: AtomicU64 = AtomicU64::new(0);
    let faults = page_faults();
    let was = FAULTS.swap(faults, Ordering::Relaxed);
    let faults = if was == 0 { 0.0 } else { faults.saturating_sub(was) as f64 / frames };
    let Some(stamps) = stamps() else {
        return;
    };
    gather();
    let (parts, first, last) = std::mem::replace(&mut *SUMS.lock().unwrap_or_else(|e| e.into_inner()), ([0; PARTS], u64::MAX, 0));
    let ms = |ticks: u64| ticks as f64 * 1000.0 / stamps.frequency as f64 / frames;
    let [title, steps, sends, hashes, compares, fetches] = parts.map(ms);
    let sum = title + steps + sends + hashes + compares + fetches;
    let span = if last > first { ms(last - first) } else { 0.0 };
    tracing::info!(
        "the card's time per frame by its own clock (VICTORY_FAST_STATS): the title's lists {title:.1} ms; ours: steps on the card \
         {steps:.1}, sends of copies in the card's memory {sends:.1}, hashes of what was sent {hashes:.1}, compares of what shaders \
         wrote {compares:.1}, fetches of it {fetches:.1}; {sum:.1} ms stamped of {span:.1} ms from the first stamp to the last \
         ({:.1} ms the card stood or ran work not stamped); page faults of the process {faults:.0} a frame",
        (span - sum).max(0.0)
    );
}

/// The process's page faults since it started (soft and hard).
fn page_faults() -> u64 {
    use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut c = PROCESS_MEMORY_COUNTERS { cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32, ..Default::default() };
    // SAFETY: our own process and a sized structure.
    unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    u64::from(c.PageFaultCount)
}

impl Host {
    fn make_part_stamps(&self) -> windows::core::Result<Stamps> {
        let count = IN_LIST + MADE;
        let Some((buffer, words)) = self.shared_buffer(u64::from(count) * 8) else {
            return Err(windows::core::Error::from(windows::Win32::Foundation::E_OUTOFMEMORY));
        };
        // SAFETY: plain object creation; a fresh mapping of that size.
        unsafe {
            std::ptr::write_bytes(words, 0, count as usize * 8);
            let mut heap: Option<ID3D12QueryHeap> = None;
            self.device.CreateQueryHeap(
                &D3D12_QUERY_HEAP_DESC { Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP, Count: count, NodeMask: 0 },
                &mut heap,
            )?;
            let heap = heap.ok_or_else(windows::core::Error::empty)?;
            let allocator: ID3D12CommandAllocator = self.device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let mut made = Vec::with_capacity(MADE as usize);
            for i in 0..MADE {
                let query = IN_LIST + i;
                let list: ID3D12GraphicsCommandList =
                    self.device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;
                list.EndQuery(&heap, D3D12_QUERY_TYPE_TIMESTAMP, query);
                list.ResolveQueryData(&heap, D3D12_QUERY_TYPE_TIMESTAMP, query, 1, &buffer, u64::from(query) * 8);
                list.Close()?;
                made.push(list.cast()?);
            }
            let frequency = self.queue.GetTimestampFrequency()?.max(1);
            name(&buffer, || "the card's time by part (VICTORY_FAST_STATS)".into());
            Ok(Stamps { heap, words: words as usize, made, _kept: (allocator, buffer), frequency })
        }
    }
}
