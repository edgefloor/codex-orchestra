from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

import jsonschema

PLUGIN = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("workflow", PLUGIN / "scripts/workflow.py")
assert SPEC and SPEC.loader
workflow_module = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = workflow_module
SPEC.loader.exec_module(workflow_module)
FIXTURE = PLUGIN / "evals/workflows/native-vertical-slice.yaml"


class WorkflowTests(unittest.TestCase):
    def base(self) -> dict:
        return workflow_module.load_workflow(FIXTURE)

    def errors(self, value: dict) -> list[str]:
        return workflow_module.schema_errors(value) + workflow_module.semantic_errors(value)

    def state(self, value: dict) -> dict:
        return {
            "status": "pending",
            "steps": {step["id"]: {"status": "pending", "attempts": 0} for step in value["steps"]},
            "next_action": "Run dependency-ready steps.",
        }

    def test_vertical_slice_is_valid(self) -> None:
        self.assertEqual(workflow_module.validate(FIXTURE), [])

    def test_all_workflow_fixtures_and_template_are_valid(self) -> None:
        paths = [PLUGIN / "assets/templates/WORKFLOW.yaml", *sorted((PLUGIN / "evals/workflows").glob("*.yaml"))]
        for path in paths:
            with self.subTest(path=path):
                self.assertEqual(workflow_module.validate(path), [])

    def test_all_json_schemas_are_well_formed(self) -> None:
        for path in (PLUGIN / "assets/schemas").glob("*.json"):
            with self.subTest(path=path):
                schema = json.loads(path.read_text(encoding="utf-8"))
                jsonschema.validators.validator_for(schema).check_schema(schema)

    def test_missing_dependency_is_rejected(self) -> None:
        value = self.base()
        value["steps"][1]["needs"] = ["missing"]
        self.assertTrue(any("unknown dependency missing" in error for error in self.errors(value)))

    def test_duplicate_step_id_is_rejected(self) -> None:
        value = self.base()
        value["steps"][1]["id"] = value["steps"][0]["id"]
        self.assertTrue(any("duplicate step id" in error for error in self.errors(value)))

    def test_invalid_concurrency_is_rejected(self) -> None:
        value = self.base()
        value["limits"]["max_parallel"] = 17
        self.assertTrue(any("max_parallel" in error for error in self.errors(value)))

    def test_unbounded_repeat_is_rejected(self) -> None:
        value = self.base()
        value["steps"][0]["repeat"] = {"until": "done"}
        self.assertTrue(any("max_rounds" in error for error in self.errors(value)))

    def test_undeclared_output_is_rejected(self) -> None:
        value = self.base()
        value["steps"][1]["instructions"] = "Use ${steps.plan.missing}."
        self.assertTrue(any("does not declare output missing" in error for error in self.errors(value)))

    def test_dependency_cycle_is_rejected(self) -> None:
        value = self.base()
        value["steps"][0]["needs"] = ["inspect-docs"]
        self.assertTrue(any("dependency cycle" in error for error in self.errors(value)))

    def test_overlapping_shared_writers_are_rejected(self) -> None:
        value = self.base()
        writer = deepcopy(value["steps"][3])
        writer["id"] = "second-writer"
        writer["needs"] = ["inspect-docs", "inspect-tests"]
        writer["worktree"] = "shared"
        value["steps"][3]["worktree"] = "shared"
        value["steps"].append(writer)
        self.assertTrue(any("unsafe parallel writers" in error for error in self.errors(value)))

    def test_read_only_agent_cannot_declare_write_scope(self) -> None:
        value = self.base()
        value["steps"][0]["write_scope"] = ["src/"]
        self.assertTrue(any("read-only agent" in error for error in self.errors(value)))

    def test_step_attempts_cannot_exceed_workflow_limit(self) -> None:
        value = self.base()
        value["steps"][0]["max_attempts"] = 3
        self.assertTrue(any("exceeds workflow limit" in error for error in self.errors(value)))

    def test_init_run_snapshots_workflow_and_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repository = Path(tmp) / "repo"
            repository.mkdir()
            subprocess.run(["git", "init", "-q", str(repository)], check=True)
            subprocess.run(["git", "-C", str(repository), "config", "user.email", "test@example.com"], check=True)
            subprocess.run(["git", "-C", str(repository), "config", "user.name", "Test"], check=True)
            (repository / "README.md").write_text("fixture\n", encoding="utf-8")
            subprocess.run(["git", "-C", str(repository), "add", "README.md"], check=True)
            subprocess.run(["git", "-C", str(repository), "commit", "-qm", "fixture"], check=True)
            run = workflow_module.initialize(FIXTURE, repository, "run-001")
            state = json.loads((run / "state.json").read_text(encoding="utf-8"))
            self.assertEqual(state["status"], "pending")
            self.assertEqual(state["workflow"], "workflow.yaml")
            self.assertEqual(set(state["steps"]), {step["id"] for step in self.base()["steps"]})
            self.assertTrue((run / "steps").is_dir())
            self.assertTrue((run / "evidence").is_dir())

    def test_pipeline_and_parallel_stage_ordering(self) -> None:
        value = self.base()
        state = self.state(value)
        self.assertEqual(workflow_module.ready_step_ids(value, state), ["plan"])
        workflow_module.start_step(value, state, "plan")
        workflow_module.finish_step(value, state, "plan", success=True)
        self.assertEqual(workflow_module.ready_step_ids(value, state), ["inspect-docs", "inspect-tests"])

    def test_parallel_stage_respects_workflow_limit(self) -> None:
        value = self.base()
        state = self.state(value)
        workflow_module.start_step(value, state, "plan")
        workflow_module.finish_step(value, state, "plan", success=True)
        value["limits"]["max_parallel"] = 1
        self.assertEqual(workflow_module.ready_step_ids(value, state), ["inspect-docs"])

    def test_failed_check_retries_then_exhausts(self) -> None:
        value = self.base()
        state = self.state(value)
        for dependency in ("plan", "inspect-docs", "inspect-tests", "implement"):
            state["steps"][dependency]["status"] = "complete"
        workflow_module.start_step(value, state, "checks")
        workflow_module.finish_step(value, state, "checks", success=False)
        self.assertEqual(state["steps"]["checks"]["status"], "pending")
        workflow_module.start_step(value, state, "checks")
        workflow_module.finish_step(value, state, "checks", success=False)
        self.assertEqual(state["status"], "failed")

    def test_approval_pauses_and_records_decision(self) -> None:
        value = self.base()
        state = self.state(value)
        for step_id in state["steps"]:
            if step_id != "approve-material-finding":
                state["steps"][step_id]["status"] = "complete"
        state["steps"]["review"]["outputs"] = {"material-finding": True, "findings": ["problem"]}
        workflow_module.start_step(value, state, "approve-material-finding")
        self.assertEqual(state["status"], "waiting_approval")
        workflow_module.decide_approval(value, state, "approve-material-finding", "repair")
        self.assertEqual(state["steps"]["approve-material-finding"]["decision"], "repair")
        self.assertEqual(state["status"], "blocked")
        self.assertIn("Revise", state["next_action"])

    def test_clean_review_skips_conditional_approval_and_completes(self) -> None:
        value = self.base()
        state = self.state(value)
        for step_id in state["steps"]:
            if step_id != "approve-material-finding":
                state["steps"][step_id]["status"] = "complete"
        state["steps"]["review"]["outputs"] = {"material-finding": False, "findings": []}
        self.assertEqual(workflow_module.ready_step_ids(value, state), [])
        self.assertEqual(state["steps"]["approve-material-finding"]["status"], "skipped")
        self.assertEqual(state["status"], "complete")

    def test_accepting_material_finding_completes_run(self) -> None:
        value = self.base()
        state = self.state(value)
        for step_id in state["steps"]:
            if step_id != "approve-material-finding":
                state["steps"][step_id]["status"] = "complete"
        state["steps"]["review"]["outputs"] = {"material-finding": True, "findings": ["accepted risk"]}
        workflow_module.start_step(value, state, "approve-material-finding")
        workflow_module.decide_approval(value, state, "approve-material-finding", "accept")
        self.assertEqual(state["status"], "complete")

    def test_attempt_exhaustion_blocks_another_start(self) -> None:
        value = self.base()
        state = self.state(value)
        for dependency in ("plan", "inspect-docs", "inspect-tests"):
            state["steps"][dependency]["status"] = "complete"
        state["steps"]["implement"]["attempts"] = 2
        with self.assertRaises(workflow_module.WorkflowError):
            workflow_module.start_step(value, state, "implement")
        self.assertEqual(state["status"], "failed")

    def test_interrupted_run_recovers_only_with_attempts_remaining(self) -> None:
        value = self.base()
        state = self.state(value)
        state["steps"]["plan"] = {"status": "running", "attempts": 1}
        exhausted = workflow_module.recover_interrupted(value, state)
        self.assertEqual(exhausted, ["plan"])
        self.assertEqual(state["status"], "failed")

    def test_review_repair_repeat_is_bounded_and_stops_without_progress(self) -> None:
        step = {"repeat": {"until": "review clean", "max_rounds": 2, "stop_on_no_progress": True}}
        self.assertTrue(workflow_module.should_repeat(step, rounds=1, until_met=False, progress=True))
        self.assertFalse(workflow_module.should_repeat(step, rounds=2, until_met=False, progress=True))
        self.assertFalse(workflow_module.should_repeat(step, rounds=1, until_met=False, progress=False))


if __name__ == "__main__":
    unittest.main()
