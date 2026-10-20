//! Which page renders next, and where. hayro cannot stop a page once it has started (its
//! rasterizer is private), so the scheduler decides before:
//!
//! - renders run on its own worker threads, one per core, so their memory stays in that many
//!   allocator arenas however many callers wait (with renders on any pool thread, glibc's
//!   per-thread arenas kept it: 536 MB instead of 226 MB on a two-core run, `.knowledge/pdf-engine.md`);
//! - the newest request goes first (after fast scrolling, the page in view renders before those
//!   scrolled past);
//! - a request superseded by a newer one for the same page is dropped unrendered, and its caller
//!   gets [`DocumentError::Cancelled`].

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

use crate::error::DocumentError;

/// Most render workers. Each keeps the memory of its last renders in its allocator arena; past 4,
/// memory grows with the cores and latency does not improve (measured, `.knowledge/pdf-engine.md`).
const MAX_WORKERS: usize = 4;

/// One page of one file.
type Key = (PathBuf, u32);

pub(crate) struct Scheduler {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
    /// A job was queued.
    queued: Condvar,
}

#[derive(Default)]
struct State {
    /// Jobs waiting for a worker.
    jobs: Vec<Job>,
    /// The newest ticket of each page.
    latest: HashMap<Key, u64>,
    next: u64,
}

struct Job {
    ticket: u64,
    key: Key,
    work: Box<dyn FnOnce() + Send>,
}

pub(crate) fn global() -> &'static Scheduler {
    static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();
    SCHEDULER
        .get_or_init(|| Scheduler::new(std::thread::available_parallelism().map_or(2, |n| n.get()).min(MAX_WORKERS)))
}

impl Scheduler {
    /// A scheduler with `workers` render threads (they live as long as the process).
    pub(crate) fn new(workers: usize) -> Self {
        let shared = Arc::new(Shared { state: Mutex::default(), queued: Condvar::new() });
        for n in 0..workers.max(1) {
            let shared = shared.clone();
            std::thread::Builder::new()
                .name(format!("quire-render-{n}"))
                .spawn(move || work(&shared))
                .expect("a render thread");
        }
        Self { shared }
    }

    /// Runs `render` for page `page` of `path` on a worker, newest request first, and waits for
    /// it; [`DocumentError::Cancelled`] if a newer request for the same page came first. A panic in
    /// `render` resumes here (the worker survives it).
    pub(crate) fn run<T: Send + 'static>(
        &self,
        path: &Path,
        page: u32,
        render: impl FnOnce() -> Result<T, DocumentError> + Send + 'static,
    ) -> Result<T, DocumentError> {
        let (send, receive) = mpsc::channel();
        let key = (path.to_path_buf(), page);
        {
            let mut state = lock(&self.shared.state);
            state.next += 1;
            let ticket = state.next;
            // An older request for this page still waiting is dropped: its caller sees the channel close.
            state.jobs.retain(|job| job.key != key);
            state.latest.insert(key.clone(), ticket);
            let work = Box::new(move || {
                let _ = send.send(catch_unwind(AssertUnwindSafe(render)));
            });
            state.jobs.push(Job { ticket, key, work });
        }
        self.shared.queued.notify_one();
        match receive.recv() {
            Ok(Ok(result)) => result,
            Ok(Err(panic)) => resume_unwind(panic),
            Err(mpsc::RecvError) => Err(DocumentError::Cancelled(page)),
        }
    }

    /// Tickets issued and jobs waiting.
    #[cfg(test)]
    fn counts(&self) -> (u64, usize) {
        let state = lock(&self.shared.state);
        (state.next, state.jobs.len())
    }
}

/// A worker: the newest waiting job, forever.
fn work(shared: &Shared) {
    loop {
        let job = {
            let mut state = lock(&shared.state);
            loop {
                let newest = state.jobs.iter().enumerate().max_by_key(|(_, job)| job.ticket).map(|(i, _)| i);
                if let Some(i) = newest {
                    break state.jobs.swap_remove(i);
                }
                state = shared.queued.wait(state).unwrap_or_else(PoisonError::into_inner);
            }
        };
        let Job { ticket, key, work } = job;
        work();
        let mut state = lock(&shared.state);
        if state.latest.get(&key) == Some(&ticket) {
            state.latest.remove(&key);
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    /// Waits until `issued` tickets were handed out and `waiting` of them wait.
    fn wait_until(scheduler: &Scheduler, issued: u64, waiting: usize) {
        for _ in 0..1000 {
            if scheduler.counts() == (issued, waiting) {
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("{:?}, expected {:?}", scheduler.counts(), (issued, waiting));
    }

    /// The one slot is busy while 20 requests for one page arrive: only the last one renders.
    #[test]
    fn only_the_newest_request_for_a_page_renders() {
        let scheduler = Arc::new(Scheduler::new(1));
        let (release, hold) = mpsc::channel::<()>();
        let busy = {
            let scheduler = scheduler.clone();
            std::thread::spawn(move || {
                scheduler.run(Path::new("other.pdf"), 0, move || {
                    hold.recv().unwrap();
                    Ok(())
                })
            })
        };
        wait_until(&scheduler, 1, 0);
        let rendered = Arc::new(AtomicUsize::new(0));
        let mut requests = Vec::new();
        for n in 0..20u64 {
            let (queue, rendered) = (scheduler.clone(), rendered.clone());
            requests.push(std::thread::spawn(move || {
                queue.run(Path::new("paper.pdf"), 3, move || {
                    rendered.fetch_add(1, Ordering::SeqCst);
                    Ok(n)
                })
            }));
            // In order, so request 19 is the newest. Each one drops the one before it from the queue.
            wait_until(&scheduler, n + 2, 1);
        }
        release.send(()).unwrap();
        busy.join().unwrap().unwrap();
        let results: Vec<_> = requests.into_iter().map(|r| r.join().unwrap()).collect();
        assert_eq!(rendered.load(Ordering::SeqCst), 1);
        assert!(matches!(results[19], Ok(19)));
        assert!(results[..19].iter().all(|r| matches!(r, Err(DocumentError::Cancelled(3)))));
    }

    /// Different pages all render, the newest first.
    #[test]
    fn waiting_pages_render_newest_first() {
        let scheduler = Arc::new(Scheduler::new(1));
        let (release, hold) = mpsc::channel::<()>();
        let busy = {
            let scheduler = scheduler.clone();
            std::thread::spawn(move || {
                scheduler.run(Path::new("paper.pdf"), 0, move || {
                    hold.recv().unwrap();
                    Ok(())
                })
            })
        };
        wait_until(&scheduler, 1, 0);
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut requests = Vec::new();
        for page in 1..=3 {
            let (queue, order) = (scheduler.clone(), order.clone());
            requests.push(std::thread::spawn(move || {
                queue.run(Path::new("paper.pdf"), page, move || {
                    order.lock().unwrap().push(page);
                    Ok(())
                })
            }));
            wait_until(&scheduler, page as u64 + 1, page as usize);
        }
        release.send(()).unwrap();
        busy.join().unwrap().unwrap();
        for request in requests {
            request.join().unwrap().unwrap();
        }
        assert_eq!(*order.lock().unwrap(), [3, 2, 1]);
    }

    /// A render that panics does not take its worker with it (else the next render waits forever).
    #[test]
    fn a_panicking_render_keeps_its_worker() {
        let scheduler = Arc::new(Scheduler::new(1));
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scheduler.run(Path::new("paper.pdf"), 0, move || -> Result<(), DocumentError> { panic!("damaged page") })
        }));
        assert!(panicked.is_err());
        let (done, finished) = mpsc::channel();
        let queue = scheduler.clone();
        std::thread::spawn(move || done.send(queue.run(Path::new("paper.pdf"), 0, move || Ok(1)).is_ok()));
        assert_eq!(finished.recv_timeout(Duration::from_secs(5)), Ok(true), "the worker died");
    }
}
