// @vitest-environment happy-dom

import { describe, expect, it, vi } from "vitest";

import page from "../index.html?raw";
import { ACCESS_SOURCE, postAccessRequest, setupAccessForm } from "./access";

function pageDocument(): Document {
  return new DOMParser().parseFromString(page, "text/html");
}

function submit(form: HTMLFormElement): void {
  form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
}

describe("request access", () => {
  it("keeps the static form truthfully unavailable without a configured endpoint", () => {
    const doc = pageDocument();
    const fetcher = vi.fn();

    expect(setupAccessForm(doc, undefined, fetcher).available).toBe(false);
    expect(doc.querySelector<HTMLInputElement>("#access-email")?.disabled).toBe(true);
    expect(doc.querySelector<HTMLButtonElement>('#access-form button[type="submit"]')?.disabled).toBe(
      true,
    );
    expect(doc.querySelector("#access-status")?.textContent).toContain("unavailable");
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("validates before sending and focuses malformed email input", async () => {
    const doc = pageDocument();
    const fetcher = vi.fn();
    setupAccessForm(doc, "https://example.com/access", fetcher);

    const form = doc.querySelector<HTMLFormElement>("#access-form")!;
    const email = doc.querySelector<HTMLInputElement>("#access-email")!;
    email.value = "not-an-email";
    submit(form);

    await vi.waitFor(() => expect(email.getAttribute("aria-invalid")).toBe("true"));
    expect(doc.activeElement).toBe(email);
    expect(doc.querySelector("#access-status")?.textContent).toContain("valid work email");
    expect(fetcher).not.toHaveBeenCalled();
  });

  it("posts the exact contract, accepts every 2xx, clears the email, and suppresses duplicates", async () => {
    const doc = pageDocument();
    let finishRequest: ((response: Response) => void) | undefined;
    const fetcher = vi.fn(
      () => new Promise<Response>((resolve) => {
        finishRequest = resolve;
      }),
    );
    setupAccessForm(doc, "https://example.com/access", fetcher);

    const form = doc.querySelector<HTMLFormElement>("#access-form")!;
    const email = doc.querySelector<HTMLInputElement>("#access-email")!;
    email.value = "person@example.com";
    submit(form);
    submit(form);

    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(fetcher).toHaveBeenCalledWith("https://example.com/access", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ email: "person@example.com", source: ACCESS_SOURCE }),
    });

    finishRequest?.({ ok: true, status: 202 } as Response);
    await vi.waitFor(() =>
      expect(doc.querySelector("#access-status")?.getAttribute("data-state")).toBe("success"),
    );
    expect(email.value).toBe("");
    expect(email.disabled).toBe(true);
    expect(doc.activeElement).toBe(doc.querySelector("#access-status"));
  });

  it("distinguishes server and network failures and leaves retry enabled", async () => {
    const server = await postAccessRequest(
      "https://example.com/access",
      "person@example.com",
      vi.fn(async () => ({ ok: false, status: 503 }) as Response),
    );
    const network = await postAccessRequest(
      "https://example.com/access",
      "person@example.com",
      vi.fn(async () => {
        throw new Error("offline");
      }),
    );

    expect(server).toEqual({ status: "server_error", responseStatus: 503 });
    expect(network).toEqual({ status: "network_error" });
  });
});
