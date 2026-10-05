"""What every Demysto Scenario needs: a mock Provider, the Settings File
pointed at it, and the names the app goes by on each OS."""
import base64
import json
import pathlib
import struct
import uuid
import zlib

HERE = pathlib.Path(__file__).parent
# Found by walking up, so that Scenarios outside .vmlab/scenarios (ad-hoc ones) read them too.
CATALOGUES = next(p / "i18n" for p in [HERE.resolve(), *HERE.resolve().parents] if (p / "i18n").is_dir())

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
PORT = 18080
APP = "demysto"  # the process, which every OS's ui commands accept as the app


def nonce():
    return uuid.uuid4().hex[:8]


def tray(g):
    # g.tray takes the process name on Windows, the app's own name elsewhere.
    return "demysto" if g.os == "windows" else "Demysto"


def palette_hotkey(g):
    return "cmd+shift+space" if g.os == "macos" else "ctrl+shift+space"


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


def pictures(handle):
    """The (width, height) of every picture the mock was sent, oldest first,
    read from the PNG itself rather than from the size of the request."""
    found = []
    for line in handle.output().splitlines():
        method, path, _, body = (line.strip().split(" ") + ["", "", "", ""])[:4]
        if method != "POST" or not body:
            continue
        content = json.loads(base64.b64decode(body))["messages"][-1]["content"]
        for part in content if isinstance(content, list) else []:
            if part.get("type") == "image_url":
                png = base64.b64decode(part["image_url"]["url"].split(",", 1)[1])
                found.append(struct.unpack(">II", png[16:24]))
    return found


def png(width, height):
    """A striped picture: a flat one would compress to nothing, and its fitted
    and original encodings would weigh the same."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            rows += bytes(((x * 7 + y * 3) % 256, (x * 3) % 256, (y * 5) % 256))

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(bytes(rows), 6)) + chunk(b"IEND", b""))


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


def configure(g, actions=None):
    """Writes a Settings File past the first run, pointed at the mock, and any
    Action files ({"explain": 'hotkey = "..."'}). The app must not be running."""
    g.put(config_file(g, "settings.toml"), f'''version = 1

default_model = "mock/mock-small"
default_vision_model = "mock/mock-small"
welcomed = true

[[providers]]
name = "mock"
base_url = "{base_url(g)}"
api_key = "not-a-key"
models = [{{ id = "mock-small", vision = true }}]
''')
    for name, text in (actions or {}).items():
        g.put(config_file(g, "actions", name + ".toml"), text)


def configured_launch(g, actions=None):
    """For Scenarios with LAUNCH = False: the app, started past its first run."""
    configure(g, actions)
    g.launch()
