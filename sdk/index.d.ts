export type ForkTurns = "none" | "all" | { last: number };
export type ContextSource =
  | { type: "file"; path: string }
  | { type: "range"; path: string; start: number; end: number }
  | { type: "diff"; from: string; to: string; paths?: string[] }
  | { type: "revision"; revision: string; path: string }
  | { type: "dependency_output"; step: string; output: string };

export interface CommonStep {
  id: string;
  needs?: string[];
  max_attempts?: number;
  write_scope?: string[];
}

export interface AgentStep extends CommonStep {
  prompt: string;
  model: string;
  reasoning_effort?: "none" | "minimal" | "low" | "medium" | "high" | "xhigh";
  service_tier?: string;
  fork_turns?: ForkTurns;
  context?: ContextSource[];
  outputs?: string[];
  allow_delegation?: boolean;
}

export interface CheckStep extends CommonStep {
  command: string[];
  cwd?: string;
  timeout_ms?: number;
}

export interface ApprovalStep extends CommonStep {
  prompt: string;
  choices: string[];
}

export interface RepeatPolicy {
  max_rounds: number;
  until_output: string;
  equals?: unknown;
  stop_on_no_progress?: boolean;
}

export interface Workflow {
  name: string;
  description?: string;
  max_parallel?: number;
  steps: StepNode[];
}

export type StepNode = Readonly<Record<string, unknown>>;
export declare function workflow(value: Workflow): Workflow;
export declare const defineWorkflow: typeof workflow;
export declare function agent(value: AgentStep): StepNode;
export declare function check(value: CheckStep): StepNode;
export declare function approval(value: ApprovalStep): StepNode;
export declare function parallel(steps: StepNode[]): StepNode;
export declare function pipeline(steps: StepNode[]): StepNode;
export declare function worktree(step: StepNode, policy: "shared" | "isolated"): StepNode;
export declare function repeat(step: StepNode, policy: RepeatPolicy): StepNode;
export declare function ref(path: `steps.${string}.outputs.${string}`): string;
