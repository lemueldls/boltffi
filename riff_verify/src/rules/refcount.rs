use std::collections::{HashMap, HashSet};

use super::{Rule, Violation, ViolationKind};
use crate::analysis::{Effect, EffectTrace};
use crate::ir::VarId;

pub struct RetainReleaseBalance;

impl Rule for RetainReleaseBalance {
    fn id(&self) -> &'static str {
        "REF001"
    }

    fn description(&self) -> &'static str {
        "Every retained object must be released exactly once"
    }

    fn check(&self, trace: &EffectTrace) -> Vec<Violation> {
        let mut retained: HashMap<VarId, &crate::source::SourceSpan> = HashMap::new();
        let mut released: HashSet<VarId> = HashSet::new();
        let mut violations = Vec::new();

        trace.iter().for_each(|entry| match &entry.effect {
            Effect::Retain { opaque_handle, .. } => {
                retained.insert(*opaque_handle, &entry.span);
            }
            Effect::Release { opaque_handle } | Effect::TakeRetained { opaque_handle, .. } => {
                if !retained.contains_key(opaque_handle) {
                    violations.push(Violation::new(
                        ViolationKind::ReleaseUnretained {
                            handle: *opaque_handle,
                        },
                        self.id(),
                        entry.span.clone(),
                    ));
                } else {
                    released.insert(*opaque_handle);
                }
            }
            _ => {}
        });

        retained
            .iter()
            .filter(|(handle, _)| !released.contains(handle))
            .for_each(|(handle, span)| {
                violations.push(Violation::new(
                    ViolationKind::RetainLeak { handle: *handle },
                    self.id(),
                    (*span).clone(),
                ));
            });

        violations
    }
}

pub struct NoDoubleRelease;

impl Rule for NoDoubleRelease {
    fn id(&self) -> &'static str {
        "REF002"
    }

    fn description(&self) -> &'static str {
        "Objects must not be released multiple times"
    }

    fn check(&self, trace: &EffectTrace) -> Vec<Violation> {
        let mut released: HashSet<VarId> = HashSet::new();

        trace
            .iter()
            .filter_map(|entry| {
                let handle = match &entry.effect {
                    Effect::Release { opaque_handle } => Some(*opaque_handle),
                    Effect::TakeRetained { opaque_handle, .. } => Some(*opaque_handle),
                    _ => None,
                };

                handle.and_then(|h| {
                    if released.contains(&h) {
                        Some(Violation::new(
                            ViolationKind::DoubleRelease { handle: h },
                            self.id(),
                            entry.span.clone(),
                        ))
                    } else {
                        released.insert(h);
                        None
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SourceFile, SourceSpan};
    use std::sync::Arc;

    fn test_span() -> SourceSpan {
        let file = Arc::new(SourceFile::new("test.swift", "test content here"));
        SourceSpan::new(file, 0u32, 4u32)
    }

    #[test]
    fn test_balanced_retain_release() {
        let mut trace = EffectTrace::new();
        let obj = VarId::new(0);
        let handle = VarId::new(1);

        trace.push(
            Effect::Retain {
                object: obj,
                opaque_handle: handle,
            },
            test_span(),
        );
        trace.push(
            Effect::Release {
                opaque_handle: handle,
            },
            test_span(),
        );

        let rule = RetainReleaseBalance;
        let violations = rule.check(&trace);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_retain_leak_detected() {
        let mut trace = EffectTrace::new();
        let obj = VarId::new(0);
        let handle = VarId::new(1);

        trace.push(
            Effect::Retain {
                object: obj,
                opaque_handle: handle,
            },
            test_span(),
        );

        let rule = RetainReleaseBalance;
        let violations = rule.check(&trace);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::RetainLeak { .. }
        ));
    }

    #[test]
    fn test_double_release_detected() {
        let mut trace = EffectTrace::new();
        let obj = VarId::new(0);
        let handle = VarId::new(1);

        trace.push(
            Effect::Retain {
                object: obj,
                opaque_handle: handle,
            },
            test_span(),
        );
        trace.push(
            Effect::Release {
                opaque_handle: handle,
            },
            test_span(),
        );
        trace.push(
            Effect::Release {
                opaque_handle: handle,
            },
            test_span(),
        );

        let rule = NoDoubleRelease;
        let violations = rule.check(&trace);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::DoubleRelease { .. }
        ));
    }

    #[test]
    fn test_take_retained_counts_as_release() {
        let mut trace = EffectTrace::new();
        let obj = VarId::new(0);
        let handle = VarId::new(1);
        let result = VarId::new(2);

        trace.push(
            Effect::Retain {
                object: obj,
                opaque_handle: handle,
            },
            test_span(),
        );
        trace.push(
            Effect::TakeRetained {
                opaque_handle: handle,
                result,
            },
            test_span(),
        );

        let rule = RetainReleaseBalance;
        let violations = rule.check(&trace);
        assert!(violations.is_empty());
    }
}
