# Issue #53 — Landing product, evidence, and workflow proof

## Result

The landing narrative now continues from the hero through the approved operating model, native product
workspace, evidence trail, explicit Workflow gates, access CTA, and footer. The copy describes Orchestra
as an extension of native Codex tasks and subagents: work stays in normal tasks and worktrees, raw child
history stays native, and bounded evidence expands only when a reviewer needs it. No detached runner or
alternate product surface is implied.

The product destination remains a build-time `VITE_PRODUCT_URL`. When absent, both product CTAs point
to `#workspace`; activating either one also moves keyboard focus to the workspace proof before the
browser completes the in-page scroll.

## Responsive media

- The hero monolith, product workspace, evidence archive, and Workflow panorama each ship two intrinsic
  widths in AVIF, WebP, and PNG, for 24 responsive files total.
- Every `picture` lists AVIF first, WebP second, and a responsive PNG `img` fallback with explicit
  dimensions and `sizes`. Chromium selected the 960-wide hero and 720-wide workspace AVIFs at 390 px,
  and the 1820-wide hero AVIF at 1440 px.
- Product and architecture proof images have meaningful alternatives or captions. The hero architecture
  remains decorative because the adjacent heading and supporting copy carry its complete meaning.

## Automated evidence

- Five DOM contract tests passed for the static semantic hero, configured/fallback destination,
  workspace focus, responsive source contracts, and animation-free baseline.
- Three Chromium tests passed: fallback focus/scroll, configured destination propagation to every CTA,
  and responsive image/containment behavior.
- Browser layout passed at 360×800, 390×844, 430×932, 600×960, 820×1180, 1024×768, 1366×768,
  1440×900, and 1920×1080 without horizontal overflow.
- TypeScript and the Vite production build passed. The built HTML is 12.48 kB, CSS is 12.27 kB, and
  JavaScript is 1.21 kB before gzip; the page still requires no backend.
- `git diff --check` passed. The root formatting, 99 executed Rust tests, and lifecycle/plugin doctor
  remained green after issue #52 and are unaffected by this static-only slice.

Issue #54 now owns the real access submission contract and states. Issue #55 owns optional intro/reveal
motion, reduced-motion controls, broader visual regression, and final asset-performance thresholds.
