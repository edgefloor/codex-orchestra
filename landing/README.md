# Orchestra landing

The landing site is an independent static Vite package. It explains and links to the Orchestra
product; it does not host the Codex application or require a product backend.

```sh
pnpm install --frozen-lockfile
pnpm test
pnpm build
pnpm dev
```

Set `VITE_PRODUCT_URL` at build time to send product links to a download or desktop destination. When
it is absent, those links scroll to the in-page product preview. The access form and its endpoint are
configured at build time with `VITE_ACCESS_REQUEST_URL`. The endpoint must allow the deployed landing
origin and accept a JSON POST with `email` and source `orchestra-landing`; any 2xx response succeeds.
When it is absent, the form stays disabled and reports that access requests are unavailable.

The contents of `dist/` can be hosted by any static file service. Hosting, TLS, CORS, the access
backend, and the product/download destination remain deployment concerns; none run inside or couple to
the Codex desktop. Optional WebGL, intro, and reveal motion progressively enhance the semantic page.
The motion control persists only its own `on`/`off` preference, and first render honors the operating
system reduced-motion preference when no explicit choice exists.

Font binaries and their OFL notices are retained under `public/assets/fonts/` from the approved design
handoff.
