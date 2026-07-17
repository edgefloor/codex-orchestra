// @vitest-environment happy-dom

import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import page from "../index.html?raw";
import {
  configureProductLinks,
  enableProductFallbackFocus,
  PRODUCT_FALLBACK,
  resolveProductHref,
} from "./landing";

const styles = readFileSync("src/styles.css", "utf8");

function pageDocument(): Document {
  return new DOMParser().parseFromString(page, "text/html");
}

describe("landing foundation", () => {
  it("ships semantic navigation and a useful static hero before JavaScript runs", () => {
    const doc = pageDocument();

    expect(doc.querySelector('nav[aria-label="Primary navigation"]')).not.toBeNull();
    expect(doc.querySelector<HTMLAnchorElement>(".skip-link")?.getAttribute("href")).toBe(
      "#content",
    );
    expect(doc.querySelectorAll("h1")).toHaveLength(1);
    expect(doc.querySelector("h1")?.textContent).toContain(
      "Control the work, not the agents.",
    );
    expect(doc.querySelector<HTMLAnchorElement>('.hero a[href="#access"]')).not.toBeNull();
    expect(doc.querySelector<HTMLAnchorElement>("[data-product-link]")?.getAttribute("href")).toBe(
      PRODUCT_FALLBACK,
    );
    expect(doc.querySelector("#workspace h2")).not.toBeNull();
    expect(doc.querySelector("#access h2")).not.toBeNull();
    expect([...doc.querySelectorAll("h1, h2")].map((heading) => heading.tagName)).toEqual([
      "H1",
      "H2",
      "H2",
      "H2",
      "H2",
      "H2",
    ]);
  });

  it("uses the configured product destination and otherwise retains the in-page fallback", () => {
    const doc = pageDocument();

    expect(resolveProductHref(undefined)).toBe(PRODUCT_FALLBACK);
    expect(resolveProductHref("   ")).toBe(PRODUCT_FALLBACK);
    expect(configureProductLinks(doc, undefined)).toBe(PRODUCT_FALLBACK);
    expect(
      [...doc.querySelectorAll<HTMLAnchorElement>("[data-product-link]")].map((link) =>
        link.getAttribute("href"),
      ),
    ).toEqual([PRODUCT_FALLBACK, PRODUCT_FALLBACK]);

    const destination = "https://example.com/orchestra";
    expect(configureProductLinks(doc, destination)).toBe(destination);
    for (const link of doc.querySelectorAll<HTMLAnchorElement>("[data-product-link]")) {
      expect(link.href).toBe(destination);
      expect(link.rel).toBe("noreferrer");
    }
  });

  it("focuses the workspace proof when the product URL falls back in-page", () => {
    const doc = pageDocument();
    configureProductLinks(doc, undefined);
    enableProductFallbackFocus(doc);

    doc.querySelector<HTMLAnchorElement>(".hero [data-product-link]")?.click();
    expect(doc.activeElement).toBe(doc.querySelector(PRODUCT_FALLBACK));
  });

  it("offers responsive AVIF, WebP, and PNG sources for every proof image", () => {
    const doc = pageDocument();
    const pictures = [...doc.querySelectorAll("picture")];

    expect(pictures).toHaveLength(4);
    for (const picture of pictures) {
      expect(picture.querySelector('source[type="image/avif"][srcset]')).not.toBeNull();
      expect(picture.querySelector('source[type="image/webp"][srcset]')).not.toBeNull();
      expect(picture.querySelector("img[srcset][sizes]")).not.toBeNull();
    }
    expect(doc.querySelector<HTMLImageElement>(".workspace-shot img")?.alt).toContain(
      "Orchestra workspace",
    );
  });

  it("keeps the complete page outside optional motion layers", () => {
    const doc = pageDocument();

    expect(doc.querySelector('canvas[aria-hidden="true"]')).not.toBeNull();
    expect(doc.querySelector('.intro-overlay[aria-hidden="true"][hidden]')).not.toBeNull();
    expect(doc.querySelector(".intro-overlay main")).toBeNull();
    expect(styles).toContain('html[data-motion="off"]');
    expect(styles).toContain("@media (prefers-reduced-motion: reduce)");
    expect(styles).toContain("overflow-x: clip");
    expect(styles).toContain("@media (max-width: 720px)");
    expect(styles).toContain(":focus-visible");
  });
});
