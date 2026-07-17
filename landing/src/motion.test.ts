// @vitest-environment happy-dom

import { describe, expect, it } from "vitest";

import { createWebGLField, resolveInitialMotion } from "./motion";

describe("landing motion", () => {
  it("honors reduced motion unless the visitor has made an explicit choice", () => {
    expect(resolveInitialMotion(null, false)).toBe(true);
    expect(resolveInitialMotion(null, true)).toBe(false);
    expect(resolveInitialMotion("off", false)).toBe(false);
    expect(resolveInitialMotion("on", true)).toBe(true);
  });

  it("treats missing WebGL as a normal static fallback", () => {
    const canvas = { getContext: () => null } as unknown as HTMLCanvasElement;
    expect(createWebGLField(canvas, window)).toBeNull();
  });
});
