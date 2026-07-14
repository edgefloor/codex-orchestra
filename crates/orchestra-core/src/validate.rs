use crate::{Action, ExecutionPlan, ForkTurns, WorktreePolicy};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
#[error("{path}: {message}")]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

pub fn validate_plan(plan: &ExecutionPlan) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    if plan.name.trim().is_empty() {
        push(&mut errors, "name", "must not be empty");
    }
    if !(1..=32).contains(&plan.max_parallel) {
        push(&mut errors, "max_parallel", "must be between 1 and 32");
    }
    let mut ids = BTreeSet::new();
    for (index, step) in plan.steps.iter().enumerate() {
        let path = format!("steps[{index}]");
        if !valid_id(&step.id) {
            push(
                &mut errors,
                &format!("{path}.id"),
                "must use lowercase letters, digits, `_`, or `-`",
            );
        }
        if !ids.insert(step.id.clone()) {
            push(&mut errors, &format!("{path}.id"), "duplicate step id");
        }
        if step.max_attempts == 0 || step.max_attempts > 10 {
            push(
                &mut errors,
                &format!("{path}.max_attempts"),
                "must be between 1 and 10",
            );
        }
        if let Some(repeat) = &step.repeat {
            if repeat.max_rounds == 0 || repeat.max_rounds > 20 {
                push(
                    &mut errors,
                    &format!("{path}.repeat.max_rounds"),
                    "must be between 1 and 20",
                );
            }
            if repeat.until_output.is_empty() {
                push(
                    &mut errors,
                    &format!("{path}.repeat.until_output"),
                    "must name an output",
                );
            }
        }
        match &step.action {
            Action::Agent(agent) => {
                if agent.model.trim().is_empty() {
                    push(
                        &mut errors,
                        &format!("{path}.model"),
                        "explicit model is required",
                    );
                }
                if matches!(agent.fork_turns, ForkTurns::All)
                    && (agent.reasoning_effort.is_some() || agent.service_tier.is_some())
                {
                    push(
                        &mut errors,
                        &format!("{path}.fork_turns"),
                        "full-history forks cannot override reasoning or service tier",
                    );
                }
            }
            Action::Check(check) if check.command.is_empty() => {
                push(&mut errors, &format!("{path}.command"), "must not be empty")
            }
            Action::Approval(_) => {}
            Action::Check(_) => {}
        }
    }
    for (index, step) in plan.steps.iter().enumerate() {
        for dependency in &step.needs {
            if !ids.contains(dependency) {
                push(
                    &mut errors,
                    &format!("steps[{index}].needs"),
                    &format!("unknown dependency `{dependency}`"),
                );
            }
        }
    }
    detect_cycles(plan, &mut errors);
    detect_write_conflicts(plan, &mut errors);
    errors
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
}

fn push(errors: &mut Vec<ValidationError>, path: &str, message: &str) {
    errors.push(ValidationError {
        path: path.into(),
        message: message.into(),
    });
}

fn detect_cycles(plan: &ExecutionPlan, errors: &mut Vec<ValidationError>) {
    let graph: BTreeMap<_, _> = plan
        .steps
        .iter()
        .map(|s| {
            (
                s.id.as_str(),
                s.needs.iter().map(String::as_str).collect::<Vec<_>>(),
            )
        })
        .collect();
    fn visit<'a>(
        id: &'a str,
        graph: &BTreeMap<&'a str, Vec<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> bool {
        if done.contains(id) {
            return false;
        }
        if !visiting.insert(id) {
            return true;
        }
        if graph
            .get(id)
            .is_some_and(|deps| deps.iter().any(|dep| visit(dep, graph, visiting, done)))
        {
            return true;
        }
        visiting.remove(id);
        done.insert(id);
        false
    }
    let mut done = BTreeSet::new();
    for id in graph.keys() {
        if visit(id, &graph, &mut BTreeSet::new(), &mut done) {
            push(errors, "steps", "dependency cycle detected");
            break;
        }
    }
}

fn detect_write_conflicts(plan: &ExecutionPlan, errors: &mut Vec<ValidationError>) {
    for (i, left) in plan.steps.iter().enumerate() {
        for right in plan.steps.iter().skip(i + 1) {
            let ordered = left.needs.contains(&right.id) || right.needs.contains(&left.id);
            if ordered || left.write_scope.is_empty() || right.write_scope.is_empty() {
                continue;
            }
            let overlaps = left.write_scope.iter().any(|a| {
                right
                    .write_scope
                    .iter()
                    .any(|b| a.starts_with(b) || b.starts_with(a))
            });
            if overlaps
                && (left.worktree == WorktreePolicy::Shared
                    || right.worktree == WorktreePolicy::Shared)
            {
                push(
                    errors,
                    "steps",
                    &format!(
                        "parallel writers `{}` and `{}` overlap without isolated worktrees",
                        left.id, right.id
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, AgentStep, ExecutionPlan, ForkTurns, Step};

    fn writer(id: &str, scope: &str, worktree: WorktreePolicy) -> Step {
        Step {
            id: id.into(),
            needs: vec![],
            max_attempts: 1,
            repeat: None,
            worktree,
            write_scope: vec![scope.into()],
            action: Action::Agent(AgentStep {
                prompt: "write".into(),
                model: "gpt-5.4".into(),
                reasoning_effort: None,
                service_tier: None,
                fork_turns: ForkTurns::None,
                context: vec![],
                outputs: vec![],
                allow_delegation: false,
            }),
        }
    }

    #[test]
    fn rejects_cycles_and_unknown_dependencies() {
        let mut a = writer("a", "a/", WorktreePolicy::Shared);
        let mut b = writer("b", "b/", WorktreePolicy::Shared);
        a.needs = vec!["b".into()];
        b.needs = vec!["a".into(), "missing".into()];
        let errors = validate_plan(&ExecutionPlan {
            name: "bad".into(),
            description: String::new(),
            max_parallel: 2,
            steps: vec![a, b],
        });
        assert!(errors.iter().any(|error| error.message.contains("cycle")));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("unknown dependency"))
        );
    }

    #[test]
    fn overlapping_parallel_writers_require_isolation() {
        let plan = ExecutionPlan {
            name: "conflict".into(),
            description: String::new(),
            max_parallel: 2,
            steps: vec![
                writer("a", "src/", WorktreePolicy::Shared),
                writer("b", "src/lib", WorktreePolicy::Isolated),
            ],
        };
        assert!(
            validate_plan(&plan)
                .iter()
                .any(|error| error.message.contains("parallel writers"))
        );
    }

    #[test]
    fn full_history_rejects_explicit_overrides() {
        let mut step = writer("a", "src/", WorktreePolicy::Isolated);
        let Action::Agent(agent) = &mut step.action else {
            unreachable!()
        };
        agent.fork_turns = ForkTurns::All;
        agent.reasoning_effort = Some("high".into());
        assert!(
            validate_plan(&ExecutionPlan {
                name: "fork".into(),
                description: String::new(),
                max_parallel: 1,
                steps: vec![step]
            })
            .iter()
            .any(|error| error.message.contains("full-history"))
        );
    }
}
