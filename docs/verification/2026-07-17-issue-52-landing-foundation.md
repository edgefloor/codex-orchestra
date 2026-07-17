# Issue #52 — Immersive landing foundation and hero

## Result

The approved immersive landing now ships as an independent static vanilla TypeScript/Vite package.
It uses the source handoff's graphite surfaces, restrained iris interaction color, Instrument Sans
display/body typography, Commit Mono operational typography, monolith hero artwork, and exact hero
message: “Control the work, not the agents.” It neither embeds nor depends on the Codex product.

The semantic HTML owns the complete navigation, hero, product fallback, access fallback, and footer.
JavaScript only resolves the optional `VITE_PRODUCT_URL`; without JavaScript or configuration, every
CTA still reaches a useful in-page destination. Intro, WebGL, reveal motion, and motion controls remain
deliberately absent until issue #55, so enhancement failure cannot hide the initial experience.

## Accessibility and responsive evidence

- The rendered accessibility tree exposes one primary navigation, one level-one heading, labelled
  hero/product/access regions, two useful CTA destinations, a skip link, and footer navigation.
- Focus-visible treatment is explicit. Buttons remain links with stable destinations before client
  code runs; decorative hero artwork has an empty alt value and intrinsic dimensions.
- Browser measurements passed at 360×800, 390×844, 430×932, 600×960, 820×1180, 1024×768,
  1366×768, 1440×900, and 1920×1080. At every size, document and body scroll width exactly matched
  the viewport, the hero filled at least one viewport, and the heading and action region remained
  rendered.
- A local browser interaction followed “View the product workspace” to `#product`, and the rendered
  page emitted no console errors or warnings.

## Automated evidence

- `pnpm install --frozen-lockfile` passed with the package-local lockfile.
- Three landing contract tests passed: semantic/static rendering, configured and fallback product CTA
  behavior, and the animation-free responsive baseline.
- TypeScript 7.0.2 and the Vite 8.1.5 production build passed. The generated entry HTML is 4.22 kB,
  CSS is 8.25 kB, and JavaScript is 0.90 kB before gzip; the static build requires no backend.
- Both supplied OFL notices ship beside the approved Instrument Sans and Commit Mono font files.
- Repository formatting, `git diff --check`, 99 executed Rust workspace tests, and the lifecycle/plugin
  doctor remained green. Five evaluator-worker tests remain delegated to the already-green pinned
  Product-worker harness.

## Follow-up boundary

Issue #53 expands the minimal product anchor into the operating-model, workspace, evidence, and
workflow proof chapters and adds optimized responsive product imagery. Issue #54 replaces the honest
access-unavailable copy with the configured request form. Issue #55 adds optional motion, responsive
visual regression, and asset-performance hardening without weakening this static baseline.
