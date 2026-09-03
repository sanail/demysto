# Demysto's interface in Spanish.
#
# Written against `en.ftl`, which is where a message is added first. Every
# identifier there exists here: the suite reads both files and fails the build
# over one this catalogue is missing, and over one it holds that English does
# not.

## The application itself

app-name = Demysto
tray-open = Abrir Demysto
tray-actions = Acciones
tray-update = Actualizar a { $version }…
tray-settings = Ajustes…
tray-quit = Salir de Demysto

# macOS only, and only for the key equivalents: `menu` says why the menu bar
# exists at all and why nothing else is on it.
menu-edit = Edición
menu-quit = Salir de Demysto

# The one thing Demysto raises a notification for: a Run started from an
# Action's own Hotkey that failed with no window on screen to say so.
notification-stopped-part-way = Demysto se ha detenido a medio camino
notification-could-not-answer = Demysto no ha podido responder

## The Actions Demysto comes with
#
# Their names and the Parameters they collect, which is what the Palette shows.
# Their prompt templates stay in `action`, in English, because they are
# addressed to a Model rather than to a person.

action-explain-name = Explicar
action-translate-name = Traducir
action-translate-target-label = ¿A qué idioma?
action-summarize-name = Resumir

## The Palette

palette-reading-selection = Leyendo lo que has seleccionado…
palette-reading-clipboard = Leyendo el portapapeles…
palette-origin-selection = Selección
palette-origin-clipboard = Del portapapeles
palette-nothing-captured = No hay nada seleccionado y el portapapeles está vacío. Selecciona un texto y pulsa el atajo otra vez.
palette-filter = Filtrar acciones…
palette-no-action-matches = Ninguna acción se llama así.
palette-back = Atrás
palette-next = Siguiente
palette-run = Ejecutar
palette-open-accessibility = Abrir los ajustes de Accesibilidad
palette-keys-collecting = Intro para ejecutar · Esc para volver
palette-keys-choosing = ↑↓ para elegir · Intro para ejecutar · Esc para cerrar
palette-keys-closing = Esc para cerrar

## The Conversation window

result-conversations = Conversaciones
result-conversation-unnamed = Conversación
result-nothing-asked-yet = Todavía no se ha preguntado nada.
result-quotation-label = El texto del que trata esta conversación
result-show-more = Mostrar más
result-show-less = Mostrar menos
result-asking = Preguntando al modelo…
result-reasoning = El modelo está razonando…
result-copy-answer = Copiar la respuesta
result-copied = Copiado
result-stopped = Detenido
result-continue = Continuar
result-try-again = Intentarlo de nuevo
result-ask-another-model = Preguntar a otro modelo…
result-open-provider-settings = Abrir los ajustes de { $provider }
result-open-accessibility = Abrir los ajustes de Accesibilidad
result-follow-up = Preguntar algo más…
result-stop = Detener
result-ask = Preguntar
result-keys = Intro para preguntar, Mayús+Intro para una línea nueva, Esc para cerrar

## A rendered code block, whose copy button is markup rather than a component

code-copy = Copiar
code-copied = Copiado

## Settings

settings-window-title = Ajustes de Demysto
settings-title = Ajustes
settings-save = Guardar
settings-saving = Guardando…
settings-saved = Guardado.
settings-keys = Esc para cerrar
settings-reading = Leyendo los ajustes…
settings-unreadable-file = Los ajustes no sobrescriben un archivo que no han podido leer, así que aquí no se puede editar nada hasta que ese archivo esté reparado. Ábrelo, corrige lo que dice y vuelve a abrir esta ventana.

### Providers

settings-providers = Proveedores
settings-add-provider = Añadir un proveedor
settings-remove-provider = Quitar este proveedor
settings-no-providers = Todavía no hay ningún proveedor configurado. Añade uno para empezar a preguntar cosas.
settings-provider-name = Nombre
settings-provider-name-example = openai
settings-provider-service = Servicio
settings-provider-no-preset = Sin preajuste
settings-provider-preset-keyless = { $preset } (sin clave)
settings-provider-base-url = URL base
settings-provider-base-url-from-preset = URL base — déjala vacía para usar la del preajuste
settings-provider-base-url-example = https://api.example.com/v1
settings-provider-key = Clave de API
settings-provider-key-variable = O la variable de entorno que la guarda
settings-provider-key-variable-example = MY_API_KEY
settings-key-in-file = Guardada en el archivo de ajustes — escribe para reemplazarla
settings-key-in-environment = Tomada de { $variable }
settings-key-not-needed = Este servicio no tiene claves
settings-key-missing = Todavía no hay clave
settings-key-going = Se quitará al guardar
settings-keep-key = Mantener la clave en el archivo
settings-remove-key = Quitar la clave del archivo

### The Models one Provider offers

settings-models = Modelos
settings-fetch-models = Consultar
settings-verify-key = Verificar la clave
settings-add-model = Añadir un modelo
settings-remove-model = Quitar
settings-model-sees-images = Ve imágenes
settings-model-verify-with = Verificar con
settings-no-models = Todavía no hay ningún modelo. Consulta la lista, o añade uno a mano.
settings-asking-provider = Preguntando al proveedor…
settings-provider-offers-nothing = No ofrece ningún modelo.
settings-provider-answered = { $model } ha respondido.

### Defaults

settings-defaults = Valores por defecto
settings-default-model = Modelo por defecto — el que usa una acción que no tiene uno propio
settings-default-vision-model = Modelo de visión por defecto — el que se usa en su lugar para una imagen
settings-model-none = Ninguno
settings-model-does-not-see = { $model } (no ve imágenes)
settings-large-selection = Avisar a partir de — cuántos caracteres puede tener una selección antes de que Demysto lo diga
settings-large-selection-default = { $characters } — lo que trae Demysto
settings-large-selection-detail = Nunca se corta nada y nunca se rechaza nada: el aviso está para que un «seleccionar todo» accidental no se pague en silencio. Deja el campo vacío para la cifra de Demysto, o ponlo a 0 para que no te diga nada.

### Language

settings-language = Idioma
settings-language-field = El idioma en el que habla Demysto
settings-language-follows-system = Seguir al sistema operativo
settings-language-detail = Se guarda con el botón Guardar de abajo, y se habla en cuanto se guarda: tanto el menú de la bandeja como esta ventana.
settings-language-from-environment = { $variable } vale { $value }, así que ese es el idioma en el que habla Demysto, se elija lo que se elija aquí.

### Hotkeys

settings-hotkeys = Atajos de teclado
settings-palette-hotkey = La paleta — lo que la abre sobre aquello que estés leyendo
settings-hotkey-record = Grabar
settings-hotkey-clear = Borrar
settings-hotkey-cancel = Cancelar
settings-hotkey-recording = Pulsa una combinación… Esc para parar
settings-hotkey-default = { $hotkey } — lo que trae Demysto
settings-hotkey-none = Ninguno — a esta acción se llega por la paleta
settings-hotkey-rule = Mantén pulsado al menos un modificador, o pulsa una tecla que por sí sola no escriba nada: F13 y superiores son las que más teclados pueden enviar.
settings-palette-hotkey-detail = Se guarda con el botón Guardar de abajo, y responde en cuanto se guarda.
settings-action-hotkey-detail = Por este camino no se piden los parámetros: cada uno toma lo que ofrece.
settings-wayland-hotkeys = Wayland tampoco deja que ninguna aplicación se reserve un atajo para sí. Demysto pide las combinaciones de abajo al portal GlobalShortcuts del escritorio, y es el escritorio quien decide a qué responde cada una: cámbialas en los ajustes de atajos de teclado del propio escritorio, donde aparecen bajo Demysto.

### Logs

settings-logs = Registros
settings-logs-detail = Demysto guarda un registro local de lo que hizo —qué acción, qué modelo, qué salió mal— y nunca de lo que estabas mirando ni de lo que dijo un modelo. No se envía nada a ninguna parte. Adjunta estos archivos a un informe de error.
settings-open-logs = Abrir la carpeta de los registros

### Updates

settings-updates = Actualizaciones
settings-updates-detail = Demysto busca una versión nueva al arrancar y ofrece lo que encuentra: no se instala nada hasta que lo digas. Cada actualización va firmada con la clave propia de Demysto y se comprueba con ella antes de instalarse.
settings-version = Esto es Demysto { $version }.
settings-check-for-update = Buscar una versión nueva
settings-checking = Buscando…
settings-up-to-date = Esta es la versión más reciente que hay.
settings-update-found = Demysto { $version } está listo para instalarse.
settings-install-update = Instalar y reiniciar
settings-installing = Instalando…

### Actions

settings-actions = Acciones
settings-write-action = Escribir una acción
settings-actions-detail = Cada acción es un archivo propio en <code>actions</code>, así que se puede guardar en una copia de seguridad o enviar a alguien. Las acciones integradas no se escriben ahí: cambiar una guarda solo lo que has cambiado, y restablecerla lo borra. Una acción se guarda por su cuenta, no con el botón Guardar de abajo.
settings-action-changed = Cambiada
settings-action-yours = Tuya
settings-action-edit = Editar
settings-action-reset = Restablecer
settings-action-delete = Eliminar
settings-action-name = Nombre — lo que lista la paleta
settings-action-name-example = Reescribir en llano
settings-action-model = Modelo — déjalo en el valor por defecto salvo que esta acción necesite uno propio
settings-action-model-default = Lo que digan los valores por defecto
settings-action-hotkey = Atajo — ejecuta esta acción sobre lo que tengas seleccionado, sin pasar por la paleta
settings-action-prompt = Prompt
settings-action-prompt-example =
    Explica el texto de abajo. El texto está en {"{{"}selection_language{"}}"}; responde en {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> es lo que has seleccionado; <code>{"{{"}ui_language{"}}"}</code> y <code>{"{{"}selection_language{"}}"}</code> son el idioma en el que lees y aquel en el que resultó estar. Cualquier otra cosa entre llaves dobles es un parámetro, que la paleta pide antes de la ejecución: decláralo abajo.
settings-parameters = Parámetros
settings-declare-parameter = Declarar un parámetro
settings-remove-parameter = Quitar
settings-no-parameters = Ninguno. Esta acción se ejecuta en cuanto se elige.
settings-parameter-id-example = target
settings-parameter-label-example = ¿A qué idioma?
settings-parameter-default-example = Lo que ofrece
settings-save-action = Guardar esta acción
settings-cancel = Cancelar
settings-reset-by-saving = Guardarla sin haber cambiado nada devuelve la integrada.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Bienvenido a Demysto
welcome-step = Paso { $at } de { $total }
welcome-back = Atrás
welcome-continue = Continuar
welcome-finish = Empezar a usar Demysto
welcome-language-title = Demysto ha encontrado tu idioma
welcome-language-detail = Este es el idioma en el que lees según tu sistema operativo. Cámbialo aquí si no es así, y vuelve a cambiarlo en los ajustes cuando quieras.
welcome-provider-title = De dónde vienen las respuestas
welcome-provider-detail = Demysto le pregunta a un modelo que eliges tú, con tu propia cuenta. Elige el servicio, pega la clave que te dio y pregúntale qué modelos ofrece.
welcome-provider-model = El modelo al que pregunta Demysto salvo que una acción diga otra cosa
welcome-provider-verify-first = La clave se le presenta al proveedor antes de terminar este paso, para que una equivocada se descubra ahora y no en tu primera pregunta.
welcome-accessibility-title = Deja que Demysto lea lo que has seleccionado
welcome-accessibility-detail = Demysto lee una selección enviando la pulsación de copiar a lo que estés leyendo, y macOS lo condiciona al permiso de Accesibilidad. Abre Privacidad y seguridad → Accesibilidad y activa Demysto.
welcome-open-accessibility = Abrir los ajustes de Accesibilidad
welcome-accessibility-later = Demysto le pregunta esto a macOS en cada ejecución, así que concederlo más tarde funciona igual de bien. Vuelve a pedirse después de una actualización, que para macOS es otra aplicación.
welcome-autostart-title = Iniciar Demysto al iniciar sesión
welcome-autostart-detail = Demysto espera en la bandeja a que pulses el atajo, así que solo puede responder mientras se está ejecutando. No se registra nada si no lo pides aquí, y los ajustes de tu sistema pueden quitarlo de nuevo.
welcome-autostart-choice = Iniciarlo al iniciar sesión
welcome-done-title = Eso es todo
welcome-done-detail = Selecciona texto en cualquier sitio y pulsa { $hotkey }. La paleta se abre junto al cursor con lo que Demysto puede hacer con él, e Intro ejecuta lo que esté resaltado.
welcome-done-clipboard = Copia texto con Ctrl+C y pulsa después { $hotkey }. La paleta se abre junto al cursor con lo que Demysto puede hacer con él, e Intro ejecuta lo que esté resaltado.
welcome-done-tray = A partir de ahora Demysto espera en la bandeja, y su menú llega a la paleta, a las acciones y a los ajustes: el atajo es el camino rápido, no el único.

## What an update could not do

update-refused = Demysto no ha podido preguntar si hay una versión nueva: { $detail }
update-install-refused = La actualización no se ha podido instalar: { $detail }
update-nothing-found = No hay ninguna actualización que instalar: busca antes una versión nueva.

## What the login items would not do

autostart-refused = Demysto no ha podido cambiar si se inicia al iniciar sesión: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Esta es una sesión de Wayland, y Wayland no deja que una aplicación escriba en otra. Demysto no puede leer lo que has seleccionado: cópialo tú con Ctrl+C, pulsa después el atajo y Demysto leerá el portapapeles.
capture-clipboard-unavailable = El portapapeles no está disponible: { $detail }
capture-keystroke-refused = No se ha podido enviar la pulsación de copiar: { $detail }
capture-no-accessibility = macOS no deja que Demysto lea lo que has seleccionado: Demysto necesita el permiso de Accesibilidad. Abre Privacidad y seguridad → Accesibilidad y activa Demysto.
accessibility-pane-unreachable = Demysto no ha podido abrir Ajustes del Sistema: { $detail }. El permiso está en Privacidad y seguridad → Accesibilidad.
accessibility-only-macos = Solo macOS pide un permiso antes de que Demysto pueda leer lo que has seleccionado.

## What stopped a Run

run-nothing-to-run = No hay nada sobre lo que ejecutar una acción: selecciona un texto, o cópialo, y pulsa el atajo otra vez.
run-no-conversation = No hay ninguna conversación en la que preguntar esto. Pulsa el atajo para empezar una.
run-no-such-action = No hay ninguna acción llamada «{ $action }». Puede que la hayan quitado desde que se abrió la paleta; pulsa el atajo otra vez.
run-nothing-to-retry = No hay ningún turno que repetir. Vuelve a hacer la pregunta para empezar uno nuevo.

# The one warning a Conversation carries, said before the Model is asked so that
# it is on screen while the answer is still being paid for.
run-large-selection =
    Esta selección tiene { $shown } { $characters ->
        [one] carácter
       *[other] caracteres
    }, más de los { $limit } que { $setting } fija en { $path }. Se ha enviado entera —no se ha cortado nada—, así que cuesta lo que eso cueste.

## What the settings file could not be made into

config-unreadable = No se ha podido leer { $path }: { $detail }
config-unwritable = No se ha podido escribir { $path }: { $detail }
config-not-toml-at-line = { $path } no es TOML válido en la línea { $line }: { $detail }
config-not-toml = { $path } no es TOML válido: { $detail }
config-newer-version = { $path } dice ser de la versión { $stated }, y este Demysto entiende la versión { $understood }; actualiza Demysto, o apunta { $variable } a otro directorio
config-uneditable = No se ha podido editar { $path } sin perder lo que hay escrito en él, así que no se ha guardado nada.
config-no-provider = no hay ningún proveedor configurado; abre { $path } y rellena el ejemplo que trae
config-in-file = { $reason } en { $path }
config-provider-no-name = hay un proveedor configurado sin nombre
config-provider-name-has-separator = el proveedor «{ $provider }» tiene un «{ $separator }» en el nombre, que es lo que separa un proveedor de un modelo
config-two-providers-named = dos proveedores se llaman «{ $provider }», así que no se puede nombrar un modelo de ninguno de los dos
config-provider-model-no-name = el proveedor «{ $provider }» lista un modelo sin nombre
config-provider-model-twice = el proveedor «{ $provider }» lista el modelo «{ $model }» dos veces
config-provider-no-base-url = el proveedor «{ $provider }» en { $path } no indica ni base_url ni un preajuste del que tomarla
config-no-key-anywhere = El proveedor «{ $provider }» no tiene clave de API: pon api_key para él en { $path }, o nombra una variable de entorno en api_key_env.
config-no-key-export = El proveedor «{ $provider }» no tiene clave de API: exporta { $variables }, o pon api_key para él en { $path }.
config-no-such-preset = No hay ningún preajuste llamado «{ $preset }».

## Which Model a Run resolves to, when it resolves to none

model-none-configured = No hay ningún modelo configurado; añade uno a un proveedor de ahí.
model-configured-are = Los modelos configurados ahí son: { $models }.
model-action-binds-nothing = Esta acción está atada al modelo «{ $model }», y ningún proveedor de { $path } ofrece uno con ese nombre. { $offered }
model-setting-names-nothing = { $setting } en { $path } nombra el modelo «{ $model }», y ningún proveedor de ahí ofrece uno con ese nombre. { $offered }
model-nothing-nominated = En { $path } no se designa ningún { $setting }. { $offered }
model-nomination-none-configured = { $setting } nombra el modelo «{ $model }», y no hay ningún modelo configurado.
model-nomination-unknown = { $setting } nombra el modelo «{ $model }», y ningún proveedor ofrece uno con ese nombre. Los modelos configurados son: { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto no ha podido abrir una conexión: { $detail }
provider-timed-out =
    { $provider } no ha respondido en { $seconds ->
        [one] un segundo
       *[other] { $seconds } segundos
    }, así que Demysto ha dejado de esperar.
provider-unreachable = No se ha podido contactar con { $provider }: { $detail }
provider-went-quiet = { $provider } se ha quedado callado a media respuesta, así que Demysto ha dejado de esperar.
provider-stopped-answering = { $provider } ha dejado de responder a media respuesta: { $detail }
provider-closed-early = { $provider } ha cerrado la conexión antes de terminar la respuesta.
provider-refused = El proveedor ha rechazado la petición (HTTP { $status }).
provider-refused-saying = El proveedor ha rechazado la petición (HTTP { $status }): { $detail }
provider-malformed = La respuesta del proveedor no era una que Demysto pudiera leer ({ $reason }): { $body }
provider-no-answer-in-it = no contiene ninguna respuesta

## What an Action could not be made into

action-file-preamble = # Una acción que ejecuta Demysto. Edítala aquí, o en los ajustes de Demysto.
action-needs-name = Una acción necesita un nombre bajo el que aparecer en la lista.
action-needs-prompt = Una acción necesita un prompt: lo que le dice al modelo, con {"{{"}selection{"}}"} donde va la selección.
action-accepts-nothing = Una acción que no acepta ningún tipo de selección nunca podría aparecer en la paleta.
action-parameter-needs-name = Un parámetro necesita un nombre para poder escribirse como {"{{"}like_this{"}}"} en el prompt.
action-parameter-reserved = Un parámetro no puede llamarse «{ $parameter }»: así es como un prompt alcanza algo que rellena el propio Demysto, de modo que nadie llegaría a pedirlo nunca.
action-parameter-needs-label = El parámetro «{ $parameter }» necesita una etiqueta, que es con lo que la paleta lo pide.
action-parameter-twice = Dos parámetros se llaman «{ $parameter }», así que {"{{"}{ $parameter }{"}}"} en el prompt podría ser cualquiera de los dos.
action-binds-nothing-configured = Esta acción ata el modelo «{ $model }», y no hay ningún modelo configurado.
action-binds-unknown-model = Esta acción ata el modelo «{ $model }», y ningún proveedor ofrece uno con ese nombre. Los modelos configurados son: { $models }.
action-id-not-a-file-name = «{ $action }» no puede ser el nombre de un archivo, así que no se puede guardar ninguna acción con él.
action-none-to-remove = No hay ninguna acción llamada «{ $action }» que quitar. Puede que ya se haya eliminado; vuelve a abrir esta ventana.
action-file-newer-version = { $path } dice ser de la versión { $stated }, y este Demysto entiende la versión { $understood }. Actualiza Demysto, o saca el archivo de ese directorio.
action-file-states-no-field = { $path } no indica { $field }. Una acción que Demysto no tenga ya debe indicar su nombre y su plantilla.
action-file-unreadable = No se ha podido leer { $path }: { $detail }
action-dir-unreadable = No se ha podido leer { $path }, así que las acciones que hay en él no aparecen en la lista: { $detail }
action-file-unwritable = No se ha podido escribir { $path }: { $detail }
action-file-unwritable-shape = No se ha podido escribir { $path } como TOML: { $detail }
action-file-invalid-at-line = { $path } no es una acción válida en la línea { $line }: { $detail }
action-file-invalid = { $path } no es una acción válida: { $detail }

## Hotkeys the desktop would not give up

hotkey-palette-fell-back = { $why } Demysto está usando { $hotkey } en su lugar.
hotkey-palette-unclaimable = Demysto no ha podido reservar { $hotkey }, el atajo que abre la paleta: { $detail }. Puede que ya lo tenga otra aplicación. El menú de la bandeja llega a todo lo que llega el atajo.
hotkey-palette-not-a-combination = Los ajustes indican el atajo «{ $hotkey }» para la paleta, y no es una combinación que Demysto entienda.
hotkey-palette-types-something = Los ajustes indican el atajo «{ $hotkey }» para la paleta, y es una sola tecla que escribe algo. Un atajo se reserva en todo el sistema, así que una tecla sola tiene que ser una que no llegue a aquello en lo que estabas escribiendo.
hotkey-palette-refused = Los ajustes indican el atajo «{ $hotkey }» para la paleta, y Demysto no ha podido reservarlo: { $detail }. Puede que ya lo tenga otra aplicación.
hotkey-action-not-a-combination = { $action } indica el atajo «{ $hotkey }», y no es una combinación que Demysto entienda. Un atajo son sus modificadores y después una tecla, escrito como «Ctrl+Shift+E».
hotkey-action-types-something = { $action } indica el atajo «{ $hotkey }», y es una sola tecla que escribe algo. Un atajo se reserva en todo el sistema, así que una tecla sola tiene que ser una que no llegue a aquello en lo que estabas escribiendo: Pause, ScrollLock, PrintScreen, F13 y superiores, o una tecla de volumen o de reproducción. Cualquier otra necesita un modificador.
hotkey-action-already-held = { $action } indica el atajo «{ $hotkey }», y { $holder } ya lo tiene. Solo { $holder } responde a él; dale otro a { $action }.
hotkey-action-refused = { $action } indica el atajo «{ $hotkey }», y Demysto no ha podido reservarlo: { $detail }. Puede que ya lo tenga otra aplicación.
hotkey-palette-holder = la paleta

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — abrir la paleta
portal-action-description = Demysto — { $action }
portal-not-taken-yet = El escritorio todavía no ha tomado un atajo para { $wanted }, así que aún no responde nada. Demysto lo está pidiendo otra vez.
portal-not-taken = El escritorio no ha tomado un atajo para { $wanted }, así que no responde nada. Los atajos de Demysto se asignan en los ajustes de atajos de teclado del escritorio.
portal-held-under-nothing = El escritorio guarda un atajo para { $wanted } sin ninguna combinación, así que aún no responde nada. Dale una en los ajustes de atajos de teclado del propio escritorio.
portal-stopped-answering = El portal GlobalShortcuts del escritorio ha dejado de responder, así que tampoco responde ningún atajo. Reiniciar Demysto vuelve a pedirlos; mientras tanto, el menú de la bandeja llega a todo lo que llega el atajo.
portal-asking-again = Así se ve un escritorio que todavía está arrancando: toma la petición de un atajo y se la da a nada, o no la responde nunca. Demysto sigue pidiendo unos minutos, y después deja al escritorio en paz.
portal-taken-in-the-end = El escritorio ha tomado los atajos que Demysto pedía cuando se le ha preguntado otra vez.
portal-asked-enough =
    Demysto ha pedido sus atajos al escritorio { $asked ->
        [one] una vez
       *[other] { $asked } veces
    } a lo largo de varios minutos, y no los ha tomado todos. No los volverá a pedir hasta que se reinicie Demysto: los atajos de Demysto se asignan en los ajustes de atajos de teclado del propio escritorio, y el menú de la bandeja llega a todo lo que llega el atajo.
portal-refused = El escritorio no le ha dado a Demysto los atajos que ha pedido: { $detail }. Hasta que lo haga no responde nada a ninguno: se asignan en sus ajustes de atajos de teclado, y el menú de la bandeja llega a todo lo que llega el atajo.
portal-unreachable = Esta es una sesión de Wayland, donde Demysto tiene que pedirle un atajo al portal GlobalShortcuts del escritorio, y no ha podido alcanzar ninguno: { $detail }. No responde ningún atajo. El portal viene con xdg-desktop-portal, en KDE y en GNOME a partir de la versión 48. El menú de la bandeja llega a todo lo que llega el atajo.

## The log folder

folder-uncreatable = No se ha podido crear { $path }: { $detail }
folder-no-file-manager = Demysto no ha podido abrir un gestor de archivos: { $detail }. La carpeta es { $path }.

## The settings file a fresh installation is met by
#
# Prose the user reads in their own editor rather than in a window, and
# translated for the same reason the windows are: it is the first thing a new
# installation says, and it says it in a file.

settings-file-preamble =
    # Ajustes de Demysto.
    #
    # Se leen cuando Demysto arranca, y otra vez cada vez que los escribe la ventana
    # de ajustes: reinicia Demysto después de editar este archivo a mano.
    #
    # Descomenta el ejemplo de abajo y rellénalo.
    #
    # `preset` nombra un servicio cuyas convenciones Demysto conoce: rellena
    # `base_url` y dice qué variable de entorno recomienda exportar la documentación
    # del propio servicio. Indica `base_url` tú mismo para un servicio que no tenga
    # preajuste, o para sustituir lo que un preajuste rellena — un servidor local
    # escuchando en un puerto tuyo, por ejemplo.
    #
    # Los preajustes son:
    #
    { $presets }
    #
    # Un preajuste marcado como «sin clave» es un servidor que corre en esta máquina
    # y no tiene claves en absoluto: un proveedor que lo use no necesita ninguna, y no
    # se envía ninguna. Todos los demás preajustes quieren una.
    #
    # La clave se busca en la variable que nombra `api_key_env`, después en la
    # variable del propio preajuste, y después en `api_key` de aquí. Dejar `api_key`
    # fuera y exportar la variable en su lugar mantiene el secreto fuera de este
    # archivo.
    #
    # `models` lista los modelos de un proveedor que quieras usar. `vision` dice si
    # uno acepta imágenes, y se indica en vez de deducirse del identificador, porque
    # un nombre no es una capacidad.
    #
    # Un modelo se nombra "<proveedor>/<modelo>" allí donde se designa o se ata uno.
    # `default_model` es a lo que se reduce una acción que no ata ningún modelo
    # propio, y `default_vision_model` es a lo que se reduce para una imagen.
    #
    # `palette_hotkey` es la combinación de teclas que abre la paleta. Déjalo fuera
    # para usar la que trae Demysto. Se escribe como sus modificadores y después una
    # tecla — "Ctrl+Alt+Space" —, y una tecla que no escribe nada, como F13, puede ir
    # sola. La ventana de ajustes graba una por ti si prefieres pulsarla a
    # deletrearla.
    #
    # `language` es el idioma en el que habla Demysto: "en", "de", "es", "fr" o "ru".
    # Déjalo fuera y Demysto sigue al sistema operativo, recurriendo al inglés.
    # { $languageEnv } se impone a los dos.
    #
    # `large_selection` es cuántos caracteres puede tener una selección antes de que
    # Demysto lo diga en la conversación. Nunca se corta nada y nunca se rechaza nada:
    # está para que un «seleccionar todo» accidental no se pague en silencio. Déjalo
    # fuera para { $largeSelection }, o ponlo a 0 para que no te diga nada.
    #
    # `welcomed` es la nota que Demysto se deja a sí mismo de que el recorrido de
    # la primera ejecución ya se hizo. Quita la línea para volver a recorrerlo en
    # el siguiente inicio.
settings-file-preset = #   { $preset }
settings-file-preset-keyless = #   { $preset } (sin clave)
