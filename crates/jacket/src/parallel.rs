//! Running the same task over many items on a few threads, for the work jacket hands to child
//! processes (compiles, examples, C sources).
//!
//! The thread count follows one rule: a command's own job count (`jacket test -j N`) when it has
//! one, else [`threads`], the number of cores. Never more threads than items.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The default thread count: the number of cores, 1 where the platform cannot say.
pub(crate) fn threads() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
}

/// `task(index, item)` for every item, `threads` at a time; the results in item order.
///
/// ```ignore
/// let lengths = parallel::map(&["a", "bcd"], parallel::threads(), |_, word| word.len());
/// assert_eq!(lengths, [1, 3]);
/// ```
pub(crate) fn map<T: Sync, R: Send>(
    items: &[T],
    threads: usize,
    task: impl Fn(usize, &T) -> R + Sync,
) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..threads.max(1).min(items.len()) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(index) else {
                        break;
                    };
                    let result = task(index, item);

                    *slots[index]
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(result);
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expect("every item ran")
        })
        .collect()
}

/// [`map`] for a task that can fail: once one fails no other item starts, and the error of the
/// first failed item (in item order) is returned.
///
/// ```ignore
/// let parsed = parallel::try_map(&["1", "x"], 2, |_, text| text.parse::<u8>());
/// assert!(parsed.is_err());
/// ```
pub(crate) fn try_map<T: Sync, R: Send, E: Send>(
    items: &[T],
    threads: usize,
    task: impl Fn(usize, &T) -> Result<R, E> + Sync,
) -> Result<Vec<R>, E> {
    let failed = AtomicBool::new(false);
    let results = map(items, threads, |index, item| {
        if failed.load(Ordering::Relaxed) {
            return None;
        }

        let result = task(index, item);

        if result.is_err() {
            failed.store(true, Ordering::Relaxed);
        }

        Some(result)
    });
    let mut done = Vec::with_capacity(items.len());
    let mut skipped = false;

    for result in results {
        match result {
            Some(Ok(value)) => done.push(value),
            Some(Err(error)) => return Err(error),
            None => skipped = true,
        }
    }

    assert!(!skipped, "an item was skipped only after another failed");
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_come_back_in_item_order() {
        let items: Vec<usize> = (0..50).collect();
        let doubled = map(&items, 4, |index, item| {
            assert_eq!(index, *item);
            item * 2
        });

        assert_eq!(doubled, (0..50).map(|i| i * 2).collect::<Vec<_>>());
        assert!(map(&[] as &[u8], 4, |_, _| ()).is_empty());
        let zero_sized = [(), (), ()];

        assert_eq!(map(&zero_sized, 2, |index, ()| index), [0, 1, 2]);
    }

    #[test]
    fn the_first_failure_stops_new_items_and_is_returned() {
        let started = AtomicUsize::new(0);
        let items: Vec<usize> = (0..100).collect();
        let result = try_map(&items, 1, |_, &item| {
            started.fetch_add(1, Ordering::Relaxed);

            if item >= 3 {
                Err(item)
            } else {
                Ok(item)
            }
        });

        assert_eq!(result, Err(3));
        assert_eq!(started.load(Ordering::Relaxed), 4);
        assert_eq!(
            try_map(&items[..3], 2, |_, &i| Ok::<_, ()>(i)),
            Ok(vec![0, 1, 2])
        );
    }
}
