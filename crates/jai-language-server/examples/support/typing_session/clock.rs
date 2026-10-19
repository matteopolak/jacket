//! The per-request time budget and the watchdog that ends a hung process.

use super::{Arc, AtomicBool, AtomicU64, Duration, Instant, Mutex, Ordering, Query};

pub(super) struct Clock {
    pub(super) budget: Duration,
    pub(super) started: Arc<AtomicU64>,
    pub(super) label: Arc<Mutex<String>>,
    pub(super) stop: Arc<AtomicBool>,
    pub(super) epoch: Instant,
    pub(super) slow: Vec<String>,
    /// Whether to keep the queries made, for replaying a failure.
    pub(super) recording: bool,
    pub(super) log: Vec<Query>,
}

impl Clock {
    pub(super) fn new(budget: Duration, hang: Option<Duration>) -> Self {
        let clock = Self {
            budget,
            started: Arc::new(AtomicU64::new(0)),
            label: Arc::new(Mutex::new(String::new())),
            stop: Arc::new(AtomicBool::new(false)),
            epoch: Instant::now(),
            slow: Vec::new(),
            recording: false,
            log: Vec::new(),
        };
        if let Some(hang) = hang {
            let (started, label, stop, epoch) = (
                clock.started.clone(),
                clock.label.clone(),
                clock.stop.clone(),
                clock.epoch,
            );
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(200));
                    let began = started.load(Ordering::Relaxed);
                    let now = epoch.elapsed().as_millis() as u64 + 1;
                    if began != 0 && now.saturating_sub(began) > hang.as_millis() as u64 {
                        let what = label.lock().map(|l| l.clone()).unwrap_or_default();
                        eprintln!("typing session hung for {hang:?} in {what}");
                        std::process::exit(112);
                    }
                }
            });
        }
        clock
    }

    pub(super) fn run<R>(&mut self, query: Query, f: impl FnOnce() -> R) -> R {
        if let Ok(mut l) = self.label.lock() {
            *l = format!("{query:?}");
        }
        if self.recording {
            self.log.push(query);
        }
        self.started.store(
            self.epoch.elapsed().as_millis() as u64 + 1,
            Ordering::Relaxed,
        );
        let begin = Instant::now();
        let out = f();
        self.started.store(0, Ordering::Relaxed);
        if begin.elapsed() > self.budget {
            let what = self.label.lock().map(|l| l.clone()).unwrap_or_default();
            self.slow.push(format!("{what} took {:?}", begin.elapsed()));
        }
        out
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
