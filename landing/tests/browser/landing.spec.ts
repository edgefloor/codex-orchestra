import { expect, test, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const viewports = [
  { width: 360, height: 800 },
  { width: 390, height: 844 },
  { width: 430, height: 932 },
  { width: 600, height: 960 },
  { width: 820, height: 1180 },
  { width: 1024, height: 768 },
  { width: 1366, height: 768 },
  { width: 1440, height: 900 },
  { width: 1920, height: 1080 },
] as const;

async function enableAccess(page: Page, endpoint: string): Promise<void> {
  await page.evaluate(async (url) => {
    const { setupAccessForm } = await import("/src/access.ts");
    setupAccessForm(document, url, window.fetch.bind(window));
  }, endpoint);
}

test("fallback product CTA scrolls to and focuses the workspace proof", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "View the product workspace" }).click();

  await expect(page).toHaveURL(/#workspace$/);
  await expect(page.locator("#workspace")).toBeFocused();
});

test("configured product destination updates every product CTA", async ({ page }) => {
  await page.goto("/");
  const destination = "https://example.com/orchestra";

  await page.evaluate(async (url) => {
    const { configureProductLinks } = await import("/src/landing.ts");
    configureProductLinks(document, url);
  }, destination);

  const links = page.locator("[data-product-link]");
  await expect(links).toHaveCount(2);
  expect(await links.evaluateAll((items) => items.map((item) => item.getAttribute("href")))).toEqual([
    destination,
    destination,
  ]);
});

test("unconfigured access remains disabled and validation prevents requests", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#access-email")).toBeDisabled();
  await expect(page.locator("#access-status")).toContainText("unavailable");

  await enableAccess(page, "https://api.example.test/access");
  await page.locator("#access-email").fill("not-an-email");
  await page.locator("#access-email").press("Enter");
  await expect(page.locator("#access-email")).toBeFocused();
  await expect(page.locator("#access-email")).toHaveAttribute("aria-invalid", "true");
  await expect(page.locator("#access-status")).toContainText("valid work email");
});

test("configured access posts once, accepts 2xx, clears the address, and announces success", async ({
  page,
}) => {
  const endpoint = "https://api.example.test/access";
  let requests = 0;
  let requestBody: unknown;
  let requestMethod = "";
  await page.route(endpoint, async (route) => {
    requests += 1;
    requestBody = route.request().postDataJSON();
    requestMethod = route.request().method();
    await new Promise((resolve) => setTimeout(resolve, 100));
    await route.fulfill({ status: 202 });
  });

  await page.goto("/");
  await enableAccess(page, endpoint);
  const input = page.locator("#access-email");
  await input.fill("person@example.com");
  await input.press("Enter");
  await input.press("Enter");

  await expect(page.locator("#access-status")).toContainText("Sending");
  await expect(page.locator("#access-status")).toContainText("received");
  expect(requests).toBe(1);
  expect(requestMethod).toBe("POST");
  expect(requestBody).toEqual({ email: "person@example.com", source: "orchestra-landing" });
  await expect(input).toHaveValue("");
  await expect(input).toBeDisabled();
  await expect(page.locator("#access-status")).toBeFocused();
});

test("server and network failures expose retry states", async ({ page }) => {
  const endpoint = "https://api.example.test/access";
  await page.route(endpoint, (route) => route.fulfill({ status: 503 }));
  await page.goto("/");
  await enableAccess(page, endpoint);
  await page.locator("#access-email").fill("person@example.com");
  await page.locator("#access-email").press("Enter");
  await expect(page.locator("#access-status")).toHaveAttribute("data-state", "server_error");
  await expect(page.getByRole("button", { name: "Try again" })).toBeEnabled();

  await page.unroute(endpoint);
  await page.route(endpoint, (route) => route.abort("failed"));
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.locator("#access-status")).toHaveAttribute("data-state", "network_error");
  await expect(page.getByRole("button", { name: "Try again" })).toBeEnabled();
});

test("motion control toggles progressive enhancement without hiding content", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("html")).toHaveAttribute("data-motion", "on");
  await expect(page.getByRole("heading", { name: "Control the work, not the agents." })).toBeVisible();

  const toggle = page.getByRole("button", { name: "Pause motion" });
  await toggle.click();
  await expect(page.locator("html")).toHaveAttribute("data-motion", "off");
  await expect(page.getByRole("button", { name: "Resume motion" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  expect(await page.evaluate(() => localStorage.getItem("orchestra-motion"))).toBe("off");
});

test("reduced motion is honored from first render", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");

  await expect(page.locator("html")).toHaveAttribute("data-motion", "off");
  await expect(page.locator("#intro-overlay")).toBeHidden();
  await expect(page.getByRole("button", { name: "Resume motion" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Control the work, not the agents." })).toBeVisible();
});

test("WebGL failure leaves the complete static page available", async ({ page }) => {
  await page.addInitScript(() => {
    HTMLCanvasElement.prototype.getContext = () => null;
  });
  await page.goto("/");

  await expect(page.locator("html")).toHaveAttribute("data-webgl", "unavailable");
  await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Control the work, not the agents." })).toBeVisible();
  await expect(page.locator("#access-form")).toBeVisible();
});

test("finished semantic page has no automated accessibility violations", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("orchestra-motion", "off"));
  await page.goto("/");
  const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
  expect(results.violations).toEqual([]);
});

test("responsive sources select modern assets and every approved viewport is contained", async ({
  page,
}) => {
  const mobileImageRequests: string[] = [];
  page.on("request", (request) => {
    if (request.resourceType() === "image") mobileImageRequests.push(request.url());
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  const smallHeroSource = await page.locator(".hero-monolith img").evaluate((image) =>
    new URL((image as HTMLImageElement).currentSrc).pathname,
  );
  expect(smallHeroSource).toContain("operations-monolith-960.avif");

  const workspaceImage = page.locator(".workspace-shot img");
  await workspaceImage.scrollIntoViewIfNeeded();
  await expect(workspaceImage).toBeVisible();
  const smallWorkspaceSource = await workspaceImage.evaluate((image) =>
    new URL((image as HTMLImageElement).currentSrc).pathname,
  );
  expect(smallWorkspaceSource).toContain("product-workspace-720.avif");
  expect(mobileImageRequests.some((url) => url.includes("operations-monolith-1820"))).toBe(false);
  expect(mobileImageRequests.some((url) => url.includes("product-workspace-1440"))).toBe(false);

  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    await page.goto("/");

    const dimensions = await page.evaluate(() => ({
      clientWidth: document.documentElement.clientWidth,
      scrollWidth: document.documentElement.scrollWidth,
    }));
    expect(dimensions.scrollWidth).toBe(dimensions.clientWidth);
  }

  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  const largeHeroSource = await page.locator(".hero-monolith img").evaluate((image) =>
    new URL((image as HTMLImageElement).currentSrc).pathname,
  );
  expect(largeHeroSource).toContain("operations-monolith-1820.avif");
});

test("motion-off visual baseline matches the approved viewport matrix", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("orchestra-motion", "off"));
  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    await page.goto("/");
    await expect(page).toHaveScreenshot(`landing-${viewport.width}x${viewport.height}.png`, {
      animations: "disabled",
      fullPage: false,
    });
  }
});
