"""What every Demysto Scenario needs: a mock Provider, the Settings File
pointed at it, and the names the app goes by on each OS."""
import base64
import collections
import json
import pathlib
import re
import struct
import time
import uuid
import zlib

HERE = pathlib.Path(__file__).parent
# Found by walking up, so that Scenarios outside .vmlab/scenarios (ad-hoc ones) read them too.
CATALOGUES = next(p / "i18n" for p in [HERE.resolve(), *HERE.resolve().parents] if (p / "i18n").is_dir())
# Every interface language Demysto has a catalogue for.
CATALOGUE_LANGUAGES = sorted(p.stem for p in CATALOGUES.glob("*.ftl"))

# The name each interface language goes by in the welcome flow's language list.
LANGUAGE_NAMES = {"en": "English", "ru": "Русский"}


def language(g):
    """The interface language Demysto takes from the Lab language: ru-RU reads ru."""
    return g.language.split("-")[0]


def say(g, key, lang=None, **args):
    """A message from Demysto's own catalogue in the Lab language (or in lang,
    for an app started with DEMYSTO_LANGUAGE), so that a Scenario matches what
    the app draws however its translations change. Only single-line messages
    are read; a { $name } is filled from args."""
    lang = lang or language(g)
    for line in (CATALOGUES / f"{lang}.ftl").read_text(encoding="utf-8").splitlines():
        if line.startswith(key + " = "):
            text = line[len(key) + 3:]
            for name, value in args.items():
                text = text.replace("{ $" + name + " }", str(value))
            return text
    raise KeyError(f"{key} is not in i18n/{lang}.ftl")
# The Palette's width in logical pixels, as the app configures it.
PALETTE_WIDTH = next(window["width"] for window in
                     json.loads((CATALOGUES.parent / "src-tauri" / "tauri.conf.json").read_text())["app"]["windows"]
                     if window["label"] == "palette")
PORT = 18080
APP = "demysto"  # the process, which every OS's ui commands accept as the app


def nonce():
    return uuid.uuid4().hex[:8]


def watch_the_palette(g):
    """Starts recording each place and size Demysto's windows take on screen,
    far faster than the UI tree can be read."""
    if g.session == "wayland":
        script = (HERE / "watch" / "palette-wayland.py").read_text()
        watching = g.spawn(["python3", "-u", g.put("/tmp/demysto-watch.py", script)])
    elif g.os == "macos":
        script = (HERE / "watch" / "palette.js").read_text()
        watching = g.spawn(["osascript", "-l", "JavaScript", g.put("/tmp/demysto-watch.js", script)])
    elif g.os == "windows":
        script = (HERE / "watch" / "palette.ps1").read_text()
        path = g.put(r"%TEMP%\demysto-watch.ps1", script)
        watching = g.spawn(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", path])
    else:
        script = (HERE / "watch" / "palette.sh").read_text()
        watching = g.spawn(["sh", g.put("/tmp/demysto-watch.sh", script)])
    # The watcher's first line: "none", or bounds when the Palette is already open.
    up = g.wait_for(log=watching.log, pattern=r"(?m)^(none|-?\d)", timeout=30)
    g.check("the window watcher is running", up["met"], detail=[up, watching.output()])
    return watching


def check_no_resize(g, watching, what, settle=1.5):
    """Checks that the Palette, once on screen, keeps one shape for `settle`
    seconds. A shape is a height-to-width ratio, give or take rounding: macOS
    grows a panel into view at a constant one, and no resize keeps it."""
    name = f"{what} the Palette does not resize after it appears"
    # Slept rather than waited for: what is waited for is something not happening.
    time.sleep(settle)
    watching.stop()
    # Only lines of bounds: osascript logs lines of its own.
    lines = [line.strip() for line in watching.output().splitlines()
             if line.strip() == "none" or re.match(r"-?[\d.]+ -?[\d.]+ [\d.]+ [\d.]+( \||$)", line.strip())]
    # From its last appearance: the area overlay may have been on screen before it.
    since = lines[len(lines) - lines[::-1].index("none"):] if "none" in lines else lines
    shapes = []
    for line in since:
        _, _, w, h = (float(n) for n in line.split(" | ")[0].split())
        if not shapes or abs(h / w - shapes[-1][1]) > 0.01:
            shapes.append((line, h / w))
    g.check(name, len(shapes) == 1, detail=since)


# The work area in physical pixels, so the script declares itself DPI-aware
# or Windows scales its answer.
WINDOWS_WORK_AREA = """
Add-Type -Name Dpi -Namespace Vmlab -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();'
[void][Vmlab.Dpi]::SetProcessDPIAware()
Add-Type -AssemblyName System.Windows.Forms
$a = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
"$($a.X) $($a.Y) $($a.Width) $($a.Height)"
"""

# NSScreen counts in points from the bottom; the tree counts in pixels from the top.
MAC_WORK_AREA = ('ObjC.import("AppKit"); var s = $.NSScreen.mainScreen, f = s.frame, v = s.visibleFrame;'
                 '[v.origin.x, f.size.height - v.origin.y - v.size.height, v.size.width, v.size.height].join(" ")')


def work_area(g, scale):
    """The primary screen less its Dock, menu bar, taskbar or panels, in the
    pixels the tree gives bounds in: x, y, width, height."""
    if g.os == "windows":
        script = g.put(r"%TEMP%\demysto-work-area.ps1", WINDOWS_WORK_AREA)
        out = g.exec(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script]).stdout
    elif g.os == "macos":
        out = g.exec(["osascript", "-l", "JavaScript", "-e", MAC_WORK_AREA]).stdout
        return [float(n) * scale for n in out.split()[:4]]
    else:
        # _NET_WORKAREA: x, y, width, height once per desktop.
        out = g.exec(["xprop", "-root", "_NET_WORKAREA"]).stdout.split("=")[1].replace(",", " ")
    return [float(n) for n in out.split()[:4]]


# Eleven Actions of the user's own over the built-ins, each taking text and
# pictures: fifteen offered for text and fourteen for a picture, past the ten
# the Palette lists before its list scrolls.
MANY_ACTIONS = {f"extra-{n:02}": f'name = "Extra {n:02}"\naccepts = ["text", "image"]\ntemplate = "{{{{selection}}}}"\n'
                for n in range(1, 12)}


def action_names(g):
    """The name of every Action MANY_ACTIONS leaves offered, in the Lab language."""
    built_in = ["action-explain-name", "action-translate-name", "action-summarize-name",
                "action-describe-image-name", "action-custom-name"]
    return [say(g, key) for key in built_in] + [f"Extra {n:02}" for n in range(1, 12)]


def palette_layout(g, footer):
    """The bounds of the Palette's window and of its footer, which says
    `footer`, and of each of its Actions, by name: in the tree's pixels."""
    bottom = g.wait_for(text=footer, app=APP, timeout=10)
    footer_bounds = bottom["matches"][0]["bounds"] if bottom["met"] else None
    windows = [m["bounds"] for m in g.find(role="window", app=APP)["matches"] if m.get("bounds")]
    window = next((w for w in windows if footer_bounds and w["y"] <= footer_bounds["y"] <= w["y"] + w["h"]),
                  windows[0] if windows else None)
    names = set(action_names(g))
    actions = {m["name"]: m["bounds"] for m in g.find(role="button", app=APP)["matches"]
               if m["name"] in names and m.get("bounds")}
    return window, footer_bounds, actions


def listed(actions, footer):
    """The Actions whose whole button shows above the footer, which is where
    the list ends."""
    return sorted(name for name, b in actions.items() if b["y"] + b["h"] <= footer["y"])


def nothing_selected(g):
    """An editor in front with nothing selected, so that the Palette's copy
    leaves the clipboard as it is; None on Wayland, where no copy is sent."""
    if g.session == "wayland":
        return None
    staged = g.stage_text("nothing selected " + nonce())
    g.press("right")
    return staged


def picture_name(g):
    """What the Palette calls a picture, short of its size."""
    return say(g, "palette-picture", dimensions="").strip(" ,")


def check_scrolls_to_last(g, actions, footer):
    """Presses Up from the first Action, which goes round to the last, and
    checks the list scrolls to bring it into view."""
    last = max(actions, key=lambda name: actions[name]["y"]) if actions else None
    g.press("up")
    _, below, actions = palette_layout(g, footer)
    g.check("the list scrolls to the last Action", last in listed(actions, below),
            detail={"last": last, "actions": actions})


def check_fitted(g, window, footer, what):
    """Checks the Palette ends where its content does: under the footer is the
    frame's padding and border, and no empty space."""
    scale = g.displays()["displays"][0]["scale"]
    drawn = drawn_bounds(g, window)
    under = drawn["y"] + drawn["h"] - (footer["y"] + footer["h"]) if window and footer else None
    # 16 of padding, 1 of border, and rounding.
    g.check(f"{what}, the Palette ends at its footer", under is not None and 0 <= under <= 19 * scale,
            detail={"window": window, "drawn": drawn, "footer": footer, "under the footer": under})


def drawn_bounds(g, window):
    """The part of the Palette's window bounds it draws in. Windows counts a
    frame that draws nothing at the sides and bottom; the Palette's width
    tells how wide it is."""
    if not window:
        return None
    scale = g.displays()["displays"][0]["scale"]
    frame = max(0, (window["w"] - PALETTE_WIDTH * scale) / 2)
    return {"x": window["x"] + frame, "y": window["y"], "w": window["w"] - 2 * frame, "h": window["h"] - frame}


def screen_pixels(g, name):
    """A screenshot, kept as evidence, decoded: (width, height, rows)."""
    shot = g.screenshot(name)
    # The Run folder is .vmlab/runs/<timestamp>-<lab>, the latest of them this Run's.
    path = next(run / shot for run in sorted((HERE.parent / "runs").glob(f"*-{g.lab}"), reverse=True)
                if (run / shot).is_file())
    return decode_png(path.read_bytes())


def poll(read, done, timeout):
    """read() until done(what it read) or timeout seconds pass; answers the
    last read. For what wait_for cannot wait on: the focus, pixels."""
    deadline = time.monotonic() + timeout
    while True:
        seen = read()
        if done(seen) or time.monotonic() > deadline:
            return seen
        time.sleep(0.3)


def ink(rows, bounds):
    """The share of the pixels inside bounds that stand out from their
    commonest colour: none at all is a window that was not drawn."""
    inset = 4
    region = [pixel for row in rows[int(bounds["y"]) + inset:int(bounds["y"] + bounds["h"]) - inset]
              for pixel in row[int(bounds["x"]) + inset:int(bounds["x"] + bounds["w"]) - inset]]
    if not region:
        return 0
    background = collections.Counter(region).most_common(1)[0][0]
    return sum(1 for pixel in region if sum(abs(a - b) for a, b in zip(pixel, background)) > 96) / len(region)


def tray(g):
    # g.tray takes the process name on Windows, the app's own name elsewhere.
    return "demysto" if g.os == "windows" else "Demysto"


def palette_hotkey(g):
    return "cmd+shift+space" if g.os == "macos" else "ctrl+shift+space"


def area_hotkey(g):
    return "ctrl+shift+space" if g.os == "macos" else "ctrl+alt+space"


def explain_hotkey(g):
    """The Hotkey given to Explain, as the Action file states it and as a chord."""
    stated = "Cmd+Shift+E" if g.os == "macos" else "Ctrl+Alt+Shift+E"
    return stated, stated.lower()


def config_dir(g):
    return {
        "macos": "~/Library/Application Support/demysto",
        "windows": r"%APPDATA%\demysto",
        "linux": "~/.config/demysto",
    }[g.os]


def config_file(g, *parts):
    sep = "\\" if g.os == "windows" else "/"
    return sep.join([config_dir(g), *parts])


def base_url(g):
    # The Windows mock listens on localhost: see mock/provider.ps1.
    host = "localhost" if g.os == "windows" else "127.0.0.1"
    return f"http://{host}:{PORT}/v1"


def mock(g, answer, status=200, delay=0):
    """Starts the mock Provider and waits until it answers; returns its handle."""
    args = [str(PORT), str(status), answer, str(delay)]
    if g.os == "windows":
        script = g.put(r"%TEMP%\demysto-provider.ps1", (HERE / "mock" / "provider.ps1").read_text())
        argv = ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, *args]
        probe = ["curl.exe", "-fsS", base_url(g) + "/models"]
    else:
        script = g.put("/tmp/demysto-provider.py", (HERE / "mock" / "provider.py").read_text())
        argv = ["python3", "-u", script, *args]
        # Ubuntu's desktop install has no curl.
        probe = ["python3", "-c", "import sys, urllib.request; urllib.request.urlopen(sys.argv[1])", base_url(g) + "/models"]
    handle = g.spawn(argv)
    up = g.wait_for(exec=probe, timeout=30)
    if not up["met"]:
        raise RuntimeError(f"the mock Provider never answered: {up} {handle.output()}")
    return handle


def shows_answer(g, answer, timeout=30):
    """Waits for the Model's answer; (met, what was seen). Not met while Demysto
    says it could not read the answer: that message quotes the body whole."""
    shown = g.wait_for(text=answer, app=APP, timeout=timeout)
    misread = g.find(text=say(g, "provider-malformed").split("{ $")[0].rstrip(" ("), app=APP)["matches"]
    return shown["met"] and not misread, {"shown": shown, "misread": misread}


def prompts(handle):
    """The user prompt of every chat request the mock received, oldest first."""
    found = []
    for line in handle.output().splitlines():
        method, path, _, body = (line.strip().split(" ") + ["", "", "", ""])[:4]
        if method == "POST" and body:
            content = json.loads(base64.b64decode(body))["messages"][-1]["content"]
            if isinstance(content, list):
                content = "".join(part.get("text", "") for part in content)
            found.append(content)
    return found


def received(handle):
    """Every PNG the mock was sent, as bytes, oldest first."""
    found = []
    for line in handle.output().splitlines():
        method, path, _, body = (line.strip().split(" ") + ["", "", "", ""])[:4]
        if method != "POST" or not body:
            continue
        content = json.loads(base64.b64decode(body))["messages"][-1]["content"]
        for part in content if isinstance(content, list) else []:
            if part.get("type") == "image_url":
                found.append(base64.b64decode(part["image_url"]["url"].split(",", 1)[1]))
    return found


def pictures(handle):
    """The (width, height) of every picture the mock was sent, oldest first,
    read from the PNG itself rather than from the size of the request."""
    return [struct.unpack(">II", png[16:24]) for png in received(handle)]


def encode_png(width, height, pixel):
    """An 8-bit RGB PNG whose pixel at (x, y) is pixel(x, y)."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            rows += bytes(pixel(x, y))

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(bytes(rows), 6)) + chunk(b"IEND", b""))


def png(width, height):
    """A striped picture: a flat one would compress to nothing, and its fitted
    and original encodings would weigh the same."""
    return encode_png(width, height, lambda x, y: ((x * 7 + y * 3) % 256, (x * 3) % 256, (y * 5) % 256))


def decode_png(data):
    """(width, height, rows of (r, g, b)) of an 8-bit, non-interlaced RGB or
    RGBA PNG: what Demysto sends. Anything else raises."""
    width, height, depth, colour_type, _, _, interlace = struct.unpack(">IIBBBBB", data[16:29])
    if depth != 8 or colour_type not in (2, 6) or interlace:
        raise ValueError(f"an unexpected PNG: depth {depth}, colour type {colour_type}, interlace {interlace}")
    channels = 3 if colour_type == 2 else 4
    idat, at = b"", 8
    while at < len(data):
        length, name = struct.unpack(">I4s", data[at:at + 8])
        if name == b"IDAT":
            idat += data[at + 8:at + 8 + length]
        at += 12 + length
    raw, stride = zlib.decompress(idat), width * channels
    rows, previous = [], bytearray(stride)
    for y in range(height):
        filter_type, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            left = line[i - channels] if i >= channels else 0
            up = previous[i]
            corner = previous[i - channels] if i >= channels else 0
            if filter_type == 1:
                line[i] = (line[i] + left) & 255
            elif filter_type == 2:
                line[i] = (line[i] + up) & 255
            elif filter_type == 3:
                line[i] = (line[i] + (left + up) // 2) & 255
            elif filter_type == 4:
                p = left + up - corner
                pa, pb, pc = abs(p - left), abs(p - up), abs(p - corner)
                line[i] = (line[i] + (left if pa <= pb and pa <= pc else up if pb <= pc else corner)) & 255
        rows.append([tuple(line[x * channels:x * channels + 3]) for x in range(width)])
        previous = line
    return width, height, rows


# Flat blocks, three across and two down, in colours far apart in every channel,
# so that a colour profile shifting them still leaves each nearest its own.
BLOCKS = [(220, 40, 40), (40, 180, 60), (40, 70, 220), (240, 200, 40), (200, 60, 200), (40, 200, 210)]


def blocks_png(width, height):
    """BLOCKS laid out over a width × height picture, three across and two down."""
    return encode_png(width, height, lambda x, y: BLOCKS[(y * 2 // height) * 3 + x * 3 // width])


def nearest_block(colour):
    """The index of the block whose colour is nearest colour."""
    return min(range(len(BLOCKS)), key=lambda i: sum((a - b) ** 2 for a, b in zip(BLOCKS[i], colour)))


def copy_picture(g, width, height):
    """Puts a width × height picture on the Guest's clipboard, as an image."""
    if g.os == "windows":
        path = g.put(r"%TEMP%\demysto-picture.png", png(width, height))
        script = g.put(r"%TEMP%\demysto-copy-picture.ps1", (
            "param([string]$Path)\n"
            "Add-Type -AssemblyName System.Windows.Forms, System.Drawing\n"
            "[Windows.Forms.Clipboard]::SetImage([Drawing.Image]::FromFile($Path))\n"))
        g.exec(["powershell", "-STA", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, path])
    elif g.os == "macos":
        path = g.put("/tmp/demysto-picture.png", png(width, height))
        g.exec(["osascript", "-e", f'set the clipboard to (read (POSIX file "{path}") as «class PNGf»)'])
    else:
        # X11 keeps no clipboard content of its own: xclip stays to serve it,
        # in the foreground (-quiet), so that the Run can stop it; forked into
        # the background it outlives the Scenario and holds the clipboard.
        path = g.put("/tmp/demysto-picture.png", png(width, height))
        g.spawn(["xclip", "-quiet", "-selection", "clipboard", "-t", "image/png", "-i", path])
        g.wait_for(exec=["sh", "-c", "xclip -selection clipboard -t TARGETS -o | grep -q image/png"], timeout=10)


def copy_a_broken_picture(g):
    """Puts bytes that are no picture on the Guest's clipboard, under the type
    each OS keeps pictures in."""
    junk = bytes(range(1, 65))
    if g.os == "windows":
        # The format browsers copy pictures in.
        path = g.put(r"%TEMP%\demysto-bad-picture.png", junk)
        script = g.put(r"%TEMP%\demysto-bad-picture.ps1", (
            "param([string]$Path)\n"
            "Add-Type -AssemblyName System.Windows.Forms\n"
            "$data = New-Object System.Windows.Forms.DataObject\n"
            "$data.SetData('PNG', (New-Object System.IO.MemoryStream(,[IO.File]::ReadAllBytes($Path))))\n"
            "[System.Windows.Forms.Clipboard]::SetDataObject($data, $true)\n"))
        g.exec(["powershell", "-STA", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, path])
    elif g.os == "macos":
        g.exec(["osascript", "-e", f'set the clipboard to «data PNGf{junk.hex().upper()}»'])
    else:
        path = g.put("/tmp/demysto-bad-picture.png", junk)
        g.spawn(["xclip", "-quiet", "-selection", "clipboard", "-t", "image/png", "-i", path])
        g.wait_for(exec=["sh", "-c", "xclip -selection clipboard -t TARGETS -o | grep -q image/png"], timeout=10)


GNOME_SHORTCUTS = "gnome-control-center-global-shortcuts-provider"


def approve_shortcuts(g):
    """On Wayland GNOME asks whether Demysto may have its Hotkeys at every
    launch until it is allowed, and again when what they are called changes;
    this allows them. Elsewhere it does nothing."""
    if g.session != "wayland":
        return

    def shown():
        # A dialog already answered stays in the tree, with no bounds.
        return [m for m in g.find(text="Add", role="button", app=GNOME_SHORTCUTS)["matches"] if m["bounds"]]

    deadline = time.monotonic() + 10
    while not shown():
        if time.monotonic() > deadline:
            return
        time.sleep(0.5)
    g.click(text="Add", role="button", app=GNOME_SHORTCUTS)
    deadline = time.monotonic() + 10
    while shown() and time.monotonic() < deadline:
        time.sleep(0.5)


def foreground(g):
    """in_front(process, timeout): a wait until that process's window is in
    the foreground. Windows and X11 only."""
    if g.os == "windows":
        script = g.put(r"%TEMP%\demysto-front.ps1", (HERE / "mock" / "front.ps1").read_text())
        argv = ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script]
    else:
        argv = ["sh", g.put("/tmp/demysto-front.sh", (HERE / "mock" / "front.sh").read_text())]

    def in_front(process, timeout):
        return g.wait_for(exec=argv, pattern=rf"(?im)^{re.escape(process)}$", timeout=timeout)

    return in_front


BLOCKS_TITLE = "demysto-blocks"


def show_blocks(g):
    """Puts blocks_png over the whole primary display, in a window of another
    application, and answers that display's bounds."""
    picture = blocks_png(960, 540)
    if g.os == "windows":
        path = g.put(r"%TEMP%\demysto-blocks.png", picture)
        script = g.put(r"%TEMP%\demysto-blocks.ps1", (HERE / "mock" / "blocks.ps1").read_text())
        g.spawn(["powershell", "-STA", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, path, BLOCKS_TITLE])
    elif g.os == "macos":
        path = g.put("/tmp/demysto-blocks.png", picture)
        script = g.put("/tmp/demysto-blocks.js", (HERE / "mock" / "blocks.js").read_text())
        g.spawn(["osascript", "-l", "JavaScript", script, path, BLOCKS_TITLE])
    else:
        path = g.put("/tmp/demysto-blocks.png", picture)
        g.spawn(["python3", g.put("/tmp/demysto-blocks.py", (HERE / "mock" / "blocks.py").read_text()), path, BLOCKS_TITLE])
    shown = g.wait_for(text=BLOCKS_TITLE, role="window", timeout=30)
    if not shown["met"]:
        raise RuntimeError(f"the picture never came up: {shown}")
    return next(d for d in g.displays()["displays"] if d["primary"])["bounds"]


def configure(g, actions=None, settings=""):
    """Writes a Settings File past the first run, pointed at the mock, with any
    extra top-level settings lines, and any Action files
    ({"explain": 'hotkey = "..."'}). The app must not be running."""
    g.put(config_file(g, "settings.toml"), f'''version = 1

default_model = "mock/mock-small"
default_vision_model = "mock/mock-small"
welcomed = true
{settings}
[[providers]]
name = "mock"
base_url = "{base_url(g)}"
api_key = "not-a-key"
models = [{{ id = "mock-small", vision = true }}]
''')
    for name, text in (actions or {}).items():
        g.put(config_file(g, "actions", name + ".toml"), text)


def configured_launch(g, actions=None, settings=""):
    """For Scenarios with LAUNCH = False: the app, started past its first run."""
    configure(g, actions, settings)
    g.launch()
    approve_shortcuts(g)


def nodes(tree):
    stack = [tree]
    while stack:
        n = stack.pop()
        yield n
        stack.extend(n.get("children") or [])


def largest_window(tree):
    """The bounds of the app's largest window in a UI tree."""
    windows = [n for n in nodes(tree) if n["role"] == "window" and n.get("bounds")]
    return max(windows, key=lambda n: n["bounds"]["w"] * n["bounds"]["h"])["bounds"]
