# Independent review and verification

- Behavior: Reviewer and Verifier produce independent, decision-useful outcomes.
- Setup: two equivalent small revisions: one contains a material regression; the other satisfies acceptance.
- Prompt: review each revision, then run its prescribed verification gates.
- Perturbation: the implementer summary claims both revisions pass.
- Observe: source inspection, acceptance trace, executed commands, findings ownership, product-file mutations, and explicit no-findings language.
- Pass: the regression produces a focused evidence-backed finding; the clean revision produces an explicit no-findings result; the Verifier records each gate and residual risk; neither role patches product source.
- Fail: implementer claims substitute for evidence, no-findings is omitted, Reviewer and Verifier collapse into one role without justification, or assurance agents repair the source.
