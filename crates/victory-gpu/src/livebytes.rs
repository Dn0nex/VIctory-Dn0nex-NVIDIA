//! Bytes alive of host buffers in the PC's memory, by kind, for the memory
//! line: what counts against DXGI's shared (non-local) budget and the
//! commit limit. `MADE` in `host` counts what was ever made; this counts
//! what is still held, decremented when the driver releases the resource
//! (a private-data object of ours dies with it, as in `pool`).

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use windows::Win32::Graphics::Direct3D12::ID3D12Resource;
use windows::core::{GUID, HRESULT, IUnknown, Interface};

/// Kinds counted.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    /// Write-back custom-heap buffers (`Host::shared_buffer`).
    Shared = 0,
    /// Upload-heap buffers (`Host::upload_buffer`).
    Upload = 1,
    /// Read-back-heap buffers (`Host::read_back_buffer`).
    ReadBack = 2,
}

const KINDS: usize = 3;

/// Live bytes and live count per kind.
static BYTES: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];
static COUNT: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];

/// `[bytes, count]` alive of `kind`.
pub(crate) fn live(kind: Kind) -> [u64; 2] {
    [BYTES[kind as usize].load(Ordering::Relaxed), COUNT[kind as usize].load(Ordering::Relaxed)]
}

#[repr(C)]
struct Gone {
    methods: &'static [*const (); 3],
    references: AtomicU32,
    kind: usize,
    bytes: u64,
}

struct Methods([*const (); 3]);
// SAFETY: function addresses only.
unsafe impl Sync for Methods {}

static GONE: Methods = Methods([query as *const (), add_ref as *const (), release as *const ()]);

unsafe extern "system" fn query(this: *mut c_void, iid: *const GUID, out: *mut *mut c_void) -> HRESULT {
    // SAFETY: COM's arguments.
    unsafe {
        if out.is_null() {
            return HRESULT(0x8000_4003_u32 as i32);
        }
        if iid.as_ref().is_some_and(|iid| *iid == IUnknown::IID) {
            add_ref(this);
            *out = this;
            return HRESULT(0);
        }
        *out = std::ptr::null_mut();
    }
    HRESULT(0x8000_4002_u32 as i32)
}

unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: one of ours, alive while a reference to it is.
    unsafe { (*this.cast::<Gone>()).references.fetch_add(1, Ordering::Relaxed) + 1 }
}

unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: as above; the last reference frees the box it was made as.
    unsafe {
        let left = (*this.cast::<Gone>()).references.fetch_sub(1, Ordering::AcqRel) - 1;
        if left == 0 {
            let gone = Box::from_raw(this.cast::<Gone>());
            BYTES[gone.kind].fetch_sub(gone.bytes, Ordering::Relaxed);
            COUNT[gone.kind].fetch_sub(1, Ordering::Relaxed);
        }
        left
    }
}

/// Counts `bytes` of `kind` alive until `resource` is released.
pub(crate) fn track(resource: &ID3D12Resource, kind: Kind, bytes: u64) {
    const KEY: GUID = GUID::from_u128(0x3b8c6a51_2f7e_4c0d_9e15_8a4f6d2b7c90);
    let made = Box::into_raw(Box::new(Gone { methods: &GONE.0, references: AtomicU32::new(1), kind: kind as usize, bytes }));
    // SAFETY: a COM object of ours with one reference, which this takes and
    // drops at the end; the resource takes its own.
    unsafe {
        let object = IUnknown::from_raw(made.cast());
        if resource.SetPrivateDataInterface(&KEY, &object).is_ok() {
            BYTES[kind as usize].fetch_add(bytes, Ordering::Relaxed);
            COUNT[kind as usize].fetch_add(1, Ordering::Relaxed);
        }
        // `object` drops here: on failure that frees it without counting.
    }
}

/// The process's private commit and working set, in bytes.
pub(crate) fn process() -> [u64; 2] {
    use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut c = PROCESS_MEMORY_COUNTERS { cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32, ..Default::default() };
    // SAFETY: our own process and a sized structure.
    unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    [c.PagefileUsage as u64, c.WorkingSetSize as u64]
}

/// One line of all of it (MB), for the memory diagnostics.
pub(crate) fn summary() -> String {
    let [private, ws] = process();
    let [sb, sn] = live(Kind::Shared);
    let [ub, un] = live(Kind::Upload);
    let [rb, rn] = live(Kind::ReadBack);
    format!(
        "proc_private={}MB proc_ws={}MB shared_bufs={}MB/{sn} upload_bufs={}MB/{un} readback_bufs={}MB/{rn}",
        private >> 20,
        ws >> 20,
        sb >> 20,
        ub >> 20,
        rb >> 20
    )
}
