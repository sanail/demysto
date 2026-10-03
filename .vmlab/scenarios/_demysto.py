"""What every Demysto Scenario needs: a mock Provider, the Settings File
pointed at it, and the names the app goes by on each OS."""
import base64
import json
import pathlib
import uuid

HERE = pathlib.Path(__file__).parent
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


def configure(g, actions=None):
    """Writes a Settings File past the first run, pointed at the mock, and any
    Action files ({"explain": 'hotkey = "..."'}). The app must not be running."""
    g.put(config_file(g, "settings.toml"), f'''version = 1

default_model = "mock/mock-small"
welcomed = true

[[providers]]
name = "mock"
base_url = "{base_url(g)}"
api_key = "not-a-key"
models = [{{ id = "mock-small" }}]
''')
    for name, text in (actions or {}).items():
        g.put(config_file(g, "actions", name + ".toml"), text)


def configured_launch(g, actions=None):
    """For Scenarios with LAUNCH = False: the app, started past its first run."""
    configure(g, actions)
    g.launch()
