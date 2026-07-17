# Issue #54 — Landing request access

## Result

The static landing now exposes a real optional access-request contract. When
`VITE_ACCESS_REQUEST_URL` is configured, the form enables and sends one JSON POST shaped as
`{ "email": "<address>", "source": "orchestra-landing" }`. Every 2xx response succeeds. Without an
endpoint, the HTML starts disabled and clearly announces that requests are unavailable; it cannot
produce a fake success before JavaScript or configuration loads.

The form validates before sending, announces pending state, disables duplicate submission, and
distinguishes validation, server, and network failures. Failures leave an explicit enabled “Try again”
action. Success clears and disables the email field, labels the terminal button “Request received,”
announces the result, and moves focus to the status. No address is written to storage, logs, URL state,
or any Orchestra product/runtime surface.

## Automated evidence

- Nine Vitest/DOM tests passed. They cover static unavailability, malformed email focus, the exact POST
  body and source, arbitrary 2xx success, pending duplicate suppression, address clearing, success
  focus, server-error classification, network-error classification, product CTA behavior, responsive
  media, and the static no-motion baseline.
- Six Chromium tests passed. They cover keyboard submission, unavailable state, malformed input,
  configured POST capture, one request during duplicate Enter presses, 202 success, address clearing,
  success focus, 503 retry, aborted-network retry, configured/fallback product links, responsive AVIF
  selection, and all nine viewport widths without overflow.
- TypeScript and the Vite production build passed. Static output remains backend-independent; endpoint
  presence changes only the built client configuration and does not add a server to this repository.
- `git diff --check` passed. Root formatting, 99 executed Rust tests, and lifecycle/plugin doctor remain
  green and are unaffected by this isolated landing implementation.

Issue #55 owns optional motion, reduced-motion controls, visual regression, and final performance
thresholds. It must retain the disabled no-endpoint state and all focus/status behavior above.
