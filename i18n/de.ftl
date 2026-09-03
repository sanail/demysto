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

## The Palette

palette-reading-selection = Lese, was Sie ausgewählt haben…
palette-reading-clipboard = Lese die Zwischenablage…
palette-origin-selection = Auswahl
palette-origin-clipboard = Aus der Zwischenablage
palette-nothing-captured = Es ist nichts ausgewählt, und die Zwischenablage ist leer. Wählen Sie Text aus und drücken Sie das Tastenkürzel noch einmal.
palette-filter = Aktionen filtern…
palette-no-action-matches = Keine Aktion heißt so.
palette-open-accessibility = Einstellungen für Bedienungshilfen öffnen
palette-keys-collecting = Enter zum Ausführen · Esc zurück
palette-keys-choosing = ↑↓ zum Auswählen · Enter zum Ausführen · Esc zum Schließen
palette-keys-closing = Esc zum Schließen

## The Conversation window

result-conversations = Unterhaltungen
result-conversation-unnamed = Unterhaltung
result-nothing-asked-yet = Noch nichts gefragt.
result-quotation-label = Der Text, um den es in dieser Unterhaltung geht
result-show-more = Mehr anzeigen
result-show-less = Weniger anzeigen
result-asking = Frage das Modell…
result-reasoning = Das Modell denkt nach…
result-copy-answer = Antwort kopieren
result-copied = Kopiert
result-stopped = Angehalten
result-continue = Fortsetzen
result-try-again = Noch einmal versuchen
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
settings-unreadable-file = Die Einstellungen überschreiben keine Datei, die sie nicht lesen konnten; hier lässt sich also nichts bearbeiten, bis diese Datei in Ordnung ist. Öffnen Sie sie, berichtigen Sie, was darin steht, und öffnen Sie dieses Fenster erneut.

### Providers

settings-providers = Anbieter
settings-add-provider = Einen Anbieter hinzufügen
settings-remove-provider = Diesen Anbieter entfernen
settings-no-providers = Noch ist kein Anbieter eingerichtet. Fügen Sie einen hinzu, um Fragen stellen zu können.
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
settings-key-in-file = Liegt in der Einstellungsdatei — tippen Sie, um ihn zu ersetzen
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
settings-add-model = Ein Modell hinzufügen
settings-remove-model = Entfernen
settings-model-sees-images = Sieht Bilder
settings-model-verify-with = Prüfen mit
settings-no-models = Noch kein Modell. Rufen Sie die Liste ab, oder tragen Sie eines von Hand ein.
settings-asking-provider = Frage den Anbieter…
settings-provider-offers-nothing = Er bietet kein Modell an.
settings-provider-answered = { $model } hat geantwortet.

### Defaults

settings-defaults = Standardwerte
settings-default-model = Standardmodell — das, was eine Aktion ohne eigenes Modell nimmt
settings-default-vision-model = Standardmodell für Bilder — das, was stattdessen für ein Bild genommen wird
settings-model-none = Keines
settings-model-does-not-see = { $model } (sieht keine Bilder)
settings-large-selection = Warnen ab — wie viele Zeichen eine Auswahl haben darf, bevor Demysto es sagt
settings-large-selection-default = { $characters } — womit Demysto ausgeliefert wird
settings-large-selection-detail = Es wird nie etwas abgeschnitten und nie etwas abgelehnt: die Warnung ist dafür da, dass ein versehentliches „Alles auswählen“ nicht stillschweigend bezahlt wird. Lassen Sie das Feld leer für Demystos eigene Zahl, oder setzen Sie es auf 0, um gar nicht gewarnt zu werden.

### Language

settings-language = Sprache
settings-language-field = Die Sprache, die Demysto spricht
settings-language-follows-system = Dem Betriebssystem folgen
settings-language-detail = Wird mit „Speichern“ unten gespeichert und sofort gesprochen — im Tray-Menü wie in diesem Fenster.
settings-language-from-environment = { $variable } steht auf { $value }; das ist also die Sprache, die Demysto spricht, was auch immer hier gewählt wird.

### Hotkeys

settings-hotkeys = Tastenkürzel
settings-palette-hotkey = Die Palette — was sie über dem öffnet, was Sie gerade lesen
settings-hotkey-record = Aufnehmen
settings-hotkey-clear = Leeren
settings-hotkey-recording = Drücken Sie eine Kombination… Esc zum Beenden
settings-hotkey-default = { $hotkey } — womit Demysto ausgeliefert wird
settings-hotkey-none = Keines — diese Aktion wird über die Palette erreicht
settings-hotkey-rule = Halten Sie mindestens einen Modifikator, oder drücken Sie eine Taste, die für sich nichts schreibt — F13 und aufwärts sind die, die die meisten Tastaturen senden können.
settings-palette-hotkey-detail = Wird mit „Speichern“ unten gespeichert und antwortet sofort darauf.
settings-action-hotkey-detail = Auf diesem Weg wird nach Parametern nicht gefragt — jeder nimmt, was er anbietet.
settings-wayland-hotkeys = Wayland lässt außerdem keine Anwendung ein Tastenkürzel für sich beanspruchen. Demysto bittet das GlobalShortcuts-Portal der Arbeitsumgebung um die Kombinationen unten, und die Arbeitsumgebung entscheidet, worauf jede antwortet — ändern Sie sie in deren eigenen Tastenkürzel-Einstellungen, wo sie unter Demysto aufgeführt sind.

### Logs

settings-logs = Protokolle
settings-logs-detail = Demysto führt ein lokales Protokoll darüber, was es getan hat — welche Aktion, welches Modell, was schiefging — und nie darüber, was Sie angesehen oder was ein Modell gesagt hat. Es wird nichts irgendwohin gesendet. Legen Sie diese Dateien einem Fehlerbericht bei.
settings-open-logs = Den Protokollordner öffnen

### Updates

settings-updates = Aktualisierungen
settings-updates-detail = Demysto sucht beim Start nach einer neuen Version und bietet an, was es findet — installiert wird nichts, bevor Sie es sagen. Jede Aktualisierung ist mit Demystos eigenem Schlüssel signiert und wird damit geprüft, bevor sie eingespielt wird.
settings-version = Dies ist Demysto { $version }.
settings-check-for-update = Nach einer neuen Version suchen
settings-checking = Wird gesucht…
settings-up-to-date = Dies ist die neueste Version, die es gibt.
settings-update-found = Demysto { $version } ist bereit zur Installation.
settings-install-update = Installieren und neu starten
settings-installing = Wird installiert…

### Actions

settings-actions = Aktionen
settings-write-action = Eine Aktion schreiben
settings-actions-detail = Jede Aktion ist eine eigene Datei in <code>actions</code>, sodass eine gespeichert oder jemandem geschickt werden kann. Eingebaute Aktionen werden dort nicht abgelegt: eine zu ändern behält nur das, was Sie geändert haben, und sie zurückzusetzen löscht das wieder. Eine Aktion wird für sich gespeichert, nicht mit „Speichern“ unten.
settings-action-changed = Geändert
settings-action-yours = Ihre
settings-action-edit = Bearbeiten
settings-action-reset = Zurücksetzen
settings-action-delete = Löschen
settings-action-name = Name — was die Palette auflistet
settings-action-name-example = Schlicht umschreiben
settings-action-model = Modell — lassen Sie es beim Standard, außer diese Aktion braucht ein eigenes
settings-action-model-default = Was die Standardwerte sagen
settings-action-hotkey = Tastenkürzel — führt diese Aktion auf dem Ausgewählten aus, ohne den Umweg über die Palette
settings-action-prompt = Prompt
settings-action-prompt-example =
    Erkläre den Text unten. Der Text ist auf {"{{"}selection_language{"}}"}; antworte auf {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> ist das, was Sie ausgewählt haben; <code>{"{{"}ui_language{"}}"}</code> und <code>{"{{"}selection_language{"}}"}</code> sind die Sprache, die Sie lesen, und die, in der der Text sich herausstellte. Alles andere in doppelten geschweiften Klammern ist ein Parameter, nach dem die Palette vor der Ausführung fragt — deklarieren Sie ihn unten.
settings-parameters = Parameter
settings-declare-parameter = Einen Parameter deklarieren
settings-remove-parameter = Entfernen
settings-no-parameters = Keine. Diese Aktion läuft, sobald sie gewählt ist.
settings-parameter-id-example = target
settings-parameter-label-example = In welche Sprache?
settings-parameter-default-example = Was sie anbietet
settings-save-action = Diese Aktion speichern
settings-cancel = Abbrechen
settings-reset-by-saving = Sie ohne Änderung zu speichern stellt die eingebaute wieder her.

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
welcome-language-detail = Das ist die Sprache, in der Sie laut Ihrem Betriebssystem lesen. Ändern Sie sie hier, wenn es nicht stimmt — und in den Einstellungen jederzeit wieder.
welcome-provider-title = Woher die Antworten kommen
welcome-provider-detail = Demysto fragt ein Modell Ihrer Wahl über Ihr eigenes Konto. Wählen Sie den Dienst, fügen Sie den Schlüssel ein, den er Ihnen gegeben hat, und fragen Sie ihn, welche Modelle er anbietet.
welcome-provider-model = Das Modell, das Demysto fragt, sofern eine Aktion nichts anderes sagt
welcome-provider-verify-first = Der Schlüssel wird noch in diesem Schritt beim Anbieter geprüft, damit ein falscher jetzt auffällt und nicht bei Ihrer ersten Frage.
welcome-accessibility-title = Demysto lesen lassen, was Sie ausgewählt haben
welcome-accessibility-detail = Demysto liest eine Auswahl, indem es den Tastendruck zum Kopieren an das schickt, was Sie gerade lesen, und macOS knüpft das an die Berechtigung für Bedienungshilfen. Öffnen Sie „Datenschutz & Sicherheit“ → „Bedienungshilfen“ und schalten Sie Demysto ein.
welcome-open-accessibility = Einstellungen für Bedienungshilfen öffnen
welcome-accessibility-later = Demysto fragt macOS bei jedem Lauf danach; die Berechtigung später zu erteilen wirkt also genauso. Nach einer Aktualisierung wird sie erneut verlangt, denn für macOS ist das eine andere Anwendung.
welcome-autostart-title = Demysto beim Anmelden starten
welcome-autostart-detail = Demysto wartet im Tray auf das Tastenkürzel und kann nur antworten, solange es läuft. Ohne Ihre Zustimmung hier wird nichts eingetragen, und die Einstellungen Ihres Systems nehmen den Eintrag jederzeit wieder heraus.
welcome-autostart-choice = Beim Anmelden starten
welcome-done-title = Das war alles
welcome-done-detail = Markieren Sie irgendwo Text und drücken Sie { $hotkey }. Die Palette öffnet sich an Ihrem Mauszeiger und zeigt, was Demysto damit tun kann; Enter führt die hervorgehobene Aktion aus.
welcome-done-clipboard = Kopieren Sie Text mit Ctrl+C und drücken Sie dann { $hotkey }. Die Palette öffnet sich an Ihrem Mauszeiger und zeigt, was Demysto damit tun kann; Enter führt die hervorgehobene Aktion aus.
welcome-done-tray = Demysto wartet von nun an im Tray, und sein Menü führt zur Palette, zu den Aktionen und zu den Einstellungen — das Tastenkürzel ist der schnelle Weg, nicht der einzige.

## What an update could not do

update-refused = Demysto konnte nicht nachfragen, ob es eine neue Version gibt: { $detail }
update-install-refused = Die Aktualisierung konnte nicht installiert werden: { $detail }
update-nothing-found = Es gibt keine Aktualisierung zu installieren: suchen Sie zuerst nach einer neuen Version.

## What the login items would not do

autostart-refused = Demysto konnte den Start beim Anmelden nicht ändern: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Dies ist eine Wayland-Sitzung, und Wayland lässt eine Anwendung nicht in eine andere schreiben. Demysto kann nicht lesen, was Sie ausgewählt haben: kopieren Sie es selbst mit Ctrl+C, drücken Sie dann das Tastenkürzel, und Demysto liest die Zwischenablage.
capture-clipboard-unavailable = Die Zwischenablage ist nicht verfügbar: { $detail }
capture-keystroke-refused = Der Tastendruck zum Kopieren konnte nicht gesendet werden: { $detail }
capture-no-accessibility = macOS lässt Demysto nicht lesen, was Sie ausgewählt haben: Demysto braucht die Berechtigung für Bedienungshilfen. Öffnen Sie „Datenschutz & Sicherheit“ → „Bedienungshilfen“ und schalten Sie Demysto ein.
accessibility-pane-unreachable = Demysto konnte die Systemeinstellungen nicht öffnen: { $detail }. Die Berechtigung liegt unter „Datenschutz & Sicherheit“ → „Bedienungshilfen“.
accessibility-only-macos = Nur macOS fragt nach einer Berechtigung, bevor Demysto lesen darf, was Sie ausgewählt haben.

## What stopped a Run

run-nothing-to-run = Es gibt nichts, worauf sich eine Aktion ausführen ließe: wählen Sie Text aus oder kopieren Sie ihn, und drücken Sie das Tastenkürzel noch einmal.
run-no-conversation = Es gibt keine Unterhaltung, in der sich das fragen ließe. Drücken Sie das Tastenkürzel, um eine zu beginnen.
run-no-such-action = Es gibt keine Aktion namens „{ $action }“. Sie wurde vielleicht entfernt, seit die Palette sich geöffnet hat; drücken Sie das Tastenkürzel noch einmal.
run-nothing-to-retry = Es gibt keine Runde, die sich wiederholen ließe. Stellen Sie die Frage erneut, um eine neue zu beginnen.

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
provider-went-quiet = { $provider } ist mitten in der Antwort verstummt, deshalb hat Demysto aufgehört zu warten.
provider-stopped-answering = { $provider } hat mitten in der Antwort aufgehört zu antworten: { $detail }
provider-closed-early = { $provider } hat die Verbindung geschlossen, bevor die Antwort fertig war.
provider-refused = Der Anbieter hat die Anfrage abgelehnt (HTTP { $status }).
provider-refused-saying = Der Anbieter hat die Anfrage abgelehnt (HTTP { $status }): { $detail }
provider-malformed = Die Antwort des Anbieters war keine, die Demysto lesen konnte ({ $reason }): { $body }
provider-no-answer-in-it = sie enthält keine Antwort

## What an Action could not be made into

action-file-preamble = # Eine Aktion, die Demysto ausführt. Bearbeiten Sie sie hier oder in Demystos Einstellungen.
action-needs-name = Eine Aktion braucht einen Namen, unter dem sie aufgeführt wird.
action-needs-prompt = Eine Aktion braucht einen Prompt: das, was sie dem Modell sagt, mit {"{{"}selection{"}}"} dort, wo die Auswahl hingehört.
action-accepts-nothing = Eine Aktion, die keinerlei Auswahl annimmt, könnte nie in der Palette auftauchen.
action-parameter-needs-name = Ein Parameter braucht einen Namen, um im Prompt als {"{{"}like_this{"}}"} geschrieben zu werden.
action-parameter-reserved = Ein Parameter kann nicht „{ $parameter }“ heißen: so greift ein Prompt auf etwas zu, das Demysto selbst einsetzt, und niemand käme je dazu, ihn abzufragen.
action-parameter-needs-label = Der Parameter „{ $parameter }“ braucht eine Beschriftung; damit fragt die Palette nach ihm.
action-parameter-twice = Zwei Parameter heißen „{ $parameter }“, deshalb könnte {"{{"}{ $parameter }{"}}"} im Prompt jeden von beiden meinen.
action-binds-nothing-configured = Diese Aktion bindet das Modell „{ $model }“, und es ist überhaupt kein Modell eingerichtet.
action-binds-unknown-model = Diese Aktion bindet das Modell „{ $model }“, und kein Anbieter bietet eines dieses Namens an. Eingerichtet sind die Modelle: { $models }.
action-id-not-a-file-name = „{ $action }“ kann kein Dateiname sein, deshalb lässt sich darunter keine Aktion aufbewahren.
action-none-to-remove = Es gibt keine Aktion namens „{ $action }“, die sich entfernen ließe. Sie wurde vielleicht schon gelöscht; öffnen Sie dieses Fenster erneut.
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
hotkey-palette-unclaimable = Demysto konnte { $hotkey } nicht beanspruchen, das Tastenkürzel, das die Palette öffnet: { $detail }. Vielleicht hat es schon eine andere Anwendung. Das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.
hotkey-palette-not-a-combination = Die Einstellungen nennen für die Palette das Tastenkürzel „{ $hotkey }“, und das ist keine Kombination, die Demysto versteht.
hotkey-palette-types-something = Die Einstellungen nennen für die Palette das Tastenkürzel „{ $hotkey }“, und das ist eine einzelne Taste, die etwas schreibt. Ein Tastenkürzel wird überall beansprucht, eine Taste für sich muss also eine sein, die nichts erreicht, worin Sie gerade geschrieben haben.
hotkey-palette-refused = Die Einstellungen nennen für die Palette das Tastenkürzel „{ $hotkey }“, und Demysto konnte es nicht beanspruchen: { $detail }. Vielleicht hat es schon eine andere Anwendung.
hotkey-action-not-a-combination = { $action } nennt das Tastenkürzel „{ $hotkey }“, und das ist keine Kombination, die Demysto versteht. Ein Tastenkürzel sind seine Modifikatoren und dann eine Taste, geschrieben wie „Ctrl+Shift+E“.
hotkey-action-types-something = { $action } nennt das Tastenkürzel „{ $hotkey }“, und das ist eine einzelne Taste, die etwas schreibt. Ein Tastenkürzel wird überall beansprucht, eine Taste für sich muss also eine sein, die nichts erreicht, worin Sie gerade geschrieben haben — Pause, ScrollLock, PrintScreen, F13 und aufwärts, oder eine Lautstärke- oder Medientaste. Alles andere braucht einen Modifikator.
hotkey-action-already-held = { $action } nennt das Tastenkürzel „{ $hotkey }“, und { $holder } hat es bereits. Nur { $holder } antwortet darauf; geben Sie { $action } ein anderes.
hotkey-action-refused = { $action } nennt das Tastenkürzel „{ $hotkey }“, und Demysto konnte es nicht beanspruchen: { $detail }. Vielleicht hat es schon eine andere Anwendung.
hotkey-palette-holder = die Palette

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — die Palette öffnen
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Die Arbeitsumgebung hat für { $wanted } noch kein Tastenkürzel übernommen, deshalb antwortet darauf noch nichts. Demysto fragt erneut.
portal-not-taken = Die Arbeitsumgebung hat für { $wanted } kein Tastenkürzel übernommen, deshalb antwortet darauf nichts. Demystos Tastenkürzel werden in ihren eigenen Tastenkürzel-Einstellungen vergeben.
portal-held-under-nothing = Die Arbeitsumgebung hält ein Tastenkürzel für { $wanted } unter gar keiner Kombination, deshalb antwortet darauf noch nichts. Geben Sie ihm eine in den Tastenkürzel-Einstellungen der Arbeitsumgebung selbst.
portal-stopped-answering = Das GlobalShortcuts-Portal der Arbeitsumgebung hat aufgehört zu antworten, deshalb antwortet auch kein Tastenkürzel mehr. Ein Neustart von Demysto fragt sie erneut an; bis dahin erreicht das Tray-Menü alles, was das Tastenkürzel erreicht.
portal-asking-again = So sieht eine Arbeitsumgebung aus, die noch hochfährt: sie nimmt die Anfrage nach einem Tastenkürzel an und gibt sie an nichts weiter, oder beantwortet sie nie. Demysto fragt ein paar Minuten lang weiter und lässt die Arbeitsumgebung dann in Ruhe.
portal-taken-in-the-end = Die Arbeitsumgebung hat die Tastenkürzel übernommen, um die Demysto gebeten hatte, als sie erneut gefragt wurde.
portal-asked-enough =
    Demysto hat die Arbeitsumgebung über mehrere Minuten { $asked ->
        [one] einmal
       *[other] { $asked }-mal
    } um seine Tastenkürzel gebeten, und sie hat nicht alle übernommen. Es fragt nicht wieder, bis Demysto neu gestartet wird — Demystos Tastenkürzel werden in den Tastenkürzel-Einstellungen der Arbeitsumgebung selbst vergeben, und das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.
portal-refused = Die Arbeitsumgebung hat Demysto die Tastenkürzel nicht gegeben, um die es gebeten hat: { $detail }. Bis sie es tut, antwortet auf keines etwas — vergeben werden sie in ihren Tastenkürzel-Einstellungen, und das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.
portal-unreachable = Dies ist eine Wayland-Sitzung, in der Demysto die Arbeitsumgebung über ihr GlobalShortcuts-Portal um ein Tastenkürzel bitten muss — und es konnte keines erreichen: { $detail }. Kein Tastenkürzel antwortet. Das Portal kommt mit xdg-desktop-portal, unter KDE und unter GNOME ab Version 48. Das Tray-Menü erreicht alles, was das Tastenkürzel erreicht.

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
    # `palette_hotkey` ist die Tastenkombination, die die Palette öffnet. Lassen Sie
    # sie weg für die, mit der Demysto ausgeliefert wird. Sie wird als ihre
    # Modifikatoren und dann eine Taste geschrieben — "Ctrl+Alt+Space" — und eine
    # Taste, die nichts schreibt, etwa F13, darf für sich stehen. Das
    # Einstellungsfenster nimmt eine für Sie auf, wenn Sie sie lieber drücken als
    # buchstabieren.
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
