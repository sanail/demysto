# Demysto's interface in Russian.
#
# Written against `en.ftl`, which is where a message is added first. Every
# identifier there exists here: the suite reads both files and fails the build
# over one this catalogue is missing, and over one it holds that English does
# not.

## The application itself

app-name = Demysto
tray-open = Открыть Demysto
tray-actions = Действия
tray-update = Обновить до { $version }…
tray-settings = Настройки…
tray-quit = Завершить Demysto

# macOS only, and only for the key equivalents: `menu` says why the menu bar
# exists at all and why nothing else is on it.
menu-edit = Правка
menu-quit = Завершить Demysto

# The one thing Demysto raises a notification for: a Run started from an
# Action's own Hotkey that failed with no window on screen to say so.
notification-stopped-part-way = Demysto остановился на полуслове
notification-could-not-answer = Demysto не смог ответить

## The Actions Demysto comes with
#
# Their names and the Parameters they collect, which is what the Palette shows.
# Their prompt templates stay in `action`, in English, because they are
# addressed to a Model rather than to a person.

action-explain-name = Объяснить
action-translate-name = Перевести
action-translate-target-label = На какой язык?
action-summarize-name = Пересказать
action-describe-image-name = Описать изображение
action-custom-name = Своё…
action-custom-prompt-label = Что сделать?

## The Palette

palette-reading-selection = Читаю выделенное…
palette-reading-clipboard = Читаю буфер обмена…
palette-origin-selection = Выделение
palette-origin-clipboard = Из буфера обмена
palette-picture = Изображение, { $dimensions }
palette-nothing-captured = Ничего не выделено, и буфер обмена пуст. Выделите текст и нажмите горячую клавишу ещё раз.
palette-filter = Фильтр действий…
palette-no-action-matches = Ничего не найдено.
palette-back = Назад
palette-next = Далее
palette-run = Запустить
palette-open-accessibility = Открыть настройки Универсального доступа
palette-keys-collecting = Enter — запустить · Esc — назад
palette-keys-choosing = ↑↓ — выбрать · Enter — запустить · Esc — закрыть
palette-keys-closing = Esc — закрыть

## The Conversation window

result-conversations = Чаты
result-conversation-unnamed = Чат
result-nothing-asked-yet = Здесь пока пусто.
result-quotation-label = Текст этого чата
result-picture-label = Изображение этого чата
result-show-more = Показать целиком
result-show-less = Свернуть
result-ask-at-original = Спросить снова в исходном разрешении — { $weight }
result-picture-let-go = Изображение больше не хранится.
result-sealed = Изображение больше не хранится, поэтому продолжить чат нельзя. Скопируйте его и нажмите горячую клавишу, чтобы начать новый.
result-asking = Спрашиваю модель…
result-reasoning = Модель размышляет…
result-copy-answer = Скопировать ответ
result-copied = Скопировано
result-stopped = Остановлено
result-continue = Продолжить
result-try-again = Повторить
result-ask-another-model = Спросить другую модель…
result-open-provider-settings = Открыть настройки провайдера { $provider }
result-open-accessibility = Открыть настройки Универсального доступа
result-follow-up = Спросить ещё…
result-stop = Остановить
result-ask = Спросить
result-keys = Enter — спросить, Shift+Enter — перенос строки, Esc — закрыть

## A rendered code block, whose copy button is markup rather than a component

code-copy = Копировать
code-copied = Скопировано

## Settings

settings-window-title = Demysto — Настройки
settings-title = Настройки
settings-save = Сохранить
settings-saving = Сохраняю…
settings-saved = Сохранено.
settings-keys = Esc — закрыть
settings-reading = Читаю настройки…
settings-unreadable-file = Изменить ничего нельзя, пока файл настроек не исправлен. Поправьте написанное и откройте это окно снова.

settings-folder = Папка настроек

### The tabs the window is divided into

settings-tab-models = Модели
settings-tab-actions = Действия
settings-tab-general = Общие
settings-tab-about = О программе
settings-tab-unsaved = есть несохранённое
settings-tab-update = обновление готово

### Providers

settings-providers = Провайдеры
settings-add-provider = Добавить провайдера
settings-remove-provider = Удалить этого провайдера
settings-provider-edit = Изменить
settings-provider-unnamed = (без имени)
settings-no-providers = Провайдеров пока нет. Добавьте одного, чтобы начать.
settings-provider-name = Имя
settings-provider-name-example = openai
settings-provider-service = Сервис
settings-provider-no-preset = Без пресета
settings-provider-preset-keyless = { $preset } (без ключа)
settings-provider-base-url = Базовый URL
settings-provider-base-url-from-preset = Базовый URL — оставьте пустым, чтобы взять из пресета
settings-provider-base-url-example = https://api.example.com/v1
settings-provider-key = Ключ API
settings-provider-key-variable = Или переменная окружения, в которой он лежит
settings-provider-key-variable-example = MY_API_KEY
settings-key-in-file = Хранится в файле настроек
settings-key-in-environment = Берётся из { $variable }
settings-key-not-needed = У этого сервиса нет ключей
settings-key-missing = Ключа пока нет
settings-key-going = Будет удалён при сохранении
settings-keep-key = Оставить ключ в файле
settings-remove-key = Удалить ключ из файла

### The Models one Provider offers

settings-models = Модели
settings-fetch-models = Запросить
settings-verify-key = Проверить ключ
settings-verify-which-model = На какой модели?
settings-add-model = Добавить модель
settings-remove-model = Удалить
settings-model-sees-images = Видит изображения
settings-no-models = Моделей пока нет. Запросите список или добавьте модель вручную.
settings-asking-provider = Спрашиваю провайдера…
settings-provider-offers-nothing = Он не предлагает ни одной модели.
settings-provider-answered = { $model } ответила.

### Defaults

settings-defaults = Умолчания
settings-default-model = Модель по умолчанию — если действие не указывает свою
settings-default-vision-model = Модель по умолчанию для изображений
settings-model-none = Нет
settings-model-does-not-see = { $model } (не видит изображений)
settings-large-selection = Предупреждать, если знаков больше
settings-large-selection-default = { $characters } по умолчанию
settings-large-selection-detail = Ничего не обрезается — предупреждение нужно лишь затем, чтобы случайное «выделить всё» не оплатилось незаметно. Оставьте поле пустым, чтобы взять значение Demysto, или поставьте 0, чтобы не предупреждать.

### Language

settings-language = Язык
settings-language-field = Язык интерфейса
settings-language-follows-system = Как в системе
settings-language-from-environment = { $variable } имеет значение { $value }, поэтому Demysto говорит на этом языке, что бы ни было выбрано здесь.

### Hotkeys

settings-hotkeys = Горячие клавиши
settings-palette-hotkey = Открывает список действий
settings-hotkey-record = Записать
settings-hotkey-clear = Очистить
settings-hotkey-cancel = Отмена
settings-hotkey-recording = Нажмите клавиши… Esc — отмена
settings-hotkey-default = { $hotkey } — по умолчанию
settings-hotkey-none = Нет — действие запускается из списка
settings-palette-hotkey-rule = Удерживайте Ctrl, Alt или Shift и нажмите клавишу. Печатающую клавишу занять в одиночку нельзя: она перестанет печататься во всех программах.
settings-action-hotkey-rule = Удерживайте Ctrl, Alt или Shift и нажмите клавишу. Печатающую клавишу занять в одиночку нельзя: она перестанет печататься во всех программах. Параметры при этом не спрашиваются — каждый берёт значение по умолчанию.
settings-wayland-hotkeys = В Wayland горячие клавиши раздаёт окружение, а не Demysto. Меняйте их в настройках клавиатуры самого окружения — там они перечислены под именем Demysto.

### Startup

settings-autostart = Запуск
settings-autostart-choice = Запускать при входе
settings-autostart-changed = Готово
settings-autostart-detail = Список автозапуска ведёт сама система, поэтому флажок срабатывает сразу и снять его можно там же.

### Logs

settings-logs = Журналы
settings-logs-detail = В журнал попадает то, что делал Demysto, — какое действие, какая модель, что пошло не так, — и никогда то, что вы читали или что ответила модель. Прикладывайте его к сообщению об ошибке.
settings-open-logs = Открыть папку с журналами

### Updates

settings-updates = Обновления
settings-updates-detail = Каждое обновление подписано ключом Demysto и проверяется перед установкой, и без вашего согласия ничего не устанавливается.
settings-version = Demysto { $version }
settings-check-for-update = Проверить обновления
settings-checking = Проверяю…
settings-up-to-date = Установлена последняя версия.
settings-update-found = Demysto { $version } готов к установке.
settings-install-update = Установить и перезапустить
settings-installing = Устанавливаю…

### Actions

settings-actions = Действия
settings-write-action = Новое действие
settings-actions-detail = Каждое действие — отдельный файл в <code>actions</code>: его можно сохранить про запас или передать другому.
settings-action-changed = Изменено
settings-action-yours = Ваше
settings-action-unsaved = не сохранено
settings-action-edit = Изменить
settings-action-reset = Сбросить
settings-action-delete = Удалить
settings-action-name = Название
settings-action-name-example = Переписать просто
settings-action-model = Модель
settings-action-model-default = По умолчанию
settings-action-hotkey = Горячая клавиша
settings-action-accepts = Принимает
settings-action-accepts-text = Текст
settings-action-accepts-image = Изображения
settings-action-accepts-detail = Выберите хотя бы одно. Изображение идёт рядом с запросом, а не внутри него, поэтому для изображения {"{{"}selection{"}}"} пустой.
settings-action-prompt = Промпт
settings-action-prompt-example =
    Объясни текст ниже. Текст на языке {"{{"}selection_language{"}}"}; отвечай на {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> — то, что вы выделили. <code>{"{{"}ui_language{"}}"}</code> — язык, на котором вы читаете, <code>{"{{"}selection_language{"}}"}</code> — язык самого текста. Всё остальное в двойных фигурных скобках — параметр.
settings-parameters = Параметры
settings-declare-parameter = Добавить параметр
settings-remove-parameter = Удалить
settings-no-parameters = Нет — действие запускается сразу.
settings-parameter-id = Имя
settings-parameter-label = Вопрос
settings-parameter-default = По умолчанию
settings-parameter-id-example = target
settings-parameter-label-example = На какой язык?
settings-save-action = Сохранить действие
settings-cancel = Отменить
settings-reset-by-saving = Сохранение без изменений вернёт встроенное действие.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Добро пожаловать в Demysto
welcome-step = Шаг { $at } из { $total }
welcome-back = Назад
welcome-continue = Дальше
welcome-finish = Начать работу
welcome-language-title = Demysto определил ваш язык
welcome-language-detail = Система говорит, что вы читаете на нём. Поменяйте здесь или потом в настройках.
welcome-provider-title = Откуда берутся ответы
welcome-provider-detail = Demysto спрашивает модель через вашу учётную запись. Выберите сервис, вставьте ключ и запросите список моделей.
welcome-provider-model = Модель по умолчанию
welcome-provider-verify-first = Проверьте ключ, чтобы продолжить: пусть неверный найдётся сейчас, а не при первом вопросе.
welcome-accessibility-title = Разрешите Demysto читать выделенное
welcome-accessibility-detail = Чтение выделенного macOS выдаёт только с разрешением «Универсальный доступ». Откройте «Конфиденциальность и безопасность» → «Универсальный доступ» и включите Demysto.
welcome-open-accessibility = Открыть настройки Универсального доступа
welcome-accessibility-later = Разрешение можно выдать позже — Demysto спрашивает его при каждом запуске действия. После обновления macOS спросит снова: для неё это уже другое приложение.
welcome-autostart-title = Запускать Demysto при входе в систему
welcome-autostart-detail = Demysto ждёт в трее и отвечает, только пока запущен.
welcome-autostart-choice = Запускать при входе
welcome-done-title = Вот и всё
welcome-done-detail = Выделите где угодно текст и нажмите { $hotkey }. Рядом с курсором откроется список действий; Enter запустит выбранное.
welcome-done-clipboard = Скопируйте текст через Ctrl+C и нажмите { $hotkey }. Рядом с курсором откроется список действий; Enter запустит выбранное.
welcome-done-tray = Дальше Demysto ждёт в трее, и через его меню доступно всё то же, что и по горячей клавише.

## What an update could not do

update-refused = Demysto не смог спросить, есть ли новая версия: { $detail }
update-install-refused = Обновление не удалось установить: { $detail }
update-nothing-found = Устанавливать нечего: сперва проверьте обновления.

## What the login items would not do

autostart-refused = Demysto не смог изменить запуск при входе в систему: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Wayland не позволяет одному приложению печатать в другое, поэтому Demysto не может прочитать выделенное. Скопируйте его через Ctrl+C и нажмите горячую клавишу.
capture-clipboard-unavailable = Буфер обмена недоступен: { $detail }
capture-keystroke-refused = Не удалось отправить сочетание копирования: { $detail }
capture-no-accessibility = macOS не даёт Demysto прочитать выделенное без разрешения «Универсальный доступ». Откройте «Конфиденциальность и безопасность» → «Универсальный доступ» и включите Demysto.
capture-picture-unreadable = Demysto не смог прочитать изображение из буфера обмена. Скопируйте его ещё раз или скопируйте другое.
accessibility-pane-unreachable = Demysto не смог открыть Системные настройки: { $detail }. Разрешение находится в «Конфиденциальность и безопасность» → «Универсальный доступ».
accessibility-only-macos = Разрешение на чтение выделенного спрашивает только macOS.

## What stopped a Run

run-nothing-to-run = Запускать действие не на чем. Выделите или скопируйте текст и нажмите горячую клавишу ещё раз.
run-no-conversation = Нет чата, в котором можно спросить. Нажмите горячую клавишу, чтобы начать.
run-conversation-sealed = Изображение, о котором этот чат, больше не хранится. Скопируйте его и нажмите горячую клавишу, чтобы начать новый чат.
run-no-such-action = Действия «{ $action }» нет — возможно, его удалили. Нажмите горячую клавишу ещё раз.
run-nothing-to-retry = Повторять нечего. Задайте вопрос заново.

# The one warning a Conversation carries, said before the Model is asked so that
# it is on screen while the answer is still being paid for.
run-large-selection =
    В этом выделении { $shown } { $characters ->
        [one] символ
        [few] символа
       *[many] символов
    }, а это больше { $limit }, на которых стоит { $setting } в { $path }. Оно отправлено целиком — ничего не обрезано, — так что стоит ровно столько, сколько это стоит.

## What the settings file could not be made into

config-unreadable = { $path } не удалось прочитать: { $detail }
config-unwritable = { $path } не удалось записать: { $detail }
config-not-toml-at-line = { $path } — не действительный TOML, строка { $line }: { $detail }
config-not-toml = { $path } — не действительный TOML: { $detail }
config-newer-version = { $path } объявляет себя версией { $stated }, а этот Demysto понимает версию { $understood }; обновите Demysto или укажите в { $variable } другой каталог
config-uneditable = { $path } не удалось изменить, не потеряв написанное в нём, поэтому ничего не сохранено.
config-no-provider = ни один провайдер не настроен; откройте { $path } и заполните пример, который там лежит
config-in-file = { $reason } в { $path }
config-provider-no-name = настроен провайдер без имени
config-provider-name-has-separator = в имени провайдера «{ $provider }» есть «{ $separator }», а это разделитель между провайдером и моделью
config-two-providers-named = два провайдера называются «{ $provider }», поэтому модель ни одного из них нельзя назвать
config-provider-model-no-name = провайдер «{ $provider }» перечисляет модель без имени
config-provider-model-twice = провайдер «{ $provider }» перечисляет модель «{ $model }» дважды
config-provider-no-base-url = провайдер «{ $provider }» в { $path } не указывает ни base_url, ни пресет, из которого его взять
config-no-key-anywhere = У провайдера «{ $provider }» нет ключа API: задайте для него api_key в { $path } или укажите переменную окружения в api_key_env.
config-no-key-export = У провайдера «{ $provider }» нет ключа API: экспортируйте { $variables } или задайте для него api_key в { $path }.
config-no-such-preset = Нет пресета с именем «{ $preset }».

## Which Model a Run resolves to, when it resolves to none

model-none-configured = Ни одна модель не настроена; добавьте её какому-нибудь провайдеру там же.
model-configured-are = Там настроены такие модели: { $models }.
model-action-binds-nothing = Это действие привязано к модели «{ $model }», и ни один провайдер в { $path } не предлагает модель с таким именем. { $offered }
model-setting-names-nothing = { $setting } в { $path } называет модель «{ $model }», и ни один провайдер там не предлагает модель с таким именем. { $offered }
model-nothing-nominated = В { $path } не назначено { $setting }. { $offered }
model-no-vision-model = В { $path } не назначено { $setting }, и показать изображение некому. { $offered }
model-nomination-none-configured = { $setting } называет модель «{ $model }», а не настроено ни одной модели.
model-nomination-unknown = { $setting } называет модель «{ $model }», и ни один провайдер не предлагает модель с таким именем. Настроены такие модели: { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto не смог открыть соединение: { $detail }
provider-timed-out =
    { $provider } не ответил за { $seconds ->
        [one] { $seconds } секунду
        [few] { $seconds } секунды
       *[many] { $seconds } секунд
    }, и Demysto перестал ждать.
provider-unreachable = До { $provider } не удалось достучаться: { $detail }
provider-went-quiet = { $provider } замолчал на полуслове.
provider-stopped-answering = { $provider } перестал отвечать посреди ответа: { $detail }
provider-closed-early = { $provider } закрыл соединение прежде, чем ответ был закончен.
provider-refused = Провайдер отклонил запрос (HTTP { $status }).
provider-refused-saying = Провайдер отклонил запрос (HTTP { $status }): { $detail }
provider-malformed = Demysto не смог разобрать ответ провайдера ({ $reason }): { $body }
provider-no-answer-in-it = в нём нет ответа

## What an Action could not be made into

action-file-preamble = # Действие, которое запускает Demysto. Правьте его здесь или в настройках Demysto.
action-needs-name = Действию нужно имя, под которым оно будет перечислено.
action-needs-prompt = Действию нужен промпт: то, что оно говорит модели, с {"{{"}selection{"}}"} там, где встаёт выделение.
action-accepts-nothing = Действие, которое ничего не принимает, никогда не будет предложено. Выберите хотя бы один вид выделения.
action-parameter-needs-name = Параметру нужно имя, чтобы его можно было написать в промпте как {"{{"}like_this{"}}"}.
action-parameter-reserved = Параметр нельзя назвать «{ $parameter }»: это имя Demysto заполняет сам.
action-parameter-needs-label = Параметру «{ $parameter }» нужен вопрос, которым его спросят.
action-parameter-twice = Два параметра называются «{ $parameter }», поэтому {"{{"}{ $parameter }{"}}"} в промпте может означать любой из них.
action-binds-nothing-configured = Это действие привязано к модели «{ $model }», а не настроено ни одной модели.
action-binds-unknown-model = Это действие привязано к модели «{ $model }», и ни один провайдер не предлагает модель с таким именем. Настроены такие модели: { $models }.
action-id-not-a-file-name = «{ $action }» не может быть именем файла, поэтому под ним нельзя хранить действие.
action-none-to-remove = Действия «{ $action }» нет — возможно, оно уже удалено. Откройте это окно заново.
action-file-newer-version = { $path } объявляет себя версией { $stated }, а этот Demysto понимает версию { $understood }. Обновите Demysto или уберите файл из этого каталога.
action-file-states-no-field = { $path } не указывает { $field }. Действие, которого у Demysto ещё нет, обязано указать своё имя и свой шаблон.
action-file-unreadable = { $path } не удалось прочитать: { $detail }
action-dir-unreadable = { $path } не удалось прочитать, поэтому действия из него не перечислены: { $detail }
action-file-unwritable = { $path } не удалось записать: { $detail }
action-file-unwritable-shape = { $path } не удалось записать как TOML: { $detail }
action-file-invalid-at-line = { $path } — не действительное действие, строка { $line }: { $detail }
action-file-invalid = { $path } — не действительное действие: { $detail }

## Hotkeys the desktop would not give up

hotkey-palette-fell-back = { $why } Demysto использует { $hotkey } вместо неё.
hotkey-palette-unclaimable = Demysto не смог занять { $hotkey }, которая открывает список действий: { $detail }. Возможно, сочетание уже занято другим приложением. Через меню в трее доступно всё то же самое.
hotkey-palette-not-a-combination = В настройках указано сочетание «{ $hotkey }», которого Demysto не понимает. Сочетание — это модификаторы и одна клавиша, например «Ctrl+Shift+E».
hotkey-palette-types-something = В настройках указано сочетание «{ $hotkey }» — это одна печатающая клавиша. Добавьте Ctrl, Alt или Shift: сама по себе она перестанет печататься во всех программах.
hotkey-palette-refused = Demysto не смог занять указанное в настройках сочетание «{ $hotkey }»: { $detail }. Возможно, оно уже занято другим приложением.
hotkey-action-not-a-combination = { $action } указывает сочетание «{ $hotkey }», которого Demysto не понимает. Сочетание — это модификаторы и одна клавиша, например «Ctrl+Shift+E».
hotkey-action-types-something = { $action } указывает сочетание «{ $hotkey }» — это одна печатающая клавиша. Добавьте Ctrl, Alt или Shift: сама по себе она перестанет печататься во всех программах.
hotkey-action-already-held = { $action } указывает горячую клавишу «{ $hotkey }», и её уже занимает { $holder }. Отвечает на неё только { $holder }; дайте { $action } другую.
hotkey-action-refused = { $action } указывает сочетание «{ $hotkey }», и Demysto не смог его занять: { $detail }. Возможно, оно уже занято другим приложением.
hotkey-palette-holder = список действий

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — открыть список действий
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Окружение пока не приняло горячую клавишу для «{ $wanted }», поэтому на неё ничего не отвечает. Demysto спрашивает снова.
portal-not-taken = Окружение не приняло горячую клавишу для «{ $wanted }», поэтому на неё ничего не отвечает. Клавиатурные сокращения Demysto назначаются в настройках самого окружения.
portal-held-under-nothing = Окружение держит горячую клавишу для «{ $wanted }» без сочетания, поэтому на неё пока ничего не отвечает. Назначьте его в настройках клавиатурных сокращений самого окружения.
portal-stopped-answering = Портал GlobalShortcuts перестал отвечать, а значит, перестали отвечать и горячие клавиши. Перезапуск Demysto запросит их снова; пока этого не случилось, меню в трее ведёт туда же, куда и горячая клавиша.
portal-asking-again = Похоже, окружение ещё запускается: оно принимает запрос на горячую клавишу и ни к чему её не привязывает. Demysto продолжит спрашивать несколько минут.
portal-taken-in-the-end = Окружение выдало горячие клавиши со второй попытки.
portal-asked-enough =
    Demysto спрашивал у окружения свои горячие клавиши { $asked ->
        [one] { $asked } раз
        [few] { $asked } раза
       *[many] { $asked } раз
    } в течение нескольких минут, и оно приняло не все. Больше он не спросит, пока Demysto не перезапустят: клавиатурные сокращения Demysto назначаются в настройках самого окружения, а меню в трее ведёт туда же, куда и горячая клавиша.
portal-refused = Окружение не отдало Demysto горячие клавиши, о которых он попросил: { $detail }. Пока этого не произошло, на них ничего не отвечает — назначаются они в настройках клавиатурных сокращений, а меню в трее ведёт туда же, куда и горячая клавиша.
portal-unreachable = Demysto не смог достучаться до портала GlobalShortcuts, поэтому горячие клавиши не отвечают: { $detail }. Портал приходит с xdg-desktop-portal, в KDE и в GNOME начиная с 48-й версии. Через меню в трее доступно всё то же самое.

## The log folder

folder-uncreatable = { $path } не удалось создать: { $detail }
folder-no-file-manager = Demysto не смог открыть файловый менеджер: { $detail }. Папка — { $path }.

## The settings file a fresh installation is met by
#
# Prose the user reads in their own editor rather than in a window, and
# translated for the same reason the windows are: it is the first thing a new
# installation says, and it says it in a file.

settings-file-preamble =
    # Настройки Demysto.
    #
    # Читаются при запуске Demysto и заново каждый раз, когда их пишет окно настроек, —
    # так что после правки этого файла вручную перезапустите Demysto.
    #
    # Раскомментируйте пример ниже и заполните его.
    #
    # `preset` называет сервис, соглашения которого Demysto знает: он подставляет
    # `base_url` и говорит, какую переменную окружения документация самого сервиса
    # советует экспортировать. Указывайте `base_url` сами для сервиса, у которого нет
    # пресета, или чтобы переопределить подставленное пресетом — например, для
    # локального сервера на своём порту.
    #
    # Пресеты такие:
    #
    { $presets }
    #
    # Пресет, помеченный «без ключа», — это сервер на этой машине, у которого ключей
    # нет вовсе: провайдеру на таком пресете ключ не нужен, и ключ не отправляется.
    # Всем остальным пресетам ключ нужен.
    #
    # Ключ ищется в переменной, названной в `api_key_env`, затем в собственной
    # переменной пресета, затем в `api_key` здесь. Если не писать `api_key`, а
    # экспортировать переменную, секрет останется вне этого файла.
    #
    # `models` перечисляет модели провайдера, которыми вы хотите пользоваться.
    # `vision` говорит, принимает ли модель изображения, и указывается явно, а не
    # угадывается по идентификатору, потому что имя — это не возможность.
    #
    # Модель называется "<провайдер>/<модель>" везде, где её назначают или к ней
    # привязываются. `default_model` — то, к чему сводится действие, не привязанное
    # к своей модели, а `default_vision_model` — то, к чему оно сводится для картинки.
    #
    # `palette_hotkey` — сочетание клавиш, открывающее список действий. Не пишите
    # его, чтобы взять то, с которым Demysto поставляется. Оно записывается как
    # модификаторы и одна клавиша — "Ctrl+Alt+Space". Клавиша, которая ничего не
    # печатает, — громкости, воспроизведения или F13 и выше — может стоять сама по
    # себе. Окно настроек запишет сочетание за вас, если нажать его проще, чем
    # выписать.
    #
    # `language` — язык, на котором говорит Demysto: "en", "de", "es", "fr" или
    # "ru". Не пишите его, и Demysto пойдёт за операционной системой, откатываясь
    # к английскому. { $languageEnv } перекрывает и то и другое.
    #
    # `large_selection` — сколько символов может быть в выделении, прежде чем Demysto
    # скажет об этом в беседе. Ничего никогда не обрезается и ничего не отклоняется:
    # это нужно затем, чтобы случайное «выделить всё» не было оплачено молча. Не
    # пишите его, чтобы взять { $largeSelection }, или поставьте 0, чтобы не получать
    # предупреждений.
    #
    # `welcomed` — собственная пометка Demysto о том, что первый запуск уже
    # пройден. Уберите строку, чтобы пройти его снова при следующем старте.
settings-file-preset = #   { $preset }
settings-file-preset-keyless = #   { $preset } (без ключа)
