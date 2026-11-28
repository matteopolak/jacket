//! Dependency standings found by `jacket dep`'s own code on background threads.
//!
//! A request is answered from what the last lookup found; a lookup that is missing or older
//! than [`REFRESH`] starts on a thread of its own (at most [`THREADS`] at once), and the server
//! is told when one finishes so the client asks for its hints again.
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use jaic::fxhash::HashMap;

use super::dependencies::{Request, Source, Standing};

/// How long a standing is shown before it is looked up again.
const REFRESH: Duration = Duration::from_mins(10);

/// How many lookups run at once.
const THREADS: usize = 4;

#[derive(Default)]
struct Shared {
    entries: HashMap<String, Entry>,
    running: usize,
    arrived: bool,
}

#[derive(Default)]
struct Entry {
    found: Option<(Instant, Standing)>,
    running: bool,
}

#[derive(Default)]
pub struct Live {
    shared: Arc<Mutex<Shared>>,
}

fn locked(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    // A lookup that panicked left its entry marked running; nothing else is shared.
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Source for Live {
    fn lookup(&self, request: &Request) -> Option<Standing> {
        let key = format!(
            "{}\0{}\0{:?}\0{:?}",
            request.dir.display(),
            request.name,
            request.entry,
            request.resolver
        );
        let mut shared = locked(&self.shared);
        let running = shared.running;
        let entry = shared.entries.entry(key.clone()).or_default();
        let stale = entry
            .found
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() > REFRESH);
        let answer = entry.found.as_ref().map(|(_, standing)| standing.clone());

        if stale && !entry.running && running < THREADS {
            entry.running = true;
            shared.running += 1;
            drop(shared);
            self.start(key, request.clone());
        }

        answer
    }

    fn waiting(&self) -> bool {
        locked(&self.shared).running > 0
    }

    fn take_arrivals(&self) -> bool {
        std::mem::take(&mut locked(&self.shared).arrived)
    }
}

impl Live {
    fn start(&self, key: String, request: Request) {
        let shared = Arc::clone(&self.shared);
        let work = {
            let shared = Arc::clone(&shared);
            let key = key.clone();

            move || {
                let found = jacket_dep::dep::standing::lookup(
                    &request.dir,
                    &request.name,
                    &request.entry,
                    &request.resolver,
                );

                finish(&shared, &key, standing_of(found));
            }
        };

        if let Err(error) = std::thread::Builder::new()
            .name("jailsp-dependencies".into())
            .spawn(work)
        {
            finish(
                &shared,
                &key,
                Standing {
                    problem: Some(format!("a lookup thread could not start: {error}")),
                    ..Standing::default()
                },
            );
        }
    }
}

fn standing_of(found: Result<jacket_dep::dep::standing::Standing, String>) -> Standing {
    match found {
        Ok(found) => Standing {
            pinned: found.pinned,
            newer: found.newer,
            waiting: found
                .waiting
                .map(|(tag, seconds)| (tag, jacket_dep::dep::standing::wait_text(seconds))),
            problem: None,
        },
        Err(problem) => Standing {
            problem: Some(problem),
            ..Standing::default()
        },
    }
}

fn finish(shared: &Mutex<Shared>, key: &str, standing: Standing) {
    let mut shared = locked(shared);

    shared.running = shared.running.saturating_sub(1);
    shared.arrived = true;

    if let Some(entry) = shared.entries.get_mut(key) {
        entry.running = false;
        entry.found = Some((Instant::now(), standing));
    }
}
