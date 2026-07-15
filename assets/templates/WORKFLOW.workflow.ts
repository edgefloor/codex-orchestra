import { agent, check, pipeline, ref, workflow } from "@codex-orchestra/workflow";

export default workflow({
  name: "native-slice",
  description: "One native V2 agent followed by a sandboxed check.",
  inputs: {
    request: { type: "string" },
  },
  max_parallel: 2,
  steps: [
    pipeline([
      agent({
        id: "implement",
        prompt: ref("inputs.request"),
        model: "gpt-5.4",
        reasoning_effort: "high",
        fork_turns: "none",
        context: [
          { type: "file", path: "CONTEXT.md" },
          { type: "input", input: "request" },
        ],
        outputs: ["summary", "changed_files"],
        write_scope: ["src/"],
      }),
      check({
        id: "tests",
        command: ["cargo", "test", "--workspace"],
        timeout_ms: 300000,
      }),
    ]),
  ],
});
