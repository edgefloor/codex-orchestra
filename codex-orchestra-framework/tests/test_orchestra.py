from __future__ import annotations

import json
import shutil
import sqlite3
import sys
import tempfile
import unittest
from datetime import timedelta
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "tools"))

import orchestra  # noqa: E402


class OrchestraTestCase(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name) / "repo"
        self.root.mkdir()
        shutil.copy2(REPO / "AGENTS.md", self.root / "AGENTS.md")
        shutil.copytree(REPO / ".codex", self.root / ".codex")
        shutil.copytree(REPO / ".agents", self.root / ".agents")
        shutil.copytree(
            REPO / ".orchestra",
            self.root / ".orchestra",
            ignore=shutil.ignore_patterns("orchestra.db", "orchestra.db-*"),
        )
        shutil.copy2(REPO / ".gitignore", self.root / ".gitignore")
        shutil.copy2(REPO / "README.md", self.root / "README.md")
        shutil.copy2(REPO / "DESIGN.md", self.root / "DESIGN.md")
        orchestra.initialize(self.root, "test")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def add_task(self, task_id: str = "task-1", write_domain: tuple[str, ...] = ("src/a.py",)) -> dict:
        return orchestra.add_task(
            self.root,
            project_id="test",
            task_id=task_id,
            title=f"Task {task_id}",
            workstream="platform",
            milestone="M1",
            context_ref=f"artifact://capsules/{task_id}",
            recommended_agent_type="worker_terra",
            reports_to="tl-platform",
            write_domain=write_domain,
            acceptance=("behavior passes",),
            actor="tl-platform",
        )

    def test_doctor_passes_framework(self) -> None:
        errors, warnings = orchestra.doctor(REPO)
        self.assertEqual(errors, [], f"doctor errors: {errors}; warnings: {warnings}")


    def test_doctor_requires_multi_agent_v2_and_visible_routing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "framework"
            shutil.copytree(
                REPO,
                copy,
                ignore=shutil.ignore_patterns("orchestra.db", "orchestra.db-*", "__pycache__", ".pytest_cache"),
            )
            config_path = copy / ".codex/config.toml"
            config = config_path.read_text(encoding="utf-8")
            config = config.replace("enabled = true", "enabled = false", 1)
            config = config.replace("hide_spawn_agent_metadata = false", "hide_spawn_agent_metadata = true", 1)
            config_path.write_text(config, encoding="utf-8")
            errors, _ = orchestra.doctor(copy)
            joined = "\n".join(errors)
            self.assertIn("must enable features.multi_agent_v2.enabled = true", joined)
            self.assertIn("hide_spawn_agent_metadata must be false", joined)

    def test_doctor_rejects_ultra_default_on_worker(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "framework"
            shutil.copytree(
                REPO,
                copy,
                ignore=shutil.ignore_patterns("orchestra.db", "orchestra.db-*", "__pycache__", ".pytest_cache"),
            )
            worker = copy / ".codex/agents/worker_terra.toml"
            content = worker.read_text(encoding="utf-8").replace(
                'model_reasoning_effort = "medium"', 'model_reasoning_effort = "ultra"', 1
            )
            worker.write_text(content, encoding="utf-8")
            errors, _ = orchestra.doctor(copy)
            self.assertTrue(any("multi-agent orchestration effort" in error for error in errors))

    def test_schemas_and_direct_templates_validate(self) -> None:
        try:
            import jsonschema
            import yaml
        except ImportError as exc:  # pragma: no cover - optional development dependency
            self.skipTest(f"schema validation dependencies unavailable: {exc}")

        schemas = REPO / ".orchestra" / "schemas"
        templates = REPO / ".orchestra" / "templates"
        pairs = {
            "TACTICAL_PLAN.yaml": "tactical-plan.schema.json",
            "STAFFING_DECISION.yaml": "staffing-decision.schema.json",
            "PORTFOLIO_CHECKPOINT.yaml": "portfolio-checkpoint.schema.json",
            "CONTEXT_CAPSULE.yaml": "context-capsule.schema.json",
            "HIRE_REQUEST.yaml": "hire-request.schema.json",
            "ROLE_CARD.yaml": "role-card.schema.json",
            "DELEGATION_PERMIT.yaml": "delegation-permit.schema.json",
            "BRANCH_REPORT.yaml": "branch-report.schema.json",
            "RESULT_ENVELOPE.json": "result-envelope.schema.json",
            "COLLABORATION_COMMAND.json": "collaboration-command.schema.json",
        }
        for template_name, schema_name in pairs.items():
            schema = json.loads((schemas / schema_name).read_text(encoding="utf-8"))
            jsonschema.Draft202012Validator.check_schema(schema)
            template_path = templates / template_name
            instance = (
                json.loads(template_path.read_text(encoding="utf-8"))
                if template_path.suffix == ".json"
                else yaml.safe_load(template_path.read_text(encoding="utf-8"))
            )
            validator = jsonschema.Draft202012Validator(
                schema, format_checker=jsonschema.Draft202012Validator.FORMAT_CHECKER
            )
            errors = sorted(validator.iter_errors(instance), key=lambda error: list(error.path))
            self.assertEqual(errors, [], f"{template_name} failed {schema_name}: {errors}")

    def test_collaboration_schema_handles_v2_full_history_fork(self) -> None:
        try:
            import jsonschema
        except ImportError as exc:  # pragma: no cover - optional development dependency
            self.skipTest(f"schema validation dependency unavailable: {exc}")

        schema = json.loads(
            (REPO / ".orchestra/schemas/collaboration-command.schema.json").read_text(encoding="utf-8")
        )
        validator = jsonschema.Draft202012Validator(
            schema, format_checker=jsonschema.Draft202012Validator.FORMAT_CHECKER
        )
        regular = {
            "command_id": "cmd-regular",
            "action": "spawn",
            "actor_path": "/root/manager",
            "actor_role": "manager",
            "parent_path": "/root/manager",
            "task_name": "tl_platform",
            "agent_type": "team_leader",
            "model": "gpt-5.6-sol",
            "reasoning_effort": "high",
            "fork_turns": "none",
            "message_ref": "artifact://messages/tl-platform",
            "delegation_permit_ref": "artifact://permits/tl-platform",
            "budget_reservation": {
                "thread_slots": 1, "max_children": 3, "token_or_time_class": "standard"
            },
            "reason": "Appoint the approved Team Leader.",
            "created_at": "2026-07-14T12:00:00Z",
        }
        self.assertEqual(list(validator.iter_errors(regular)), [])

        full_history = {
            key: value
            for key, value in regular.items()
            if key not in {"agent_type", "model", "reasoning_effort"}
        }
        full_history["fork_turns"] = "all"
        full_history["exception_authorization_ref"] = "decision://full-history-fork-1"
        self.assertEqual(list(validator.iter_errors(full_history)), [])

        invalid = dict(full_history)
        invalid["model"] = "gpt-5.6-sol"
        self.assertNotEqual(list(validator.iter_errors(invalid)), [])

    def test_lease_is_exclusive(self) -> None:
        self.add_task()
        first = orchestra.claim_task(self.root, project_id="test", task_id="task-1", agent_id="worker-a", ttl_seconds=900)
        self.assertTrue(first["lease_token"].startswith("lease-"))
        with self.assertRaises(orchestra.OrchestraError):
            orchestra.claim_task(self.root, project_id="test", task_id="task-1", agent_id="worker-b", ttl_seconds=900)

    def test_concurrent_overlapping_write_domains_are_rejected(self) -> None:
        self.add_task("task-a", ("src/shared.py",))
        self.add_task("task-b", ("src/shared.py", "src/other.py"))
        orchestra.claim_task(self.root, project_id="test", task_id="task-a", agent_id="worker-a", ttl_seconds=900)
        with self.assertRaisesRegex(orchestra.OrchestraError, "Write-domain overlap"):
            orchestra.claim_task(self.root, project_id="test", task_id="task-b", agent_id="worker-b", ttl_seconds=900)


    def test_directory_write_domain_overlap_is_rejected(self) -> None:
        self.add_task("task-dir", ("src",))
        self.add_task("task-file", ("src/module/file.py",))
        orchestra.claim_task(self.root, project_id="test", task_id="task-dir", agent_id="worker-a", ttl_seconds=900)
        with self.assertRaisesRegex(orchestra.OrchestraError, "Write-domain overlap"):
            orchestra.claim_task(self.root, project_id="test", task_id="task-file", agent_id="worker-b", ttl_seconds=900)

    def test_manager_cannot_be_assigned_execution_task(self) -> None:
        with self.assertRaisesRegex(orchestra.OrchestraError, "cannot be assigned an execution task"):
            orchestra.add_task(
                self.root,
                project_id="test",
                task_id="manager-code",
                title="Write code",
                workstream="platform",
                milestone="M1",
                context_ref="artifact://capsules/manager-code",
                recommended_agent_type="manager",
                reports_to=None,
                write_domain=("src",),
                acceptance=("code exists",),
                actor="conductor",
            )

    def test_reporting_cycle_is_rejected(self) -> None:
        orchestra.register_agent(
            self.root, project_id="test", agent_id="manager_1", role="manager", reports_to=None,
            status="active", config_ref=".codex/agents/manager.toml", role_card_ref=None, thread_id=None, actor="conductor"
        )
        orchestra.register_agent(
            self.root, project_id="test", agent_id="tl_1", role="team_leader", reports_to="manager_1",
            physical_parent_id="manager_1", task_name="tl_1", delegation_permit_ref="artifact://permits/tl-1",
            status="active", config_ref=".codex/agents/team_leader.toml", role_card_ref=None, thread_id=None, actor="manager_1"
        )
        with self.assertRaisesRegex(orchestra.OrchestraError, "Reporting cycle"):
            orchestra.register_agent(
                self.root, project_id="test", agent_id="manager_1", role="manager", reports_to="tl_1",
                status="active", config_ref=".codex/agents/manager.toml", role_card_ref=None, thread_id=None, actor="conductor"
            )

    def test_bounded_hierarchy_and_parent_paths(self) -> None:
        manager = orchestra.register_agent(
            self.root, project_id="test", agent_id="manager", role="manager", reports_to=None,
            status="active", config_ref=".codex/agents/manager.toml", role_card_ref=None, thread_id=None, actor="conductor"
        )
        self.assertEqual(manager["agent_path"], "/root/manager")
        leader = orchestra.register_agent(
            self.root, project_id="test", agent_id="tl_platform", role="team_leader", reports_to="manager",
            physical_parent_id="manager", delegation_permit_ref="artifact://permits/tl-platform",
            status="active", config_ref=".codex/agents/team_leader.toml", role_card_ref=None, thread_id=None, actor="manager"
        )
        self.assertEqual(leader["agent_path"], "/root/manager/tl_platform")
        worker = orchestra.register_agent(
            self.root, project_id="test", agent_id="worker_a", role="worker_terra", reports_to="tl_platform",
            physical_parent_id="tl_platform", task_name="w_task_a1", delegation_permit_ref="artifact://permits/wave-1",
            status="working", config_ref=".codex/agents/worker_terra.toml", role_card_ref="artifact://roles/worker-a", thread_id=None, actor="tl_platform"
        )
        self.assertEqual(worker["agent_path"], "/root/manager/tl_platform/w_task_a1")
        self.assertEqual(worker["depth"], 3)

    def test_manager_cannot_parent_worker(self) -> None:
        orchestra.register_agent(
            self.root, project_id="test", agent_id="manager", role="manager", reports_to=None,
            status="active", config_ref=".codex/agents/manager.toml", role_card_ref=None, thread_id=None, actor="conductor"
        )
        with self.assertRaisesRegex(orchestra.OrchestraError, "manager may not spawn child role worker_terra"):
            orchestra.register_agent(
                self.root, project_id="test", agent_id="worker_a", role="worker_terra", reports_to="manager",
                physical_parent_id="manager", delegation_permit_ref="artifact://permits/invalid",
                status="working", config_ref=".codex/agents/worker_terra.toml", role_card_ref="artifact://roles/a", thread_id=None, actor="manager"
            )

    def test_nested_registration_requires_permit(self) -> None:
        orchestra.register_agent(
            self.root, project_id="test", agent_id="manager", role="manager", reports_to=None,
            status="active", config_ref=".codex/agents/manager.toml", role_card_ref=None, thread_id=None, actor="conductor"
        )
        with self.assertRaisesRegex(orchestra.OrchestraError, "delegation_permit_ref"):
            orchestra.register_agent(
                self.root, project_id="test", agent_id="tl_platform", role="team_leader", reports_to="manager",
                physical_parent_id="manager", status="active", config_ref=".codex/agents/team_leader.toml",
                role_card_ref=None, thread_id=None, actor="manager"
            )

    def test_stale_result_cannot_transition_task(self) -> None:
        self.add_task()
        lease = orchestra.claim_task(self.root, project_id="test", task_id="task-1", agent_id="worker-a", ttl_seconds=900)
        brief = self.root / ".orchestra/charter/BRIEF.md"
        brief.write_text(brief.read_text(encoding="utf-8") + "\nmaterial revision\n", encoding="utf-8")
        orchestra.record_digest(self.root, project_id="test", actor="consultant")
        old_digest = self._task("task-1")["alignment_digest"]
        with self.assertRaisesRegex(orchestra.OrchestraError, "stale"):
            orchestra.submit_task(
                self.root,
                project_id="test",
                task_id="task-1",
                lease_token=lease["lease_token"],
                result_ref="artifact://result-1",
                alignment_digest=old_digest,
            )
        self.assertEqual(self._task("task-1")["status"], "stale")
        self.assertIsNone(self._lease("task-1"))

    def test_expired_lease_is_reconciled(self) -> None:
        self.add_task()
        lease = orchestra.claim_task(self.root, project_id="test", task_id="task-1", agent_id="worker-a", ttl_seconds=900)
        expired = orchestra.iso(orchestra.utc_now() - timedelta(seconds=1))
        conn = sqlite3.connect(orchestra.db_path(self.root))
        try:
            conn.execute("UPDATE leases SET expires_at = ? WHERE task_id = ?", (expired, "task-1"))
            conn.commit()
        finally:
            conn.close()
        result = orchestra.reconcile(self.root, project_id="test")
        self.assertEqual(result["expired_count"], 1)
        self.assertEqual(self._task("task-1")["status"], "ready")
        attempt = self._attempt(lease["attempt_id"])
        self.assertEqual(attempt["status"], "expired")

    def test_submit_then_team_leader_review(self) -> None:
        self.add_task()
        lease = orchestra.claim_task(self.root, project_id="test", task_id="task-1", agent_id="worker-a", ttl_seconds=900)
        project = self._project()
        submitted = orchestra.submit_task(
            self.root,
            project_id="test",
            task_id="task-1",
            lease_token=lease["lease_token"],
            result_ref="artifact://result-1",
            alignment_digest=project["alignment_digest"],
        )
        self.assertEqual(submitted["status"], "in_review")
        reviewed = orchestra.decide_task_review(
            self.root,
            project_id="test",
            task_id="task-1",
            actor="tl-platform",
            decision="integrated",
            reason="acceptance and interface evidence pass",
        )
        self.assertEqual(reviewed["status"], "integrated")

    def test_phase_compare_and_transition_rules(self) -> None:
        result = orchestra.set_phase(
            self.root,
            project_id="test",
            target="GROUNDING",
            actor="conductor",
            gate_ref="event://operator-intent",
            expected_current="NEW",
        )
        self.assertEqual(result["to"], "GROUNDING")
        with self.assertRaisesRegex(orchestra.OrchestraError, "Illegal phase transition"):
            orchestra.set_phase(
                self.root,
                project_id="test",
                target="STAFFING",
                actor="manager",
                gate_ref="artifact://invalid-skip",
            )

    def test_event_idempotency(self) -> None:
        first = orchestra.emit_manual_event(
            self.root,
            project_id="test",
            event_type="decision.recorded",
            actor="manager",
            payload={"decision": "x"},
            idempotency_key="decision-x",
        )
        second = orchestra.emit_manual_event(
            self.root,
            project_id="test",
            event_type="decision.recorded",
            actor="manager",
            payload={"decision": "x"},
            idempotency_key="decision-x",
        )
        self.assertEqual(first["event_id"], second["event_id"])
        self.assertTrue(second["duplicate"])

    def _row(self, sql: str, params: tuple) -> dict | None:
        conn = sqlite3.connect(orchestra.db_path(self.root))
        conn.row_factory = sqlite3.Row
        try:
            row = conn.execute(sql, params).fetchone()
            return dict(row) if row else None
        finally:
            conn.close()

    def _project(self) -> dict:
        result = self._row("SELECT * FROM projects WHERE project_id = ?", ("test",))
        assert result is not None
        return result

    def _task(self, task_id: str) -> dict:
        result = self._row("SELECT * FROM tasks WHERE task_id = ?", (task_id,))
        assert result is not None
        return result

    def _lease(self, task_id: str) -> dict | None:
        return self._row("SELECT * FROM leases WHERE task_id = ?", (task_id,))

    def _attempt(self, attempt_id: str) -> dict:
        result = self._row("SELECT * FROM attempts WHERE attempt_id = ?", (attempt_id,))
        assert result is not None
        return result


if __name__ == "__main__":
    unittest.main()
