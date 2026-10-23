//! Compile-time services in the editor (trusted workspaces only): answers are remembered by
//! request, and one that takes longer than a moment reads as `.PENDING` until it arrives, so a
//! keystroke never waits on a service. The transport polls for arrivals and checks the open
//! files again when one comes in (see `main.rs`). Builds ask the same services and wait.
use jaic::fxhash::HashMap;
use jaic::service::{ServiceProvider, ServiceReply, ServiceStatus, Services, Ticket};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a compile waits for an answer before it reads as pending.
const FIRST_WAIT: Duration = Duration::from_millis(50);

/// Failures are asked again after this long (a database that was down may be up).
const FAILURE_KEPT: Duration = Duration::from_secs(10);

/// Remembered answers past this many are all dropped.
const MEMO_LIMIT: usize = 4096;

pub struct EditorServices {
    services: Services,
    state: RefCell<State>,
}

#[derive(Default)]
struct State {
    answers: HashMap<(PathBuf, Vec<u8>), Answer>,
    /// The sources fingerprint of each service and when it was last taken.
    sources: HashMap<PathBuf, (u64, Instant)>,
}

enum Answer {
    Waiting(Ticket),
    Done(ServiceReply, Instant),
}

impl EditorServices {
    #[must_use]
    pub fn new(services: Services) -> Self {
        Self {
            services,
            state: RefCell::default(),
        }
    }

    /// Drops what `service` answered before its sources (or `jaic`) changed.
    fn check_sources(&self, state: &mut State, service: &Path) {
        let now = Instant::now();
        if let Some((_, at)) = state.sources.get(service)
            && now.duration_since(*at) < Duration::from_secs(1)
        {
            return;
        }
        let jaic = self
            .services
            .config()
            .jaic
            .clone()
            .or_else(jaic::service::find_jaic);
        let current = jaic::service::fingerprint(service, jaic.as_deref());
        let previous = state.sources.insert(service.to_path_buf(), (current, now));
        if previous.is_some_and(|(before, _)| before != current) {
            state.answers.retain(|(path, _), _| path != service);
        }
    }
}

impl ServiceProvider for EditorServices {
    fn ask(&self, service: &Path, request: &[u8]) -> ServiceReply {
        let mut state = self.state.borrow_mut();
        self.check_sources(&mut state, service);
        let key = (service.to_path_buf(), request.to_vec());
        match state.answers.get(&key) {
            Some(Answer::Done(reply, at))
                if reply.status == ServiceStatus::Ok || at.elapsed() < FAILURE_KEPT =>
            {
                return reply.clone();
            }
            Some(Answer::Waiting(ticket)) => {
                return match ticket.poll() {
                    Some(reply) => {
                        state
                            .answers
                            .insert(key, Answer::Done(reply.clone(), Instant::now()));
                        reply
                    }
                    None => ServiceReply::pending(),
                };
            }
            _ => {}
        }
        if state.answers.len() >= MEMO_LIMIT {
            state.answers.clear();
        }
        let ticket = self.services.submit(service, request.to_vec());
        if let Some(reply) = ticket.wait(FIRST_WAIT) {
            state
                .answers
                .insert(key, Answer::Done(reply.clone(), Instant::now()));
            reply
        } else {
            state.answers.insert(key, Answer::Waiting(ticket));
            ServiceReply::pending()
        }
    }

    fn waiting(&self) -> bool {
        self.state
            .borrow()
            .answers
            .values()
            .any(|answer| matches!(answer, Answer::Waiting(_)))
    }

    fn take_arrivals(&self) -> bool {
        let mut arrived = false;
        for answer in self.state.borrow_mut().answers.values_mut() {
            if let Answer::Waiting(ticket) = answer
                && let Some(reply) = ticket.poll()
            {
                *answer = Answer::Done(reply, Instant::now());
                arrived = true;
            }
        }
        arrived
    }
}
