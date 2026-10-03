//! Physical case-body order is independent of labelled dispatch priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseTarget {
    Arm(usize),
    Default,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseOrderError {
    DefaultPositionOutOfBounds,
    DefaultMetadataMismatch,
    InvalidTarget(CaseTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaseOrder {
    arms: usize,
    default_position: Option<usize>,
}

impl CaseOrder {
    /// `default_position` counts normal arms physically before the bare case.
    pub fn new(arms: usize, default_position: Option<usize>) -> Result<Self, CaseOrderError> {
        if default_position.is_some_and(|position| position > arms) {
            return Err(CaseOrderError::DefaultPositionOutOfBounds);
        }
        Ok(Self {
            arms,
            default_position,
        })
    }

    pub fn first(self) -> CaseTarget {
        if self.default_position == Some(0) {
            CaseTarget::Default
        } else if self.arms != 0 {
            CaseTarget::Arm(0)
        } else {
            CaseTarget::End
        }
    }

    pub fn last(self) -> CaseTarget {
        if self.default_position == Some(self.arms) {
            CaseTarget::Default
        } else if self.arms != 0 {
            CaseTarget::Arm(self.arms - 1)
        } else {
            CaseTarget::End
        }
    }

    /// Walk backwards when deriving the exits reachable from each body.
    pub fn preceding(self, target: CaseTarget) -> Result<CaseTarget, CaseOrderError> {
        match target {
            CaseTarget::Arm(index) if index < self.arms => {
                Ok(if self.default_position == Some(index) {
                    CaseTarget::Default
                } else if index != 0 {
                    CaseTarget::Arm(index - 1)
                } else {
                    CaseTarget::End
                })
            }
            CaseTarget::Default => {
                let position = self
                    .default_position
                    .ok_or(CaseOrderError::InvalidTarget(target))?;
                Ok(if position != 0 {
                    CaseTarget::Arm(position - 1)
                } else {
                    CaseTarget::End
                })
            }
            _ => Err(CaseOrderError::InvalidTarget(target)),
        }
    }

    /// Follow the next physical body without evaluating any additional label.
    pub fn following(self, target: CaseTarget) -> Result<CaseTarget, CaseOrderError> {
        match target {
            CaseTarget::Arm(index) if index < self.arms => {
                let next = index + 1;
                Ok(if self.default_position == Some(next) {
                    CaseTarget::Default
                } else if next < self.arms {
                    CaseTarget::Arm(next)
                } else {
                    CaseTarget::End
                })
            }
            CaseTarget::Default => {
                let position = self
                    .default_position
                    .ok_or(CaseOrderError::InvalidTarget(target))?;
                Ok(if position < self.arms {
                    CaseTarget::Arm(position)
                } else {
                    CaseTarget::End
                })
            }
            _ => Err(CaseOrderError::InvalidTarget(target)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_default_slot_preserves_physical_body_order() {
        for count in 0..8 {
            for position in 0..=count {
                let order = CaseOrder::new(count, Some(position)).unwrap();
                let mut actual = Vec::new();
                let mut target = order.first();
                while target != CaseTarget::End {
                    actual.push(target);
                    target = order.following(target).unwrap();
                }
                let mut expected = (0..count).map(CaseTarget::Arm).collect::<Vec<_>>();
                expected.insert(position, CaseTarget::Default);
                assert_eq!(actual, expected);
                let mut reverse = Vec::new();
                let mut target = order.last();
                while target != CaseTarget::End {
                    reverse.push(target);
                    target = order.preceding(target).unwrap();
                }
                expected.reverse();
                assert_eq!(reverse, expected);
            }
        }
        for count in 0..8 {
            let order = CaseOrder::new(count, None).unwrap();
            let mut reverse = Vec::new();
            let mut target = order.last();
            while target != CaseTarget::End {
                reverse.push(target);
                target = order.preceding(target).unwrap();
            }
            assert_eq!(
                reverse,
                (0..count).rev().map(CaseTarget::Arm).collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn invalid_layouts_and_targets_are_rejected() {
        assert_eq!(
            CaseOrder::new(1, Some(2)),
            Err(CaseOrderError::DefaultPositionOutOfBounds)
        );
        let order = CaseOrder::new(1, None).unwrap();
        assert_eq!(
            order.following(CaseTarget::Default),
            Err(CaseOrderError::InvalidTarget(CaseTarget::Default))
        );
        assert_eq!(
            order.following(CaseTarget::Arm(1)),
            Err(CaseOrderError::InvalidTarget(CaseTarget::Arm(1)))
        );
        assert_eq!(
            order.following(CaseTarget::End),
            Err(CaseOrderError::InvalidTarget(CaseTarget::End))
        );
        assert_eq!(order.following(CaseTarget::Arm(0)), Ok(CaseTarget::End));
        // No count-plus-default arithmetic can overflow a valid last ordinal.
        let order = CaseOrder::new(usize::MAX, Some(usize::MAX)).unwrap();
        assert_eq!(
            order.following(CaseTarget::Arm(usize::MAX - 1)),
            Ok(CaseTarget::Default)
        );
        assert_eq!(order.following(CaseTarget::Default), Ok(CaseTarget::End));
        assert_eq!(
            order.preceding(CaseTarget::Default),
            Ok(CaseTarget::Arm(usize::MAX - 1))
        );
    }
}
