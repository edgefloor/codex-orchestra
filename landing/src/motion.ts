const MOTION_STORAGE_KEY = "orchestra-motion";

export function resolveInitialMotion(stored: string | null, prefersReduced: boolean): boolean {
  if (stored === "on") return true;
  if (stored === "off") return false;
  return !prefersReduced;
}

type WebGLField = {
  start(): void;
  stop(): void;
};

function shader(
  gl: WebGLRenderingContext,
  type: number,
  source: string,
): WebGLShader | null {
  const compiled = gl.createShader(type);
  if (!compiled) return null;
  gl.shaderSource(compiled, source);
  gl.compileShader(compiled);
  if (!gl.getShaderParameter(compiled, gl.COMPILE_STATUS)) {
    gl.deleteShader(compiled);
    return null;
  }
  return compiled;
}

export function createWebGLField(canvas: HTMLCanvasElement, view: Window): WebGLField | null {
  const gl = canvas.getContext("webgl", {
    alpha: true,
    antialias: false,
    depth: false,
    powerPreference: "low-power",
  });
  if (!gl) return null;

  const vertex = shader(
    gl,
    gl.VERTEX_SHADER,
    "attribute vec2 p; void main(){ gl_Position = vec4(p, 0.0, 1.0); }",
  );
  const fragment = shader(
    gl,
    gl.FRAGMENT_SHADER,
    `precision mediump float;
     uniform vec2 resolution;
     uniform float time;
     void main() {
       vec2 uv = (gl_FragCoord.xy * 2.0 - resolution.xy) / min(resolution.x, resolution.y);
       float drift = sin(uv.x * 2.4 + time * 0.18) * cos(uv.y * 2.0 - time * 0.13);
       float glow = smoothstep(1.25, 0.05, length(uv - vec2(0.48, -0.12)) + drift * 0.11);
       vec3 iris = vec3(0.35, 0.28, 0.82);
       gl_FragColor = vec4(iris, glow * 0.16);
     }`,
  );
  if (!vertex || !fragment) return null;

  const program = gl.createProgram();
  if (!program) return null;
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null;

  const buffer = gl.createBuffer();
  const position = gl.getAttribLocation(program, "p");
  const resolution = gl.getUniformLocation(program, "resolution");
  const time = gl.getUniformLocation(program, "time");
  if (!buffer || position < 0 || !resolution || !time) return null;

  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 3, -1, -1, 3]),
    gl.STATIC_DRAW,
  );
  gl.useProgram(program);
  gl.enableVertexAttribArray(position);
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

  let frame = 0;
  let running = false;
  const draw = (timestamp: number) => {
    if (!running) return;
    const ratio = Math.min(view.devicePixelRatio || 1, 1.5);
    const width = Math.max(1, Math.round(view.innerWidth * ratio));
    const height = Math.max(1, Math.round(view.innerHeight * ratio));
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      gl.viewport(0, 0, width, height);
    }
    gl.uniform2f(resolution, width, height);
    gl.uniform1f(time, timestamp / 1000);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    frame = view.requestAnimationFrame(draw);
  };

  return {
    start() {
      if (running) return;
      running = true;
      frame = view.requestAnimationFrame(draw);
    },
    stop() {
      running = false;
      if (frame) view.cancelAnimationFrame(frame);
      frame = 0;
    },
  };
}

function revealSections(root: ParentNode, view: Window, enabled: boolean): void {
  const sections = [...root.querySelectorAll<HTMLElement>(".section")];
  const Observer = Reflect.get(view, "IntersectionObserver") as
    | typeof IntersectionObserver
    | undefined;
  if (!enabled || !Observer) {
    for (const section of sections) section.dataset.reveal = "visible";
    return;
  }

  const observer = new Observer(
    (entries: IntersectionObserverEntry[]) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        (entry.target as HTMLElement).dataset.reveal = "visible";
        observer.unobserve(entry.target);
      }
    },
    { threshold: 0.12 },
  );
  for (const section of sections) {
    section.dataset.reveal = "pending";
    observer.observe(section);
  }
}

export function setupMotion(root: Document, view: Window): void {
  const canvas = root.querySelector<HTMLCanvasElement>("#motion-field");
  const toggle = root.querySelector<HTMLButtonElement>("#motion-toggle");
  const intro = root.querySelector<HTMLElement>("#intro-overlay");
  if (!canvas || !toggle || !intro) return;

  const media = view.matchMedia("(prefers-reduced-motion: reduce)");
  let stored: string | null = null;
  try {
    stored = view.localStorage.getItem(MOTION_STORAGE_KEY);
  } catch {}
  let enabled = resolveInitialMotion(stored, media.matches);
  let field: WebGLField | null | undefined;
  let introShown = false;

  const apply = () => {
    root.documentElement.dataset.motion = enabled ? "on" : "off";
    toggle.setAttribute("aria-pressed", String(!enabled));
    toggle.setAttribute("aria-label", enabled ? "Pause motion" : "Resume motion");
    toggle.textContent = enabled ? "Pause motion" : "Resume motion";
    revealSections(root, view, enabled);

    if (!enabled) {
      field?.stop();
      intro.hidden = true;
      return;
    }

    if (field === undefined) {
      field = createWebGLField(canvas, view);
      root.documentElement.dataset.webgl = field ? "ready" : "unavailable";
    }
    field?.start();

    if (!introShown) {
      introShown = true;
      intro.hidden = false;
      intro.dataset.state = "visible";
      view.setTimeout(() => {
        intro.dataset.state = "leaving";
      }, 420);
      view.setTimeout(() => {
        intro.hidden = true;
      }, 980);
    }
  };

  toggle.addEventListener("click", () => {
    enabled = !enabled;
    try {
      view.localStorage.setItem(MOTION_STORAGE_KEY, enabled ? "on" : "off");
    } catch {}
    apply();
  });
  media.addEventListener("change", (event) => {
    let explicit: string | null = null;
    try {
      explicit = view.localStorage.getItem(MOTION_STORAGE_KEY);
    } catch {}
    if (explicit) return;
    enabled = !event.matches;
    apply();
  });

  apply();
}
