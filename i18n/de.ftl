# Demysto's interface in German.
#
# Written against `en.ftl`, which is where a message is added first. Every
# identifier there exists here: the suite reads both files and fails the build
# over one this catalogue is missing, and over one it holds that English does
# not.
#
# The macOS panes are named as macOS itself names them in German —
# "Datenschutz & Sicherheit", "Bedienungshilfen" — because a sentence that sends
# somebody to a pane has to name the one they will actually see.

## The application itself

app-name = Demysto
tray-open = Demysto öffnen
tray-actions = Aktionen
tray-update = Auf { $version } aktualisieren…
tray-settings = Einstellungen…
tray-quit = Demysto beenden

# macOS only, and only for the key equivalents: `menu` says why the menu bar
# exists at all and why nothing else is on it.
menu-edit = Bearbeiten
menu-quit = Demysto beenden

# The one thing Demysto raises a notification for: a Run started from an
# Action's own Hotkey that failed with no window on screen to say so.
notification-stopped-part-way = Demysto ist mittendrin stehen geblieben
notification-could-not-answer = Demysto konnte nicht antworten

## The Actions Demysto comes with
#
# Their names and the Parameters they collect, which is what the Palette shows.
# Their prompt templates stay in `action`, in English, because they are
# addressed to a Model rather than to a person.

action-explain-name = Erklären
action-translate-name = Übersetzen
action-translate-target-label = In welche Sprache?
action-summarize-name = Zusammenfassen
action-describe-image-name = Bild beschreiben
action-custom-name = Eigene…
action-custom-prompt-label = Was soll damit geschehen?

## The Palette

palette-reading-selection = Lese die Auswahl…
palette-reading-clipboard = Lese die Zwischenablage…
palette-origin-selection = Auswahl
palette-origin-clipboard = Aus der Zwischenablage
palette-picture = Ein Bild, { $dimensions }
palette-nothing-captured = Es ist nichts ausgewählt, und die Zwischenablage ist leer. Wählen Sie Text aus und drücken Sie das Tastenkürzel noch einmal.
palette-filter = Aktionen filtern…
palette-no-action-matches = Nichts gefunden.
palette-back = Zurück
palette-next = Weiter
palette-run = Ausführen
palette-open-accessibility = Einstellungen für Bedienungshilfen öffnen
palette-keys-collecting = Enter zum Ausführen · Esc zurück
palette-keys-choosing = ↑↓ zum Auswählen · Enter zum Ausführen · Esc zum Schließen
palette-keys-closing = Esc zum Schließen

## The Conversation window

result-conversations = Chats
result-conversation-unnamed = Chat
result-nothing-asked-yet = Noch nichts gefragt.
result-quotation-label = Der Text dieses Chats
result-picture-label = Das Bild dieses Chats
result-show-more = Mehr anzeigen
result-show-less = Weniger anzeigen
result-ask-at-original = Erneut in voller Auflösung fragen — { $weight }
result-picture-let-go = Dieses Bild wird nicht mehr aufbewahrt.
result-sealed = Das Bild wird nicht mehr aufbewahrt, deshalb kann dieser Chat nicht weitergehen. Kopieren Sie es und drücken Sie das Tastenkürzel für einen neuen.
result-asking = Frage das Modell…
result-reasoning = Das Modell denkt nach…
result-copy-answer = Antwort kopieren
result-copied = Kopiert
result-stopped = Angehalten
result-continue = Fortsetzen
result-try-again = Wiederholen
result-ask-another-model = Ein anderes Modell fragen…
result-open-provider-settings = Einstellungen von { $provider } öffnen
result-open-accessibility = Einstellungen für Bedienungshilfen öffnen
result-follow-up = Nachfragen…
result-stop = Anhalten
result-ask = Fragen
result-keys = Enter zum Fragen, Umschalt+Enter für eine neue Zeile, Esc zum Schließen

## A rendered code block, whose copy button is markup rather than a component

code-copy = Kopieren
code-copied = Kopiert

## Settings

settings-window-title = Demysto-Einstellungen
settings-title = Einstellungen
settings-save = Speichern
settings-saving = Speichere…
settings-saved = Gespeichert.
settings-keys = Esc zum Schließen
settings-reading = Lese die Einstellungen…
settings-unreadable-file = Hier lässt sich nichts ändern, solange die Einstellungsdatei nicht repariert ist. Berichtigen Sie sie und öffnen Sie dieses Fenster erneut.

settings-folder = Einstellungsordner

### The tabs the window is divided into

settings-tab-models = Modelle
settings-tab-actions = Aktionen
settings-tab-general = Allgemein
settings-tab-about = Über
settings-tab-unsaved = ungespeicherte Änderungen
settings-tab-update = eine Aktualisierung steht bereit

### Providers

settings-providers = Anbieter
settings-add-provider = Einen Anbieter hinzufügen
settings-remove-provider = Diesen Anbieter entfernen
settings-provider-edit = Bearbeiten
settings-provider-unnamed = (ohne Namen)
settings-no-providers = Noch kein Anbieter. Fügen Sie einen hinzu, um zu beginnen.
settings-provider-name = Name
settings-provider-name-example = openai
settings-provider-service = Dienst
settings-provider-no-preset = Ohne Voreinstellung
settings-provider-preset-keyless = { $preset } (ohne Schlüssel)
settings-provider-base-url = Basis-URL
settings-provider-base-url-from-preset = Basis-URL — leer lassen, um die der Voreinstellung zu nehmen
settings-provider-base-url-example = https://api.example.com/v1
settings-provider-key = API-Schlüssel
settings-provider-key-variable = Oder die Umgebungsvariable, die ihn enthält
settings-provider-key-variable-example = MY_API_KEY
settings-key-in-file = Liegt in der Einstellungsdatei
settings-key-in-environment = Stammt aus { $variable }
settings-key-not-needed = Dieser Dienst hat keine Schlüssel
settings-key-missing = Noch kein Schlüssel
settings-key-going = Wird beim Speichern entfernt
settings-keep-key = Den Schlüssel in der Datei behalten
settings-remove-key = Den Schlüssel aus der Datei entfernen

### The Models one Provider offers

settings-models = Modelle
settings-fetch-models = Abrufen
settings-verify-key = Schlüssel prüfen
settings-verify-which-model = Mit welchem Modell?
settings-add-model = Ein Modell hinzufügen
settings-remove-model = Entfernen
settings-model-sees-images = Sieht Bilder
settings-no-models = Noch kein Modell. Rufen Sie die Liste ab, oder tragen Sie eines von Hand ein.
settings-asking-provider = Frage den Anbieter…
settings-provider-offers-nothing = Er bietet kein Modell an.
settings-provider-answered = { $model } hat geantwortet.

### Defaults

settings-defaults = Standardwerte
settings-default-model = Standardmodell — sofern eine Aktion kein eigenes nennt
settings-default-vision-model = Standardmodell für Bilder
settings-model-none = Keines
settings-model-does-not-see = { $model } (sieht keine Bilder)
settings-large-selection = Warnen ab so vielen Zeichen
settings-large-selection-default = { $characters } als Standard
settings-large-selection-detail = Es wird nie etwas abgeschnitten — die Warnung sorgt nur dafür, dass ein versehentliches „Alles markieren“ nicht unbemerkt bezahlt wird. Leer lassen für Demystos eigenen Wert, 0 für keine Warnung.

### Language

settings-language = Sprache
settings-language-field = Sprache der Oberfläche
settings-language-follows-system = Wie im System
settings-language-from-environment = { $variable } steht auf { $value }; das ist also die Sprache, die Demysto spricht, was auch immer hier gewählt wird.

### Hotkeys

settings-hotkeys = Tastenkürzel
settings-palette-hotkey = Öffnet die Liste der Aktionen
settings-hotkey-record = Aufnehmen
settings-hotkey-clear = Leeren
settings-hotkey-cancel = Abbrechen
settings-hotkey-recording = Tasten drücken… Esc zum Abbrechen
settings-hotkey-default = { $hotkey } — Standard
settings-hotkey-none = Keines — diese Aktion wird aus der Liste gestartet
settings-palette-hotkey-rule = Halten Sie Ctrl, Alt oder Shift und drücken Sie eine Taste. Eine Taste, die etwas schreibt, lässt sich nicht allein belegen: sie würde überall aufhören zu schreiben.
settings-action-hotkey-rule = Halten Sie Ctrl, Alt oder Shift und drücken Sie eine Taste. Eine Taste, die etwas schreibt, lässt sich nicht allein belegen: sie würde überall aufhören zu schreiben. Parameter werden dabei nicht abgefragt — jeder nimmt seinen Standardwert.
settings-wayland-hotkeys = Unter Wayland vergibt die Arbeitsumgebung die Tastenkürzel, nicht Demysto. Ändern Sie sie in deren eigenen Tastatureinstellungen, wo sie unter Demysto aufgeführt sind.

### Startup

settings-autostart = Start
settings-autostart-choice = Beim Anmelden starten
settings-autostart-changed = Erledigt
settings-autostart-detail = Die Anmeldeobjekte sind die Liste Ihres Systems, deshalb wirkt dieses Kästchen sofort und lässt sich auch dort ändern.

### Logs

settings-logs = Protokolle
settings-logs-detail = Das Protokoll hält fest, was Demysto getan hat — welche Aktion, welches Modell, was schiefging — nie, was Sie angesehen haben oder was ein Modell gesagt hat. Legen Sie es einem Fehlerbericht bei.
settings-open-logs = Den Protokollordner öffnen

### Updates

settings-updates = Aktualisierungen
settings-updates-detail = Jedes Update ist mit Demystos Schlüssel signiert und wird vor dem Einsetzen geprüft, und ohne Ihr Wort wird nichts installiert.
settings-version = Demysto { $version }
settings-check-for-update = Nach Updates suchen
settings-checking = Wird gesucht…
settings-up-to-date = Das ist die neueste Version.
settings-update-found = Demysto { $version } ist bereit zur Installation.
settings-install-update = Installieren und neu starten
settings-installing = Wird installiert…

### Actions

settings-actions = Aktionen
settings-write-action = Neue Aktion
settings-actions-detail = Jede Aktion ist eine eigene Datei in <code>actions</code> — sichern Sie eine, oder schicken Sie sie jemandem.
settings-action-changed = Geändert
settings-action-yours = Ihre
settings-action-unsaved = nicht gespeichert
settings-action-edit = Bearbeiten
settings-action-reset = Zurücksetzen
settings-action-delete = Löschen
settings-action-name = Name
settings-action-name-example = Schlicht umschreiben
settings-action-model = Modell
settings-action-model-default = Standard
settings-action-hotkey = Tastenkürzel
settings-action-accepts = Gilt für
settings-action-accepts-text = Text
settings-action-accepts-image = Bilder
settings-action-accepts-detail = Wählen Sie mindestens eines. Ein Bild reist neben dem Prompt und nicht darin, deshalb bleibt {"{{"}selection{"}}"} bei einem Bild leer.
settings-action-prompt = Prompt
settings-action-prompt-example =
    Erkläre den Text unten. Der Text ist auf {"{{"}selection_language{"}}"}; antworte auf {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> ist das, was Sie markiert haben. <code>{"{{"}ui_language{"}}"}</code> ist die Sprache, die Sie lesen, <code>{"{{"}selection_language{"}}"}</code> die Sprache des Textes selbst. Alles andere in doppelten geschweiften Klammern ist ein Parameter.
settings-parameters = Parameter
settings-declare-parameter = Parameter hinzufügen
settings-remove-parameter = Entfernen
settings-no-parameters = Keine — diese Aktion läuft sofort los.
settings-parameter-id = Name
settings-parameter-label = Frage
settings-parameter-default = Standard
settings-parameter-id-example = target
settings-parameter-label-example = In welche Sprache?
settings-save-action = Aktion speichern
settings-cancel = Abbrechen
settings-reset-by-saving = Speichern ohne Änderung stellt die eingebaute Aktion wieder her.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Willkommen bei Demysto
welcome-step = Schritt { $at } von { $total }
welcome-back = Zurück
welcome-continue = Weiter
welcome-finish = Demysto benutzen
welcome-language-title = Demysto hat Ihre Sprache gefunden
welcome-language-detail = Ihr System sagt, dass Sie diese lesen. Ändern Sie sie hier oder später in den Einstellungen.
welcome-provider-title = Woher die Antworten kommen
welcome-provider-detail = Demysto fragt ein Modell über Ihr eigenes Konto. Wählen Sie den Dienst, fügen Sie dessen Schlüssel ein und rufen Sie die Modelle ab.
welcome-provider-model = Standardmodell
welcome-provider-verify-first = Prüfen Sie den Schlüssel, um fortzufahren: besser jetzt einen falschen finden als bei der ersten Frage.
welcome-accessibility-title = Demysto lesen lassen, was Sie ausgewählt haben
welcome-accessibility-detail = macOS gibt das Lesen Ihrer Markierung nur mit der Bedienungshilfen-Berechtigung frei. Öffnen Sie Datenschutz & Sicherheit → Bedienungshilfen und schalten Sie Demysto ein.
welcome-open-accessibility = Einstellungen für Bedienungshilfen öffnen
welcome-accessibility-later = Sie können sie später erteilen — Demysto fragt bei jedem Lauf danach. Nach einem Update fragt macOS erneut: für sie ist das eine andere Anwendung.
welcome-autostart-title = Demysto beim Anmelden starten
welcome-autostart-detail = Demysto wartet im Infobereich und antwortet nur, solange es läuft.
welcome-autostart-choice = Beim Anmelden starten
welcome-done-title = Das war alles
welcome-done-detail = Markieren Sie irgendwo Text und drücken Sie { $hotkey }. Am Cursor öffnet sich eine Liste der Aktionen; Enter startet die hervorgehobene.
welcome-done-clipboard = Kopieren Sie Text mit Ctrl+C und drücken Sie { $hotkey }. Am Cursor öffnet sich eine Liste der Aktionen; Enter startet die hervorgehobene.
welcome-done-tray = Demysto wartet von nun an im Infobereich, und sein Menü erreicht alles, was das Tastenkürzel erreicht.

## What an update could not do

update-refused = Demysto konnte nicht nachfragen, ob es eine neue Version gibt: { $detail }
update-install-refused = Die Aktualisierung konnte nicht installiert werden: { $detail }
update-nothing-found = Nichts zu installieren: suchen Sie zuerst nach Updates.

## What the login items would not do

autostart-refused = Demysto konnte den Start beim Anmelden nicht ändern: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Wayland lässt eine Anwendung nicht in eine andere schreiben, deshalb kann Demysto Ihre Markierung nicht lesen. Kopieren Sie sie mit Ctrl+C und drücken Sie das Tastenkürzel.
capture-clipboard-unavailable = Die Zwischenablage ist nicht verfügbar: { $detail }
capture-keystroke-refused = Der Tastendruck zum Kopieren konnte nicht gesendet werden: { $detail }
capture-no-accessibility = macOS lässt Demysto Ihre Markierung ohne die Bedienungshilfen-Berechtigung nicht lesen. Öffnen Sie Datenschutz & Sicherheit → Bedienungshilfen und schalten Sie Demysto ein.
capture-picture-unreadable = Demysto konnte das Bild in der Zwischenablage nicht lesen. Kopieren Sie es erneut oder ein anderes.
accessibility-pane-unreachable = Demysto konnte die Systemeinstellungen nicht öffnen: { $detail }. Die Berechtigung liegt unter „Datenschutz & Sicherheit“ → „Bedienungshilfen“.
accessibility-only-macos = Nur macOS verlangt eine Berechtigung, bevor Demysto Ihre Markierung lesen kann.

## What stopped a Run

run-nothing-to-run = Es gibt nichts, worauf eine Aktion laufen könnte. Markieren oder kopieren Sie Text und drücken Sie das Tastenkürzel erneut.
run-no-conversation = Es gibt keinen Chat, in dem das gefragt werden könnte. Drücken Sie das Tastenkürzel, um einen zu beginnen.
run-conversation-sealed = Das Bild dieses Chats wird nicht mehr aufbewahrt. Kopieren Sie es und drücken Sie das Tastenkürzel für einen neuen Chat.
run-no-such-action = Es gibt keine Aktion namens „{ $action }“ — sie wurde vielleicht entfernt. Drücken Sie das Tastenkürzel erneut.
run-nothing-to-retry = Es gibt nichts zu wiederholen. Stellen Sie die Frage erneut.

# The one warning a Conversation carries, said before the Model is asked so that
# it is on screen while the answer is still being paid for.
#
# No plural selector, unlike English and Russian: "Zeichen" is the same word for
# one and for many, and a selector whose branches read alike is a selector that
# will one day be edited on one side only.
run-large-selection = Diese Auswahl ist { $shown } Zeichen lang, und damit über den { $limit }, auf die { $setting } in { $path } gesetzt ist. Sie wurde ganz gesendet — nichts wurde abgeschnitten — und kostet also, was das kostet.

## What the settings file could not be made into

config-unreadable = { $path } konnte nicht gelesen werden: { $detail }
config-unwritable = { $path } konnte nicht geschrieben werden: { $detail }
config-not-toml-at-line = { $path } ist kein gültiges TOML in Zeile { $line }: { $detail }
config-not-toml = { $path } ist kein gültiges TOML: { $detail }
config-newer-version = { $path } gibt sich als Version { $stated } aus, und dieses Demysto versteht Version { $understood }; aktualisieren Sie Demysto, oder richten Sie { $variable } auf ein anderes Verzeichnis
config-uneditable = { $path } konnte nicht bearbeitet werden, ohne zu verlieren, was darin steht, deshalb wurde nichts gespeichert.
config-no-provider = es ist kein Anbieter eingerichtet; öffnen Sie { $path } und füllen Sie das Beispiel darin aus
config-in-file = { $reason } in { $path }
config-provider-no-name = ein Anbieter ist ohne Namen eingerichtet
config-provider-name-has-separator = der Anbieter „{ $provider }“ hat ein „{ $separator }“ im Namen, und genau das trennt einen Anbieter von einem Modell
config-two-providers-named = zwei Anbieter heißen „{ $provider }“, deshalb lässt sich ein Modell von keinem der beiden benennen
config-provider-model-no-name = der Anbieter „{ $provider }“ führt ein Modell ohne Namen auf
config-provider-model-twice = der Anbieter „{ $provider }“ führt das Modell „{ $model }“ zweimal auf
config-provider-no-base-url = der Anbieter „{ $provider }“ in { $path } nennt weder base_url noch eine Voreinstellung, aus der sie zu nehmen wäre
config-no-key-anywhere = Der Anbieter „{ $provider }“ hat keinen API-Schlüssel: setzen Sie api_key für ihn in { $path }, oder nennen Sie eine Umgebungsvariable in api_key_env.
config-no-key-export = Der Anbieter „{ $provider }“ hat keinen API-Schlüssel: exportieren Sie { $variables }, oder setzen Sie api_key für ihn in { $path }.
config-no-such-preset = Es gibt keine Voreinstellung namens „{ $preset }“.

## Which Model a Run resolves to, when it resolves to none

model-none-configured = Es ist überhaupt kein Modell eingerichtet; fügen Sie dort einem Anbieter eines hinzu.
model-configured-are = Die dort eingerichteten Modelle sind: { $models }.
model-action-binds-nothing = Diese Aktion ist an das Modell „{ $model }“ gebunden, und kein Anbieter in { $path } bietet eines dieses Namens an. { $offered }
model-setting-names-nothing = { $setting } in { $path } nennt das Modell „{ $model }“, und kein Anbieter dort bietet eines dieses Namens an. { $offered }
model-nothing-nominated = In { $path } ist kein { $setting } benannt. { $offered }
model-no-vision-model = In { $path } ist kein { $setting } benannt, also gibt es kein Modell, dem sich ein Bild zeigen ließe. { $offered }
model-nomination-none-configured = { $setting } nennt das Modell „{ $model }“, und es ist überhaupt kein Modell eingerichtet.
model-nomination-unknown = { $setting } nennt das Modell „{ $model }“, und kein Anbieter bietet eines dieses Namens an. Eingerichtet sind die Modelle: { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto konnte keine Verbindung öffnen: { $detail }
provider-timed-out =
    { $provider } hat nicht innerhalb von { $seconds ->
        [one] einer Sekunde
       *[other] { $seconds } Sekunden
    } geantwortet, deshalb hat Demysto aufgehört zu warten.
provider-unreachable = { $provider } war nicht erreichbar: { $detail }
provider-went-quiet = { $provider } verstummte mitten in der Antwort.
provider-stopped-answering = { $provider } hat mitten in der Antwort aufgehört zu antworten: { $detail }
provider-closed-early = { $provider } hat die Verbindung geschlossen, bevor die Antwort fertig war.
provider-refused = Der Anbieter hat die Anfrage abgelehnt (HTTP { $status }).
provider-refused-saying = Der Anbieter hat die Anfrage abgelehnt (HTTP { $status }): { $detail }
provider-malformed = Demysto konnte die Antwort des Anbieters nicht lesen ({ $reason }): { $body }
provider-no-answer-in-it = sie enthält keine Antwort

## What an Action could not be made into

action-file-preamble = # Eine Aktion, die Demysto ausführt. Bearbeiten Sie sie hier oder in Demystos Einstellungen.
action-needs-name = Eine Aktion braucht einen Namen, unter dem sie aufgeführt wird.
action-needs-prompt = Eine Aktion braucht einen Prompt: das, was sie dem Modell sagt, mit {"{{"}selection{"}}"} dort, wo die Auswahl hingehört.
action-accepts-nothing = Eine Aktion, die nichts annimmt, könnte nie angeboten werden. Wählen Sie mindestens eine Art von Auswahl.
action-parameter-needs-name = Ein Parameter braucht einen Namen, um im Prompt als {"{{"}like_this{"}}"} geschrieben zu werden.
action-parameter-reserved = Ein Parameter kann nicht „{ $parameter }“ heißen: den füllt Demysto selbst aus.
action-parameter-needs-label = Der Parameter „{ $parameter }“ braucht eine Frage, mit der danach gefragt wird.
action-parameter-twice = Zwei Parameter heißen „{ $parameter }“, deshalb könnte {"{{"}{ $parameter }{"}}"} im Prompt jeden von beiden meinen.
action-binds-nothing-configured = Diese Aktion bindet das Modell „{ $model }“, und es ist überhaupt kein Modell eingerichtet.
action-binds-unknown-model = Diese Aktion bindet das Modell „{ $model }“, und kein Anbieter bietet eines dieses Namens an. Eingerichtet sind die Modelle: { $models }.
action-id-not-a-file-name = „{ $action }“ kann kein Dateiname sein, deshalb lässt sich darunter keine Aktion aufbewahren.
action-none-to-remove = Es gibt keine Aktion namens „{ $action }“ zum Entfernen — sie ist vielleicht schon fort. Öffnen Sie dieses Fenster erneut.
action-file-newer-version = { $path } gibt sich als Version { $stated } aus, und dieses Demysto versteht Version { $understood }. Aktualisieren Sie Demysto, oder nehmen Sie die Datei aus diesem Verzeichnis.
action-file-states-no-field = { $path } nennt kein { $field }. Eine Aktion, die Demysto noch nicht hat, muss ihren Namen und ihre Vorlage nennen.
action-file-unreadable = { $path } konnte nicht gelesen werden: { $detail }
action-dir-unreadable = { $path } konnte nicht gelesen werden, deshalb sind die Aktionen darin nicht aufgeführt: { $detail }
action-file-unwritable = { $path } konnte nicht geschrieben werden: { $detail }
action-file-unwritable-shape = { $path } konnte nicht als TOML geschrieben werden: { $detail }
action-file-invalid-at-line = { $path } ist keine gültige Aktion in Zeile { $line }: { $detail }
action-file-invalid = { $path } ist keine gültige Aktion: { $detail }

## Hotkeys the desktop would not give up

hotkey-palette-fell-back = { $why } Demysto verwendet stattdessen { $hotkey }.
hotkey-palette-unclaimable = Demysto konnte { $hotkey } nicht beanspruchen, das die Liste der Aktionen öffnet: { $detail }. Vielleicht hat eine andere Anwendung es bereits. Das Menü im Infobereich erreicht alles, was das Tastenkürzel erreicht.
hotkey-palette-not-a-combination = Die Einstellungen nennen das Tastenkürzel „{ $hotkey }“, das Demysto nicht versteht. Ein Tastenkürzel sind seine Modifikatoren und dann eine Taste, geschrieben wie „Ctrl+Shift+E“.
hotkey-palette-types-something = Die Einstellungen nennen das Tastenkürzel „{ $hotkey }“ — eine einzelne Taste, die schreibt. Fügen Sie Ctrl, Alt oder Shift hinzu: allein würde sie überall aufhören zu schreiben.
hotkey-palette-refused = Demysto konnte das in den Einstellungen genannte Tastenkürzel „{ $hotkey }“ nicht beanspruchen: { $detail }. Vielleicht hat eine andere Anwendung es bereits.
hotkey-action-not-a-combination = { $action } nennt das Tastenkürzel „{ $hotkey }“, das Demysto nicht versteht. Ein Tastenkürzel sind seine Modifikatoren und dann eine Taste, geschrieben wie „Ctrl+Shift+E“.
hotkey-action-types-something = { $action } nennt das Tastenkürzel „{ $hotkey }“ — eine einzelne Taste, die schreibt. Fügen Sie Ctrl, Alt oder Shift hinzu: allein würde sie überall aufhören zu schreiben.
hotkey-action-already-held = { $action } nennt das Tastenkürzel „{ $hotkey }“, und { $holder } hat es bereits. Nur { $holder } antwortet darauf; geben Sie { $action } ein anderes.
hotkey-action-refused = { $action } nennt das Tastenkürzel „{ $hotkey }“, und Demysto konnte es nicht beanspruchen: { $detail }. Vielleicht hat eine andere Anwendung es bereits.
hotkey-palette-holder = die Liste der Aktionen

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — die Liste der Aktionen öffnen
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Die Arbeitsumgebung hat für { $wanted } noch kein Tastenkürzel übernommen, deshalb antwortet darauf noch nichts. Demysto fragt erneut.
portal-not-taken = Die Arbeitsumgebung hat für { $wanted } kein Tastenkürzel übernommen, deshalb antwortet darauf nichts. Demystos Tastenkürzel werden in ihren eigenen Tastenkürzel-Einstellungen vergeben.
portal-held-under-nothing = Die Arbeitsumgebung hält ein Tastenkürzel für { $wanted } unter gar keiner Kombination, deshalb antwortet darauf noch nichts. Geben Sie ihm eine in den Tastenkürzel-Einstellungen der Arbeitsumgebung selbst.
portal-stopped-answering = Das GlobalShortcuts-Portal der Arbeitsumgebung hat aufgehört zu antworten, deshalb antwortet auch kein Tastenkürzel mehr. Ein Neustart von Demysto fragt sie erneut an; bis dahin erreicht das Tray-Menü alles, was das Tastenkürzel erreicht.
portal-asking-again = So sieht eine Arbeitsumgebung aus, die noch hochfährt: sie nimmt die Anfrage nach einem Tastenkürzel an und vergibt es an nichts. Demysto fragt einige Minuten weiter.
portal-taken-in-the-end = Die Arbeitsumgebung nahm die Tastenkürzel, als sie erneut gefragt wurde.
portal-asked-enough =
    Demysto hat die Arbeitsumgebung über mehrere Minuten { $asked ->
        [one] einmal
       *[other] { $asked }-mal
    } um seine Tastenkürzel gebeten, und sie hat nicht alle übernommen. Es fragt nicht wieder, bis Demysto neu gestartet wird — Demystos Tastenkürzel werden in den Tastenkürzel-Einstellungen der Arbeitsumgebung selbst vergeben, und das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.
portal-refused = Die Arbeitsumgebung hat Demysto die Tastenkürzel nicht gegeben, um die es gebeten hat: { $detail }. Bis sie es tut, antwortet auf keines etwas — vergeben werden sie in ihren Tastenkürzel-Einstellungen, und das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.
portal-unreachable = Demysto konnte das GlobalShortcuts-Portal der Arbeitsumgebung nicht erreichen, deshalb antwortet kein Tastenkürzel: { $detail }. Es kommt mit xdg-desktop-portal, unter KDE und unter GNOME ab Version 48. Das Menü im Infobereich erreicht alles, was das Tastenkürzel erreicht.

## The log folder

folder-uncreatable = { $path } konnte nicht angelegt werden: { $detail }
folder-no-file-manager = Demysto konnte keinen Dateimanager öffnen: { $detail }. Der Ordner ist { $path }.

## The settings file a fresh installation is met by
#
# Prose the user reads in their own editor rather than in a window, and
# translated for the same reason the windows are: it is the first thing a new
# installation says, and it says it in a file.

settings-file-preamble =
    # Demystos Einstellungen.
    #
    # Werden beim Start von Demysto gelesen, und wieder, sobald das
    # Einstellungsfenster sie schreibt — starten Sie Demysto also neu, nachdem Sie
    # diese Datei von Hand bearbeitet haben.
    #
    # Kommentieren Sie das Beispiel unten aus und füllen Sie es aus.
    #
    # `preset` nennt einen Dienst, dessen Gepflogenheiten Demysto kennt: es füllt
    # `base_url` aus und sagt, welche Umgebungsvariable die Dokumentation des Dienstes
    # selbst zu exportieren empfiehlt. Nennen Sie `base_url` selbst für einen Dienst,
    # der keine Voreinstellung hat, oder um zu überschreiben, was eine Voreinstellung
    # einsetzt — etwa einen lokalen Server auf einem eigenen Port.
    #
    # Die Voreinstellungen sind:
    #
    { $presets }
    #
    # Eine mit „ohne Schlüssel“ markierte Voreinstellung ist ein Server auf diesem
    # Rechner, der überhaupt keine Schlüssel hat: ein Anbieter, der sie verwendet,
    # braucht keinen, und es wird keiner gesendet. Alle anderen Voreinstellungen
    # wollen einen.
    #
    # Der Schlüssel wird in der Variablen gesucht, die `api_key_env` nennt, dann in
    # der eigenen Variablen der Voreinstellung, dann in `api_key` hier.
    # `api_key` wegzulassen und stattdessen die Variable zu exportieren hält das
    # Geheimnis aus dieser Datei heraus.
    #
    # `models` führt die Modelle eines Anbieters auf, die Sie verwenden wollen.
    # `vision` sagt, ob eines Bilder annimmt, und wird angegeben statt aus dem
    # Bezeichner erraten, denn ein Name ist keine Fähigkeit.
    #
    # Ein Modell heißt "<Anbieter>/<Modell>", wo immer eines benannt oder gebunden
    # wird. `default_model` ist das, worauf eine Aktion hinausläuft, die kein eigenes
    # Modell bindet, und `default_vision_model` das, worauf sie für ein Bild
    # hinausläuft.
    #
    # `palette_hotkey` ist die Tastenkombination, die die Liste der Aktionen öffnet.
    # Lassen Sie sie weg für die, mit der Demysto ausgeliefert wird. Sie wird als
    # ihre Modifikatoren und dann eine Taste geschrieben — "Ctrl+Alt+Space". Eine
    # Taste, die nichts schreibt — eine Lautstärke- oder Medientaste, oder F13 und
    # aufwärts —, darf für sich stehen. Das Einstellungsfenster nimmt eine für Sie
    # auf, wenn Sie sie lieber drücken als buchstabieren.
    #
    # `language` ist die Sprache, die Demysto spricht: "en", "de", "es", "fr" oder
    # "ru". Lassen Sie sie weg, und Demysto folgt dem Betriebssystem und fällt auf
    # Englisch zurück. { $languageEnv } setzt sich über beides hinweg.
    #
    # `large_selection` ist, wie viele Zeichen eine Auswahl haben darf, bevor Demysto
    # es in der Unterhaltung sagt. Es wird nie etwas abgeschnitten und nie etwas
    # abgelehnt: es ist dafür da, dass ein versehentliches „Alles auswählen“ nicht
    # stillschweigend bezahlt wird. Lassen Sie es weg für { $largeSelection }, oder
    # setzen Sie es auf 0, um gar nicht gewarnt zu werden.
    #
    # `welcomed` ist Demystos eigene Notiz, dass der Ablauf beim ersten Start
    # durchlaufen wurde. Nehmen Sie die Zeile heraus, um beim nächsten Start
    # wieder hindurchgeführt zu werden.
settings-file-preset = #   { $preset }
settings-file-preset-keyless = #   { $preset } (ohne Schlüssel)
