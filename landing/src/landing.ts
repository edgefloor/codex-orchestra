export const PRODUCT_FALLBACK = "#workspace";

export function resolveProductHref(configuredUrl: string | undefined): string {
  const candidate = configuredUrl?.trim();
  return candidate ? candidate : PRODUCT_FALLBACK;
}

export function configureProductLinks(
  root: ParentNode,
  configuredUrl: string | undefined,
): string {
  const href = resolveProductHref(configuredUrl);
  for (const link of root.querySelectorAll<HTMLAnchorElement>("[data-product-link]")) {
    link.href = href;
    if (href !== PRODUCT_FALLBACK) {
      link.rel = "noreferrer";
    }
  }
  return href;
}

export function enableProductFallbackFocus(root: ParentNode): void {
  const target = root.querySelector<HTMLElement>(PRODUCT_FALLBACK);
  if (!target) return;

  for (const link of root.querySelectorAll<HTMLAnchorElement>("[data-product-link]")) {
    if (link.dataset.fallbackFocusBound === "true") continue;
    link.dataset.fallbackFocusBound = "true";
    link.addEventListener("click", () => {
      if (link.getAttribute("href") === PRODUCT_FALLBACK) {
        target.focus({ preventScroll: true });
      }
    });
  }
}
