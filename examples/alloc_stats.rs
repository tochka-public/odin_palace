//! Считает количество и объём аллокаций на один разбор файла выписки.
//!
//! ```bash
//! cargo run --release --example alloc_stats -- benches/bench_statement.txt
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "benches/bench_statement.txt".to_string());
    let content = std::fs::read(&path)?;
    let parser = odin_palace::parser::Parser::default();

    // Прогрев, чтобы разовые инициализации не попадали в счётчики
    let _ = parser.parse(&content);

    let allocs_before = ALLOCATIONS.load(Ordering::Relaxed);
    let bytes_before = ALLOCATED_BYTES.load(Ordering::Relaxed);
    let statement = parser
        .parse(&content)
        .map_err(|e| format!("parse failed: {e:?}"))?;
    let allocs = ALLOCATIONS.load(Ordering::Relaxed) - allocs_before;
    let bytes = ALLOCATED_BYTES.load(Ordering::Relaxed) - bytes_before;

    println!(
        "file={path} docs={} allocations={allocs} allocated_bytes={bytes}",
        statement.documents.len()
    );
    Ok(())
}
