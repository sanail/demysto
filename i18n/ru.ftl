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

## The Palette

palette-reading-selection = Читаю выделенное…
palette-reading-clipboard = Читаю буфер обмена…
palette-origin-selection = Выделение
palette-origin-clipboard = Из буфера обмена
palette-nothing-captured = Ничего не выделено, и буфер обмена пуст. Выделите текст и нажмите горячую клавишу ещё раз.
palette-filter = Фильтр действий…
palette-no-action-matches = Ни одно действие так не называется.
palette-back = Назад
palette-next = Далее
palette-run = Запустить
palette-open-accessibility = Открыть настройки Универсального доступа
palette-keys-collecting = Enter — запустить · Esc — назад
palette-keys-choosing = ↑↓ — выбрать · Enter — запустить · Esc — закрыть
palette-keys-closing = Esc — закрыть

## The Conversation window

result-conversations = Беседы
result-conversation-unnamed = Беседа
result-nothing-asked-yet = Пока ничего не спрошено.
result-quotation-label = Текст, о котором эта беседа
result-show-more = Показать целиком
result-show-less = Свернуть
result-asking = Спрашиваю модель…
result-reasoning = Модель размышляет…
result-copy-answer = Скопировать ответ
result-copied = Скопировано
result-stopped = Остановлено
result-continue = Продолжить
result-try-again = Попробовать снова
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
settings-unreadable-file = Demysto не станет переписывать файл, который не смог прочитать, поэтому изменить здесь ничего нельзя, пока файл не исправлен. Откройте его, поправьте написанное и откройте это окно снова.

### Providers

settings-providers = Провайдеры
settings-add-provider = Добавить провайдера
settings-remove-provider = Удалить этого провайдера
settings-no-providers = Ни один провайдер не настроен. Добавьте одного, чтобы начать спрашивать.
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
settings-key-in-file = Хранится в файле настроек — введите новый, чтобы заменить
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
settings-add-model = Добавить модель
settings-remove-model = Удалить
settings-model-sees-images = Видит изображения
settings-model-verify-with = Проверять на ней
settings-no-models = Моделей пока нет. Запросите список или добавьте модель вручную.
settings-asking-provider = Спрашиваю провайдера…
settings-provider-offers-nothing = Он не предлагает ни одной модели.
settings-provider-answered = { $model } ответила.

### Defaults

settings-defaults = Умолчания
settings-default-model = Модель по умолчанию — та, что берёт действие без своей собственной
settings-default-vision-model = Модель для изображений — та, что берётся вместо неё для картинки
settings-model-none = Нет
settings-model-does-not-see = { $model } (не видит изображений)
settings-large-selection = Предупреждать от — сколько символов может быть в выделении, прежде чем Demysto скажет об этом
settings-large-selection-default = { $characters } — то, с чем Demysto поставляется
settings-large-selection-detail = Ничего никогда не обрезается и ничего не отклоняется: предупреждение нужно затем, чтобы случайное «выделить всё» не было оплачено молча. Оставьте поле пустым, чтобы взять число Demysto, или поставьте 0, чтобы не получать предупреждений.

### Language

settings-language = Язык
settings-language-field = Язык, на котором говорит Demysto
settings-language-follows-system = Следовать за операционной системой
settings-language-detail = Сохраняется кнопкой «Сохранить» ниже и начинает звучать сразу же — и в меню в трее, и в этом окне.
settings-language-from-environment = { $variable } имеет значение { $value }, поэтому Demysto говорит на этом языке, что бы ни было выбрано здесь.

### Hotkeys

settings-hotkeys = Горячие клавиши
settings-palette-hotkey = Палитра — то, что открывает её поверх того, что вы читаете
settings-hotkey-record = Записать
settings-hotkey-clear = Очистить
settings-hotkey-recording = Нажмите сочетание… Esc — прекратить
settings-hotkey-default = { $hotkey } — то, с чем Demysto поставляется
settings-hotkey-none = Нет — это действие вызывается через палитру
settings-hotkey-rule = Удерживайте хотя бы один модификатор или нажмите клавишу, которая сама по себе ничего не печатает, — F13 и выше есть на большинстве клавиатур.
settings-palette-hotkey-detail = Сохраняется кнопкой «Сохранить» ниже и начинает отвечать сразу же.
settings-action-hotkey-detail = Параметры на этом пути не спрашиваются — каждый берёт то, что предлагает.
settings-wayland-hotkeys = Wayland к тому же не позволяет приложению закрепить горячую клавишу за собой. Demysto просит сочетания ниже у портала GlobalShortcuts, и окружение само решает, чему какое отвечает, — меняйте их в настройках клавиатурных сокращений самого окружения, где они перечислены под именем Demysto.

### Logs

settings-logs = Журналы
settings-logs-detail = Demysto ведёт локальный журнал того, что он делал, — какое действие, какая модель, что пошло не так — и никогда того, что вы читали или что сказала модель. Никуда ничего не отправляется. Прикладывайте эти файлы к сообщению об ошибке.
settings-open-logs = Открыть папку с журналами

### Updates

settings-updates = Обновления
settings-updates-detail = Demysto ищет новую версию при запуске и предлагает то, что нашёл, — ничего не устанавливается, пока вы не скажете. Каждое обновление подписано собственным ключом Demysto и сверяется с ним, прежде чем встать на место.
settings-version = Это Demysto { $version }.
settings-check-for-update = Проверить, нет ли новой версии
settings-checking = Проверяю…
settings-up-to-date = Это самая новая версия, какая есть.
settings-update-found = Demysto { $version } готов к установке.
settings-install-update = Установить и перезапустить
settings-installing = Устанавливаю…

### Actions

settings-actions = Действия
settings-write-action = Написать действие
settings-actions-detail = Каждое действие — отдельный файл в <code>actions</code>, так что его можно сохранить в резервной копии или отправить кому-нибудь. Встроенные действия туда не пишутся: при изменении встроенного сохраняется только то, что вы изменили, а сброс это удаляет. Действие сохраняется само по себе, а не кнопкой «Сохранить» ниже.
settings-action-changed = Изменено
settings-action-yours = Ваше
settings-action-edit = Изменить
settings-action-reset = Сбросить
settings-action-delete = Удалить
settings-action-name = Имя — то, что перечисляет палитра
settings-action-name-example = Переписать просто
settings-action-model = Модель — оставьте по умолчанию, если этому действию не нужна своя
settings-action-model-default = Как скажут умолчания
settings-action-hotkey = Горячая клавиша — запускает это действие на выделенном, минуя палитру
settings-action-prompt = Промпт
settings-action-prompt-example =
    Объясни текст ниже. Текст на языке {"{{"}selection_language{"}}"}; отвечай на {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> — то, что вы выделили; <code>{"{{"}ui_language{"}}"}</code> и <code>{"{{"}selection_language{"}}"}</code> — язык, на котором вы читаете, и тот, на котором оказался текст. Всё остальное в двойных фигурных скобках — параметр, который палитра спросит перед запуском; объявите его ниже.
settings-parameters = Параметры
settings-declare-parameter = Объявить параметр
settings-remove-parameter = Удалить
settings-no-parameters = Нет. Это действие запускается сразу, как только выбрано.
settings-parameter-id-example = target
settings-parameter-label-example = На какой язык?
settings-parameter-default-example = Что предлагается
settings-save-action = Сохранить это действие
settings-cancel = Отменить
settings-reset-by-saving = Сохранение без изменений возвращает встроенное действие.

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
welcome-language-detail = На этом языке вы читаете — так говорит операционная система. Если это не так, поменяйте здесь, а потом хоть когда в настройках.
welcome-provider-title = Откуда берутся ответы
welcome-provider-detail = Demysto спрашивает модель, которую вы выбрали сами, через вашу собственную учётную запись. Выберите сервис, вставьте выданный им ключ и спросите, какие модели он предлагает.
welcome-provider-model = Модель, которую Demysto спрашивает, если действие не говорит иного
welcome-provider-verify-first = Ключ предъявляется провайдеру до конца этого шага, чтобы неверный обнаружился сейчас, а не при первом вашем вопросе.
welcome-accessibility-title = Разрешите Demysto читать выделенное
welcome-accessibility-detail = Demysto читает выделение, отправляя сочетание копирования тому, что вы читаете, а macOS выдаёт это только с разрешением «Универсальный доступ». Откройте «Конфиденциальность и безопасность» → «Универсальный доступ» и включите Demysto.
welcome-open-accessibility = Открыть настройки Универсального доступа
welcome-accessibility-later = Demysto спрашивает об этом macOS при каждом запуске, так что выдать разрешение позже — то же самое. После обновления его попросят снова: для macOS это уже другое приложение.
welcome-autostart-title = Запускать Demysto при входе в систему
welcome-autostart-detail = Demysto ждёт горячую клавишу в трее и отвечает только пока запущен. Без вашего согласия здесь ничего не прописывается, а настройки самой системы уберут запись обратно.
welcome-autostart-choice = Запускать при входе
welcome-done-title = Вот и всё
welcome-done-detail = Выделите где угодно текст и нажмите { $hotkey }. Рядом с курсором откроется палитра с тем, что Demysto может с ним сделать, а Enter запустит выделенное.
welcome-done-clipboard = Скопируйте текст через Ctrl+C и нажмите { $hotkey }. Рядом с курсором откроется палитра с тем, что Demysto может с ним сделать, а Enter запустит выделенное.
welcome-done-tray = Дальше Demysto ждёт в трее, и из его меню доступны палитра, действия и настройки: горячая клавиша — быстрый путь, но не единственный.

## What an update could not do

update-refused = Demysto не смог спросить, есть ли новая версия: { $detail }
update-install-refused = Обновление не удалось установить: { $detail }
update-nothing-found = Устанавливать нечего: сперва проверьте, нет ли новой версии.

## What the login items would not do

autostart-refused = Demysto не смог изменить запуск при входе в систему: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Это сеанс Wayland, а Wayland не позволяет одному приложению печатать в другое. Demysto не может прочитать выделенное вами: скопируйте его сами через Ctrl+C, затем нажмите горячую клавишу — и Demysto прочитает буфер обмена.
capture-clipboard-unavailable = Буфер обмена недоступен: { $detail }
capture-keystroke-refused = Не удалось отправить сочетание копирования: { $detail }
capture-no-accessibility = macOS не даёт Demysto прочитать выделенное: Demysto нужно разрешение «Универсальный доступ». Откройте «Конфиденциальность и безопасность» → «Универсальный доступ» и включите Demysto.
accessibility-pane-unreachable = Demysto не смог открыть Системные настройки: { $detail }. Разрешение находится в «Конфиденциальность и безопасность» → «Универсальный доступ».
accessibility-only-macos = Только macOS спрашивает разрешение, прежде чем Demysto сможет прочитать выделенное.

## What stopped a Run

run-nothing-to-run = Не на чем запускать действие: выделите текст или скопируйте его и нажмите горячую клавишу ещё раз.
run-no-conversation = Нет беседы, в которой можно это спросить. Нажмите горячую клавишу, чтобы начать новую.
run-no-such-action = Нет действия с именем «{ $action }». Возможно, его удалили после того, как открылась палитра; нажмите горячую клавишу ещё раз.
run-nothing-to-retry = Нет реплики, которую можно повторить. Задайте вопрос заново, чтобы начать новую.

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
provider-went-quiet = { $provider } замолчал посреди ответа, и Demysto перестал ждать.
provider-stopped-answering = { $provider } перестал отвечать посреди ответа: { $detail }
provider-closed-early = { $provider } закрыл соединение прежде, чем ответ был закончен.
provider-refused = Провайдер отклонил запрос (HTTP { $status }).
provider-refused-saying = Провайдер отклонил запрос (HTTP { $status }): { $detail }
provider-malformed = Ответ провайдера оказался не таким, какой Demysto может прочитать ({ $reason }): { $body }
provider-no-answer-in-it = в нём нет ответа

## What an Action could not be made into

action-file-preamble = # Действие, которое запускает Demysto. Правьте его здесь или в настройках Demysto.
action-needs-name = Действию нужно имя, под которым оно будет перечислено.
action-needs-prompt = Действию нужен промпт: то, что оно говорит модели, с {"{{"}selection{"}}"} там, где встаёт выделение.
action-accepts-nothing = Действие, которое не принимает ни одного вида выделения, никогда не появится в палитре.
action-parameter-needs-name = Параметру нужно имя, чтобы его можно было написать в промпте как {"{{"}like_this{"}}"}.
action-parameter-reserved = Параметр не может называться «{ $parameter }»: так промпт обращается к тому, что подставляет сам Demysto, и такой параметр никто никогда не спросит.
action-parameter-needs-label = Параметру «{ $parameter }» нужна подпись — это то, чем палитра его спрашивает.
action-parameter-twice = Два параметра называются «{ $parameter }», поэтому {"{{"}{ $parameter }{"}}"} в промпте может означать любой из них.
action-binds-nothing-configured = Это действие привязано к модели «{ $model }», а не настроено ни одной модели.
action-binds-unknown-model = Это действие привязано к модели «{ $model }», и ни один провайдер не предлагает модель с таким именем. Настроены такие модели: { $models }.
action-id-not-a-file-name = «{ $action }» не может быть именем файла, поэтому под ним нельзя хранить действие.
action-none-to-remove = Нет действия с именем «{ $action }», которое можно удалить. Возможно, оно уже удалено; откройте это окно заново.
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
hotkey-palette-unclaimable = Demysto не смог закрепить за собой { $hotkey } — горячую клавишу, открывающую палитру: { $detail }. Возможно, её уже занимает другое приложение. Меню в трее ведёт туда же, куда и горячая клавиша.
hotkey-palette-not-a-combination = Настройки указывают для палитры горячую клавишу «{ $hotkey }», а это не то сочетание, которое Demysto понимает.
hotkey-palette-types-something = Настройки указывают для палитры горячую клавишу «{ $hotkey }», а это одна клавиша, которая что-то печатает. Горячая клавиша закрепляется во всей системе, поэтому одиночная клавиша должна быть такой, которая не попадает туда, где вы печатаете.
hotkey-palette-refused = Настройки указывают для палитры горячую клавишу «{ $hotkey }», и Demysto не смог её закрепить: { $detail }. Возможно, её уже занимает другое приложение.
hotkey-action-not-a-combination = { $action } указывает горячую клавишу «{ $hotkey }», а это не то сочетание, которое Demysto понимает. Горячая клавиша — это её модификаторы и одна клавиша, записанные как «Ctrl+Shift+E».
hotkey-action-types-something = { $action } указывает горячую клавишу «{ $hotkey }», а это одна клавиша, которая что-то печатает. Горячая клавиша закрепляется во всей системе, поэтому одиночная клавиша должна быть такой, которая не попадает туда, где вы печатаете, — Pause, ScrollLock, PrintScreen, F13 и выше или клавиша громкости либо управления воспроизведением. Всему остальному нужен модификатор.
hotkey-action-already-held = { $action } указывает горячую клавишу «{ $hotkey }», и её уже занимает { $holder }. Отвечает на неё только { $holder }; дайте { $action } другую.
hotkey-action-refused = { $action } указывает горячую клавишу «{ $hotkey }», и Demysto не смог её закрепить: { $detail }. Возможно, её уже занимает другое приложение.
hotkey-palette-holder = палитра

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — открыть палитру
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Окружение пока не приняло горячую клавишу для «{ $wanted }», поэтому на неё ничего не отвечает. Demysto спрашивает снова.
portal-not-taken = Окружение не приняло горячую клавишу для «{ $wanted }», поэтому на неё ничего не отвечает. Клавиатурные сокращения Demysto назначаются в настройках самого окружения.
portal-held-under-nothing = Окружение держит горячую клавишу для «{ $wanted }» без сочетания, поэтому на неё пока ничего не отвечает. Назначьте его в настройках клавиатурных сокращений самого окружения.
portal-stopped-answering = Портал GlobalShortcuts перестал отвечать, а значит, перестали отвечать и горячие клавиши. Перезапуск Demysto запросит их снова; пока этого не случилось, меню в трее ведёт туда же, куда и горячая клавиша.
portal-asking-again = Так выглядит окружение, которое ещё поднимается: оно принимает запрос на горячую клавишу и не даёт ей ничего — или не отвечает вовсе. Demysto продолжает спрашивать несколько минут, а затем оставляет окружение в покое.
portal-taken-in-the-end = Окружение приняло горячие клавиши Demysto, когда его спросили снова.
portal-asked-enough =
    Demysto спрашивал у окружения свои горячие клавиши { $asked ->
        [one] { $asked } раз
        [few] { $asked } раза
       *[many] { $asked } раз
    } в течение нескольких минут, и оно приняло не все. Больше он не спросит, пока Demysto не перезапустят: клавиатурные сокращения Demysto назначаются в настройках самого окружения, а меню в трее ведёт туда же, куда и горячая клавиша.
portal-refused = Окружение не отдало Demysto горячие клавиши, о которых он попросил: { $detail }. Пока этого не произошло, на них ничего не отвечает — назначаются они в настройках клавиатурных сокращений, а меню в трее ведёт туда же, куда и горячая клавиша.
portal-unreachable = Это сеанс Wayland, где Demysto приходится просить горячую клавишу у портала GlobalShortcuts, — и достучаться до него не удалось: { $detail }. Ни одна горячая клавиша не отвечает. Портал приходит вместе с xdg-desktop-portal: в KDE и в GNOME начиная с версии 48. Меню в трее ведёт туда же, куда и горячая клавиша.

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
    # `palette_hotkey` — сочетание клавиш, открывающее палитру. Не пишите его, чтобы
    # взять то, с которым Demysto поставляется. Оно записывается как модификаторы и
    # одна клавиша — "Ctrl+Alt+Space", — а клавиша, которая ничего не печатает,
    # например F13, может стоять сама по себе. Окно настроек запишет сочетание за
    # вас, если нажать его проще, чем выписать.
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
