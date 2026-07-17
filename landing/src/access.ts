export const ACCESS_SOURCE = "orchestra-landing";

export type AccessRequestResult =
  | { status: "success" }
  | { status: "server_error"; responseStatus: number }
  | { status: "network_error" };

type Fetcher = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

export async function postAccessRequest(
  endpoint: string,
  email: string,
  fetcher: Fetcher,
): Promise<AccessRequestResult> {
  try {
    const response = await fetcher(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ email, source: ACCESS_SOURCE }),
    });
    return response.ok
      ? { status: "success" }
      : { status: "server_error", responseStatus: response.status };
  } catch {
    return { status: "network_error" };
  }
}

export type AccessFormSetup = {
  available: boolean;
};

export function setupAccessForm(
  root: ParentNode,
  configuredEndpoint: string | undefined,
  fetcher: Fetcher,
): AccessFormSetup {
  const form = root.querySelector<HTMLFormElement>("#access-form");
  const email = root.querySelector<HTMLInputElement>("#access-email");
  const submit = form?.querySelector<HTMLButtonElement>('button[type="submit"]');
  const status = root.querySelector<HTMLElement>("#access-status");
  if (!form || !email || !submit || !status) return { available: false };

  const endpoint = configuredEndpoint?.trim();
  if (!endpoint) {
    email.disabled = true;
    submit.disabled = true;
    status.dataset.state = "unavailable";
    status.textContent = "Access requests are unavailable until a submission endpoint is configured.";
    return { available: false };
  }

  email.disabled = false;
  submit.disabled = false;
  status.dataset.state = "idle";
  status.textContent = "";

  let pending = false;
  let complete = false;

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (pending || complete) return;

    email.setAttribute("aria-invalid", "false");
    status.dataset.state = "idle";
    if (!email.validity.valid) {
      email.setAttribute("aria-invalid", "true");
      status.dataset.state = "validation_error";
      status.textContent = "Enter a valid work email to continue.";
      email.focus();
      return;
    }

    pending = true;
    form.setAttribute("aria-busy", "true");
    submit.disabled = true;
    submit.setAttribute("aria-busy", "true");
    submit.textContent = "Sending…";
    status.dataset.state = "pending";
    status.textContent = "Sending your access request…";

    const submittedEmail = email.value.trim();
    const result = await postAccessRequest(endpoint, submittedEmail, fetcher);
    pending = false;
    form.removeAttribute("aria-busy");
    submit.removeAttribute("aria-busy");

    if (result.status === "success") {
      complete = true;
      form.reset();
      email.disabled = true;
      submit.disabled = true;
      submit.textContent = "Request received";
      status.dataset.state = "success";
      status.textContent = "Your access request was received.";
      status.focus();
      return;
    }

    submit.disabled = false;
    submit.textContent = "Try again";
    status.dataset.state = result.status;
    status.textContent =
      result.status === "server_error"
        ? "The request endpoint could not accept your request. Try again."
        : "The request could not reach the endpoint. Check your connection and try again.";
  });

  return { available: true };
}
