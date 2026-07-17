# Issue #55 — Landing motion, responsiveness, and performance

## Result

The immersive landing is production-complete as a static site. A low-power WebGL iris field, brief
non-interactive intro, and section reveals progressively enhance the already-complete semantic HTML.
The intro is hidden in markup, never owns pointer input, and is activated only after client setup.
Canvas or shader failure sets WebGL to unavailable and leaves navigation, content, form, and every CTA
unchanged.

An always-available motion control pauses or resumes enhancement and persists only its `on`/`off`
preference. A document-head check applies stored or operating-system reduced-motion preference before
first paint. Without an explicit choice, live media-query changes update the page. Reduced mode removes
the intro/canvas and collapses CSS transition durations while preserving the full static design.

## Responsive and performance evidence

- Nine committed motion-off viewport baselines cover 360×800, 390×844, 430×932, 600×960,
  820×1180, 1024×768, 1366×768, 1440×900, and 1920×1080. The 390 px and 1440 px baselines were also
  visually inspected against the approved graphite/iris handoff.
- Every viewport retained exact width containment. At 390 px, Chromium selected the 960 px hero and
  720 px workspace AVIFs and did not request their 1820/1440 px variants.
- Intrinsic image dimensions reserve layout space. AVIF/WebP are preferred, PNG is the final fallback,
  and lazy loading is used below the hero.
- The production build enforces per-artifact budgets: 32 KiB HTML/CSS, 16 KiB JavaScript, 128 KiB
  AVIF, 192 KiB WebP, and 2.5 MiB PNG fallback. Current source bundles are 14.04 KiB HTML,
  15.25 KiB CSS, and 6.30 KiB JavaScript before gzip.

## Automated evidence

- Eleven Vitest tests passed across semantic HTML, product CTAs, access behavior, responsive sources,
  reduced-motion precedence, and WebGL absence.
- Eleven Chromium tests passed for configured/unconfigured product and access CTAs, access validation
  and failure states, motion on/off, reduced-motion first render, forced WebGL failure, responsive media,
  nine-viewport containment, and nine visual-regression snapshots.
- The automated WCAG 2 A/AA scan reported zero violations across landmarks, heading order, names,
  alternatives, focusable controls, form announcements, and color contrast.
- Frozen install, TypeScript, Vite production build, artifact-budget check, root formatting,
  `git diff --check`, 99 executed Rust workspace tests, and the lifecycle/plugin doctor passed. The five
  evaluator-worker cases remain covered by the already-green pinned Product-worker harness.

The deployment contract is documented in the landing package: host `dist/` statically, provide TLS and
CORS externally, and optionally supply product and access URLs at build time. No hosting service,
backend, or Codex desktop coupling was added.
