//! Interpreted atomics against simultaneous host access.
use super::*;
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU64, Ordering},
};

#[test]
fn interpreted_cas_is_atomic_against_simultaneous_host_access() {
    let value = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let value = value.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut interp = Interp::new(Box::new(NativeHost));
                let address = Arc::as_ptr(&value) as u64;
                barrier.wait();
                for _ in 0..20_000 {
                    loop {
                        let before = value.load(Ordering::SeqCst);
                        let result = interp
                            .intrinsic(
                                ir::Intrinsic::AtomicCas(4, 4),
                                &[address, before, before + 1, 8],
                                None,
                            )
                            .unwrap()
                            .into_vec();
                        if result[0] != 0 {
                            break;
                        }
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(value.load(Ordering::SeqCst), 80_000);
}

#[test]
fn interpreted_exchange_preserves_all_prior_values_under_contention() {
    let value = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let value = value.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut interp = Interp::new(Box::new(NativeHost));
                let address = Arc::as_ptr(&value) as u64;
                barrier.wait();
                let mut sum = 0;
                let mut observed = Vec::with_capacity(20_000);
                for i in 1..=20_000 {
                    let old = interp
                        .intrinsic(
                            ir::Intrinsic::AtomicRmw(ir::AtomicOp::Xchg, ir::ORDER_SEQ_CST),
                            &[address, index * 20_000 + i, 8],
                            None,
                        )
                        .unwrap()
                        .into_vec()[0];
                    sum += old;
                    observed.push(old);
                }
                (sum, observed)
            })
        })
        .collect();
    let mut observed = Vec::with_capacity(80_001);
    let sum: u64 = workers
        .into_iter()
        .map(|worker| {
            let (sum, mut prior) = worker.join().unwrap();
            observed.append(&mut prior);
            sum
        })
        .sum();
    let last = value.load(Ordering::SeqCst);
    assert_eq!(sum + last, 80_000 * 80_001 / 2);
    observed.push(last);
    observed.sort_unstable();
    assert_eq!(observed.len(), 80_001);
    for (index, old) in observed.into_iter().enumerate() {
        assert_eq!(
            old, index as u64,
            "every prior exchange value occurs exactly once"
        );
    }
}
