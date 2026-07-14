from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

PLUGIN = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("lifecycle", PLUGIN / "scripts/lifecycle.py")
assert SPEC and SPEC.loader
lifecycle = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = lifecycle
SPEC.loader.exec_module(lifecycle)


class ScaffoldTests(unittest.TestCase):
    def test_uv_tooling_is_locked_and_non_package(self) -> None:
        pyproject = tomllib.loads((PLUGIN / "pyproject.toml").read_text(encoding="utf-8"))
        manifest = json.loads((PLUGIN / ".codex-plugin/plugin.json").read_text(encoding="utf-8"))
        self.assertFalse(pyproject["tool"]["uv"]["package"])
        self.assertEqual(pyproject["project"]["version"], manifest["version"])
        self.assertIn("pyyaml>=6.0.2", pyproject["dependency-groups"]["dev"])
        self.assertTrue((PLUGIN / "uv.lock").is_file())

    def test_manifest_is_native_and_namespaced(self) -> None:
        manifest = json.loads((PLUGIN / ".codex-plugin/plugin.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["name"], PLUGIN.name)
        self.assertEqual(manifest["skills"], "./skills/")
        self.assertNotIn("mcpServers", manifest)
        self.assertNotIn("apps", manifest)
        self.assertNotIn("hooks", manifest)

    def test_plugin_layout_accepts_source_and_versioned_cache_only(self) -> None:
        manifest = json.loads((PLUGIN / ".codex-plugin/plugin.json").read_text(encoding="utf-8"))
        self.assertTrue(lifecycle.plugin_layout_matches_manifest(PLUGIN, manifest))
        with tempfile.TemporaryDirectory() as tmp:
            cache = Path(tmp) / manifest["name"] / manifest["version"]
            shutil.copytree(
                PLUGIN,
                cache,
                ignore=shutil.ignore_patterns(".codex", ".git", ".venv", "__pycache__"),
            )
            self.assertTrue(lifecycle.plugin_layout_matches_manifest(cache, manifest))
            self.assertFalse(lifecycle.plugin_layout_matches_manifest(Path(tmp) / "unrelated", manifest))
            self.assertFalse(
                lifecycle.plugin_layout_matches_manifest(Path(tmp) / "other-plugin" / manifest["version"], manifest)
            )
            self.assertFalse(
                lifecycle.plugin_layout_matches_manifest(Path(tmp) / manifest["name"] / "other-version", manifest)
            )

    def test_skill_surface_is_discoverable(self) -> None:
        skills = sorted((PLUGIN / "skills").glob("*/SKILL.md"))
        self.assertGreaterEqual(len(skills), 12)
        self.assertTrue(any(path.parent.name == "orchestrate" for path in skills))
        for path in skills:
            text = path.read_text(encoding="utf-8")
            self.assertTrue(text.startswith("---\nname: "), path)
            self.assertIn("\ndescription: ", text, path)

    def test_configuration_templates_parse_and_use_stable_feature(self) -> None:
        for path in (PLUGIN / "config").rglob("*.toml"):
            with self.subTest(path=path):
                tomllib.loads(path.read_text(encoding="utf-8"))
        project = tomllib.loads((PLUGIN / "config/project.toml").read_text(encoding="utf-8"))
        self.assertTrue(project["features"]["multi_agent"])
        self.assertNotIn("multi_agent_v2", project["features"])
        self.assertNotIn("model", project)

    def test_mutable_state_is_not_bundled_in_plugin(self) -> None:
        state_root = PLUGIN / ".codex/orchestra"
        if (PLUGIN / ".git").exists():
            tracked = subprocess.run(
                ["git", "-C", str(PLUGIN), "ls-files", "--", ".codex/orchestra"],
                check=True,
                capture_output=True,
                text=True,
            )
            self.assertEqual(tracked.stdout, "")
        else:
            self.assertFalse(state_root.exists())
        policy = (PLUGIN / "assets/policies/orchestration.yaml").read_text(encoding="utf-8")
        self.assertIn("state_root: .codex/orchestra", policy)

    def test_architecture_decisions_are_grounded(self) -> None:
        context = (PLUGIN / "CONTEXT.md").read_text(encoding="utf-8")
        for term in ("Operator", "Grounding", "Context Capsule", "Join Owner", "Reviewer", "Verifier", "Gate", "Drift", "Handoff", "Attempt"):
            self.assertIn(f"**{term}**", context)
        adrs = sorted((PLUGIN / "docs/adr").glob("[0-9][0-9][0-9][0-9]-*.md"))
        self.assertEqual(len(adrs), 8)
        combined = "\n".join(path.read_text(encoding="utf-8") for path in adrs)
        for decision in ("Codex-native", "optional configuration", "not a fixed organization chart", "self-hosting", "Context Capsules", "isolate mutation", "risk", "recover"):
            self.assertIn(decision, combined)
        structure = (PLUGIN / "docs/REPOSITORY-STRUCTURE.md").read_text(encoding="utf-8")
        self.assertIn("## Migration outcome", structure)

    def test_repository_cutover_is_complete(self) -> None:
        self.assertTrue((PLUGIN / "config/project.toml").is_file())
        self.assertTrue((PLUGIN / "config/orchestra.config.toml").is_file())
        self.assertEqual(len(list((PLUGIN / "config/agents").glob("*.toml"))), 5)
        for retired in (
            "codex-orchestra",
            "codex-orchestra-framework",
            "SCAFFOLDING-PLAN.md",
            "README.txt",
            "SHA256SUMS.txt",
            "hooks",
            "docs/MIGRATION-INVENTORY.md",
        ):
            self.assertFalse((PLUGIN / retired).exists(), retired)

    def test_interactive_verification_separates_automated_and_human_evidence(self) -> None:
        runbook = (PLUGIN / "docs/INTERACTIVE-VERIFICATION.md").read_text(encoding="utf-8")
        record = (PLUGIN / "assets/templates/INTERACTIVE-VERIFICATION.md").read_text(encoding="utf-8")
        baseline = (PLUGIN / "docs/verification/2026-07-14-interactive-baseline.md").read_text(encoding="utf-8")
        for stage in range(5):
            self.assertIn(f"## Stage {stage}", runbook)
        self.assertIn("Human-only evidence", runbook)
        self.assertIn("Automated evidence", record)
        self.assertIn("Human UI evidence", record)
        self.assertIn("pending", record)
        self.assertIn("Verdict: `pending`", baseline)

    def test_permanent_behavioral_scenarios_cover_required_risks(self) -> None:
        scenario_root = PLUGIN / "evals/scenarios"
        expected = {
            "bounded-workstreams.md",
            "independent-assurance.md",
            "interruption-recovery.md",
            "large-repository-context.md",
            "write-conflict-isolation.md",
            "semantic-retry.md",
            "risk-derived-assurance.md",
            "self-hosting-promotion.md",
        }
        self.assertEqual({path.name for path in scenario_root.glob("*.md")}, expected)
        for path in scenario_root.glob("*.md"):
            text = path.read_text(encoding="utf-8")
            for field in ("- Behavior:", "- Setup:", "- Prompt:", "- Perturbation:", "- Observe:", "- Pass:", "- Fail:"):
                self.assertIn(field, text, path)

    def test_project_install_is_preview_first_and_initializes_state(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "repo"
            target.mkdir()
            files = lifecycle.desired_project(target)
            state = target / ".codex/orchestra/install-state.json"
            self.assertEqual(lifecycle.install(files, target, state, apply=False), 0)
            self.assertFalse((target / ".codex").exists())
            lifecycle.init_project_state(target, apply=True)
            self.assertEqual(lifecycle.install(files, target, state, apply=True), 0)
            self.assertTrue((target / ".codex/config.toml").is_file())
            for name in lifecycle.STATE_DIRS:
                self.assertTrue((target / ".codex/orchestra" / name).is_dir())

    def test_install_refuses_existing_conflict(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "repo"
            (target / ".codex").mkdir(parents=True)
            original = "model = \"user-owned\"\n"
            (target / ".codex/config.toml").write_text(original, encoding="utf-8")
            result = lifecycle.install(
                lifecycle.desired_project(target),
                target,
                target / ".codex/orchestra/install-state.json",
                apply=True,
            )
            self.assertEqual(result, 2)
            self.assertEqual((target / ".codex/config.toml").read_text(encoding="utf-8"), original)

    def test_uninstall_preserves_modified_files_and_run_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "repo"
            target.mkdir()
            lifecycle.init_project_state(target, apply=True)
            state = target / ".codex/orchestra/install-state.json"
            self.assertEqual(lifecycle.install(lifecycle.desired_project(target), target, state, apply=True), 0)
            config = target / ".codex/config.toml"
            config.write_text(config.read_text(encoding="utf-8") + "# local\n", encoding="utf-8")
            artifact = target / ".codex/orchestra/results/keep.md"
            artifact.write_text("evidence\n", encoding="utf-8")
            self.assertEqual(lifecycle.uninstall(target, apply=True), 0)
            self.assertTrue(config.exists())
            self.assertTrue(artifact.exists())

    def test_exact_preexisting_file_is_not_claimed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "repo"
            (target / ".codex").mkdir(parents=True)
            source = PLUGIN / "config/project.toml"
            destination = target / ".codex/config.toml"
            destination.write_bytes(source.read_bytes())
            state_path = target / ".codex/orchestra/install-state.json"
            self.assertEqual(lifecycle.install(lifecycle.desired_project(target), target, state_path, apply=True), 0)
            state = json.loads(state_path.read_text(encoding="utf-8"))
            self.assertNotIn(".codex/config.toml", state["managed"])

    def test_profile_and_global_default_have_distinct_config_targets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            profile = lifecycle.desired_global(home, default=False)
            default = lifecycle.desired_global(home, default=True)
            self.assertEqual(profile[0].target, home / "orchestra.config.toml")
            self.assertEqual(default[0].target, home / "config.toml")
            (home / "config.toml").write_text("model = \"user-owned\"\n", encoding="utf-8")
            self.assertEqual(
                lifecycle.install(default, home, home / "orchestra-install-state.json", apply=True),
                2,
            )
            self.assertEqual((home / "config.toml").read_text(encoding="utf-8"), "model = \"user-owned\"\n")

    def test_upgrade_snapshot_and_rollback_are_reversible(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            root.mkdir()
            source = Path(tmp) / "source.toml"
            source.write_text("value = 1\n", encoding="utf-8")
            target = root / ".codex/config.toml"
            state_path = root / ".codex/orchestra/install-state.json"
            desired = [lifecycle.DesiredFile(source, target)]
            self.assertEqual(lifecycle.install(desired, root, state_path, apply=True), 0)
            source.write_text("value = 2\n", encoding="utf-8")
            self.assertEqual(lifecycle.install(desired, root, state_path, apply=True, upgrade=True), 0)
            self.assertEqual(target.read_text(encoding="utf-8"), "value = 2\n")
            self.assertEqual(lifecycle.rollback(root, apply=True), 0)
            self.assertEqual(target.read_text(encoding="utf-8"), "value = 1\n")

    def test_rollback_removes_files_introduced_by_upgrade(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            root.mkdir()
            state_path = root / ".codex/orchestra/install-state.json"
            original_source = Path(tmp) / "one.toml"
            original_source.write_text("value = 1\n", encoding="utf-8")
            original = lifecycle.DesiredFile(original_source, root / ".codex/one.toml")
            self.assertEqual(lifecycle.install([original], root, state_path, apply=True), 0)
            new_source = Path(tmp) / "two.toml"
            new_source.write_text("value = 2\n", encoding="utf-8")
            introduced = lifecycle.DesiredFile(new_source, root / ".codex/two.toml")
            self.assertEqual(lifecycle.install([original, introduced], root, state_path, apply=True, upgrade=True), 0)
            self.assertTrue(introduced.target.exists())
            self.assertEqual(lifecycle.rollback(root, apply=True), 0)
            self.assertFalse(introduced.target.exists())


if __name__ == "__main__":
    unittest.main()
