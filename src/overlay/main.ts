import { getCurrentWindow } from "@tauri-apps/api/window";
import { overlayChosen, overlayFrame, type Area } from "../lib/ipc";

const frame = document.getElementById("frame") as HTMLImageElement;
const dim = document.getElementById("dim")!;
const area = document.getElementById("area")!;
const size = document.getElementById("size")!;

// The frame is stretched over the window, so CSS pixels times this are the
// frame's whatever scaling the system applies.
const scale = () => frame.naturalWidth / innerWidth;

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
  const ratio = scale();
  const x = Math.round(left * ratio);
  const y = Math.round(top * ratio);
  return {
    x,
    y,
    width: Math.round(right * ratio) - x,
    height: Math.round(bottom * ratio) - y,
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
  const ratio = scale();
  Object.assign(area.style, {
    left: `${picked.x / ratio}px`,
    top: `${picked.y / ratio}px`,
    width: `${picked.width / ratio}px`,
    height: `${picked.height / ratio}px`,
  });
  size.textContent = `${picked.width} × ${picked.height}`;
  // Below the area, or above it where the screen ends.
  const below = (picked.y + picked.height) / ratio + 6;
  size.style.left = `${picked.x / ratio}px`;
  size.style.top =
    below + 24 < innerHeight ? `${below}px` : `${Math.max(0, picked.y / ratio - 26)}px`;
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

async function freeze() {
  const bmp = await overlayFrame();
  frame.src = URL.createObjectURL(new Blob([bmp], { type: "image/bmp" }));
  await frame.decode();
  const window = getCurrentWindow();
  await window.show();
  await window.setFocus();
}

freeze().catch(() => choose(null));
