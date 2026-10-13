import { getCurrentWindow } from "@tauri-apps/api/window";
import { overlayChosen, overlayFrame, type Area } from "../lib/ipc";

const frame = document.getElementById("frame") as HTMLImageElement;
const dim = document.getElementById("dim")!;
const area = document.getElementById("area")!;
const size = document.getElementById("size")!;

// The window covers the monitor exactly, so CSS pixels times this are the
// frame's.
const scale = devicePixelRatio;

let from: { x: number; y: number } | null = null;
let chosen = false;

function choose(picked: Area | null) {
  if (chosen) return;
  chosen = true;
  void overlayChosen(picked);
}

/** The drag in the frame's own pixels, which are the monitor's physical ones. */
function dragged(to: PointerEvent): Area {
  const left = Math.min(from!.x, to.clientX);
  const top = Math.min(from!.y, to.clientY);
  const right = Math.max(from!.x, to.clientX);
  const bottom = Math.max(from!.y, to.clientY);
  const x = Math.round(left * scale);
  const y = Math.round(top * scale);
  return {
    x,
    y,
    width: Math.round(right * scale) - x,
    height: Math.round(bottom * scale) - y,
  };
}

document.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  from = { x: event.clientX, y: event.clientY };
  document.body.setPointerCapture(event.pointerId);
});

document.addEventListener("pointermove", (event) => {
  if (!from) return;
  const picked = dragged(event);
  Object.assign(area.style, {
    left: `${picked.x / scale}px`,
    top: `${picked.y / scale}px`,
    width: `${picked.width / scale}px`,
    height: `${picked.height / scale}px`,
  });
  size.textContent = `${picked.width} × ${picked.height}`;
  // Below the area, or above it where the screen ends.
  const below = (picked.y + picked.height) / scale + 6;
  size.style.left = `${picked.x / scale}px`;
  size.style.top =
    below + 24 < innerHeight ? `${below}px` : `${Math.max(0, picked.y / scale - 26)}px`;
  dim.hidden = true;
  area.hidden = size.hidden = false;
});

// A click without a drag is an empty area, which Demysto takes as a cancel.
document.addEventListener("pointerup", (event) => {
  if (from) choose(dragged(event));
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") choose(null);
});

addEventListener("blur", () => choose(null));

async function freeze() {
  const bmp = await overlayFrame();
  frame.src = URL.createObjectURL(new Blob([bmp], { type: "image/bmp" }));
  await frame.decode();
  const window = getCurrentWindow();
  await window.show();
  await window.setFocus();
}

freeze().catch(() => choose(null));
