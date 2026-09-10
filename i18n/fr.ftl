# Demysto's interface in French.
#
# Written against `en.ftl`, which is where a message is added first. Every
# identifier there exists here: the suite reads both files and fails the build
# over one this catalogue is missing, and over one it holds that English does
# not.
#
# Settings is "réglages" rather than "paramètres" throughout, because a
# Parameter is a thing of its own here — the value an Action collects before it
# runs — and one word for both would make half these sentences ambiguous.

## The application itself

app-name = Demysto
tray-open = Ouvrir Demysto
tray-actions = Actions
tray-update = Mettre à jour vers { $version }…
tray-settings = Réglages…
tray-quit = Quitter Demysto

# macOS only, and only for the key equivalents: `menu` says why the menu bar
# exists at all and why nothing else is on it.
menu-edit = Édition
menu-quit = Quitter Demysto

# The one thing Demysto raises a notification for: a Run started from an
# Action's own Hotkey that failed with no window on screen to say so.
notification-stopped-part-way = Demysto s'est arrêté en chemin
notification-could-not-answer = Demysto n'a pas pu répondre

## The Actions Demysto comes with
#
# Their names and the Parameters they collect, which is what the Palette shows.
# Their prompt templates stay in `action`, in English, because they are
# addressed to a Model rather than to a person.

action-explain-name = Expliquer
action-translate-name = Traduire
action-translate-target-label = Vers quelle langue ?
action-summarize-name = Résumer
action-describe-image-name = Décrire l'image
action-custom-name = Personnalisée…
action-custom-prompt-label = Que faut-il en faire ?

## The Palette

palette-reading-selection = Lecture de la sélection…
palette-reading-clipboard = Lecture du presse-papiers…
palette-origin-selection = Sélection
palette-origin-clipboard = Depuis le presse-papiers
palette-picture = Une image, { $dimensions }
palette-nothing-captured = Rien n'est sélectionné et le presse-papiers est vide. Sélectionnez du texte et appuyez de nouveau sur le raccourci.
palette-filter = Filtrer les actions…
palette-no-action-matches = Aucun résultat.
palette-back = Retour
palette-next = Suivant
palette-run = Exécuter
palette-open-accessibility = Ouvrir les réglages d'Accessibilité
palette-keys-collecting = Entrée pour exécuter · Échap pour revenir
palette-keys-choosing = ↑↓ pour choisir · Entrée pour exécuter · Échap pour fermer
palette-keys-closing = Échap pour fermer

## The Conversation window

result-conversations = Discussions
result-conversation-unnamed = Discussion
result-nothing-asked-yet = Rien de demandé pour l'instant.
result-quotation-label = Le texte de cette discussion
result-picture-label = L'image de cette discussion
result-show-more = Afficher plus
result-show-less = Afficher moins
result-ask-at-original = Redemander en pleine résolution — { $weight }
result-picture-let-go = Cette image n'est plus conservée.
result-sealed = L'image n'est plus conservée, cette discussion ne peut donc pas continuer. Copiez-la et appuyez sur le raccourci pour en ouvrir une nouvelle.
result-asking = Interrogation du modèle…
result-reasoning = Le modèle réfléchit…
result-copy-answer = Copier la réponse
result-copied = Copié
result-stopped = Arrêté
result-continue = Continuer
result-try-again = Réessayer
result-ask-another-model = Demander à un autre modèle…
result-open-provider-settings = Ouvrir les réglages de { $provider }
result-open-accessibility = Ouvrir les réglages d'Accessibilité
result-follow-up = Poser une autre question…
result-stop = Arrêter
result-ask = Demander
result-keys = Entrée pour demander, Maj+Entrée pour une nouvelle ligne, Échap pour fermer

## A rendered code block, whose copy button is markup rather than a component

code-copy = Copier
code-copied = Copié

## Settings

settings-window-title = Réglages de Demysto
settings-title = Réglages
settings-save = Enregistrer
settings-saving = Enregistrement…
settings-saved = Enregistré.
settings-keys = Échap pour fermer
settings-reading = Lecture des réglages…
settings-unreadable-file = Rien ne peut être modifié tant que le fichier de réglages n'est pas réparé. Corrigez-le, puis rouvrez cette fenêtre.

settings-folder = Dossier des réglages

### The tabs the window is divided into

settings-tab-models = Modèles
settings-tab-actions = Actions
settings-tab-general = Général
settings-tab-about = À propos
settings-tab-unsaved = modifications non enregistrées
settings-tab-update = une mise à jour est prête

### Providers

settings-providers = Fournisseurs
settings-add-provider = Ajouter un fournisseur
settings-remove-provider = Retirer ce fournisseur
settings-provider-edit = Modifier
settings-provider-unnamed = (sans nom)
settings-no-providers = Aucun fournisseur pour l'instant. Ajoutez-en un pour commencer.
settings-provider-name = Nom
settings-provider-name-example = openai
settings-provider-service = Service
settings-provider-no-preset = Sans préréglage
settings-provider-preset-keyless = { $preset } (sans clé)
settings-provider-base-url = URL de base
settings-provider-base-url-from-preset = URL de base — laissez vide pour prendre celle du préréglage
settings-provider-base-url-example = https://api.example.com/v1
settings-provider-key = Clé d'API
settings-provider-key-variable = Ou la variable d'environnement qui la contient
settings-provider-key-variable-example = MY_API_KEY
settings-key-in-file = Conservée dans le fichier de réglages
settings-key-in-environment = Prise dans { $variable }
settings-key-not-needed = Ce service n'a pas de clés
settings-key-missing = Pas encore de clé
settings-key-going = Sera retirée à l'enregistrement
settings-keep-key = Garder la clé dans le fichier
settings-remove-key = Retirer la clé du fichier

### The Models one Provider offers

settings-models = Modèles
settings-fetch-models = Récupérer
settings-verify-key = Vérifier la clé
settings-verify-which-model = Avec quel modèle ?
settings-add-model = Ajouter un modèle
settings-remove-model = Retirer
settings-model-sees-images = Voit les images
settings-no-models = Pas encore de modèle. Récupérez la liste, ou ajoutez-en un à la main.
settings-asking-provider = Interrogation du fournisseur…
settings-provider-offers-nothing = Il n'offre aucun modèle.
settings-provider-answered = { $model } a répondu.

### Defaults

settings-defaults = Valeurs par défaut
settings-default-model = Modèle par défaut — sauf si une action indique le sien
settings-default-vision-model = Modèle par défaut pour les images
settings-model-none = Aucun
settings-model-does-not-see = { $model } (ne voit pas les images)
settings-large-selection = Avertir au-delà de tant de caractères
settings-large-selection-default = { $characters } par défaut
settings-large-selection-detail = Rien n'est jamais coupé : l'avertissement sert seulement à ce qu'un « tout sélectionner » involontaire ne soit pas payé sans qu'on le voie. Laissez vide pour la valeur de Demysto, ou mettez 0 pour ne rien signaler.

### Language

settings-language = Langue
settings-language-field = Langue de l'interface
settings-language-follows-system = Celle du système
settings-language-from-environment = { $variable } vaut { $value } : c'est donc la langue que parle Demysto, quoi qu'on choisisse ici.

### Hotkeys

settings-hotkeys = Raccourcis
settings-palette-hotkey = Ouvre la liste des actions
settings-hotkey-record = Saisir
settings-hotkey-clear = Effacer
settings-hotkey-cancel = Annuler
settings-hotkey-recording = Appuyez sur les touches… Échap pour annuler
settings-hotkey-default = { $hotkey } — par défaut
settings-hotkey-none = Aucun — cette action se lance depuis la liste
settings-palette-hotkey-rule = Maintenez Ctrl, Alt ou Shift et appuyez sur une touche. Une touche qui écrit quelque chose ne peut pas être prise seule : elle cesserait de s'écrire partout.
settings-action-hotkey-rule = Maintenez Ctrl, Alt ou Shift et appuyez sur une touche. Une touche qui écrit quelque chose ne peut pas être prise seule : elle cesserait de s'écrire partout. Les paramètres ne sont pas demandés par cette voie : chacun prend sa valeur par défaut.
settings-wayland-hotkeys = Sous Wayland, c'est le bureau qui distribue les raccourcis, pas Demysto. Changez-les dans ses propres réglages de clavier, où ils figurent sous Demysto.

### Startup

settings-autostart = Démarrage
settings-autostart-choice = Le lancer à l'ouverture de session
settings-autostart-changed = Fait
settings-autostart-detail = Les éléments d'ouverture sont la liste de votre système : cette case prend effet aussitôt et se modifie là-bas aussi.

### Logs

settings-logs = Journaux
settings-logs-detail = Le journal note ce qu'a fait Demysto — quelle action, quel modèle, ce qui a échoué — jamais ce que vous regardiez ni ce qu'un modèle a répondu. Joignez-le à un rapport de bogue.
settings-open-logs = Ouvrir le dossier des journaux

### Updates

settings-updates = Mises à jour
settings-updates-detail = Chaque mise à jour est signée par Demysto et vérifiée avant d'être posée, et rien ne s'installe sans votre accord.
settings-version = Demysto { $version }
settings-check-for-update = Rechercher des mises à jour
settings-checking = Recherche…
settings-up-to-date = C'est la version la plus récente.
settings-update-found = Demysto { $version } est prêt à être installé.
settings-install-update = Installer et redémarrer
settings-installing = Installation…

### Actions

settings-actions = Actions
settings-write-action = Nouvelle action
settings-actions-detail = Chaque action est un fichier à part dans <code>actions</code> : gardez-en une copie, ou envoyez-la à quelqu'un.
settings-action-changed = Modifiée
settings-action-yours = À vous
settings-action-unsaved = non enregistrée
settings-action-edit = Modifier
settings-action-reset = Réinitialiser
settings-action-delete = Supprimer
settings-action-name = Nom
settings-action-name-example = Réécrire simplement
settings-action-model = Modèle
settings-action-model-default = Par défaut
settings-action-hotkey = Raccourci
settings-action-accepts = S'exécute sur
settings-action-accepts-text = Texte
settings-action-accepts-image = Images
settings-action-accepts-detail = Choisissez-en au moins un. L'image voyage à côté du prompt et non dedans : pour une image, {"{{"}selection{"}}"} reste vide.
settings-action-prompt = Prompt
settings-action-prompt-example =
    Explique le texte ci-dessous. Le texte est en {"{{"}selection_language{"}}"} ; réponds en {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> est ce que vous avez sélectionné. <code>{"{{"}ui_language{"}}"}</code> est la langue que vous lisez, <code>{"{{"}selection_language{"}}"}</code> celle du texte lui-même. Tout le reste entre doubles accolades est un paramètre.
settings-parameters = Paramètres
settings-declare-parameter = Ajouter un paramètre
settings-remove-parameter = Retirer
settings-no-parameters = Aucun — cette action part dès qu'on la choisit.
settings-parameter-id = Nom
settings-parameter-label = Question
settings-parameter-default = Par défaut
settings-parameter-id-example = target
settings-parameter-label-example = Vers quelle langue ?
settings-save-action = Enregistrer l'action
settings-cancel = Annuler
settings-reset-by-saving = Enregistrer sans rien changer rétablit l'action intégrée.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Bienvenue dans Demysto
welcome-step = Étape { $at } sur { $total }
welcome-back = Retour
welcome-continue = Continuer
welcome-finish = Commencer à utiliser Demysto
welcome-language-title = Demysto a trouvé votre langue
welcome-language-detail = Votre système dit que vous lisez celle-ci. Changez-la ici, ou plus tard dans les réglages.
welcome-provider-title = D'où viennent les réponses
welcome-provider-detail = Demysto interroge un modèle avec votre propre compte. Choisissez le service, collez sa clé et demandez les modèles qu'il propose.
welcome-provider-model = Modèle par défaut
welcome-provider-verify-first = Vérifiez la clé pour continuer : mieux vaut en découvrir une mauvaise maintenant qu'à votre première question.
welcome-accessibility-title = Laisser Demysto lire ce que vous avez sélectionné
welcome-accessibility-detail = macOS ne laisse lire votre sélection qu'avec l'autorisation Accessibilité. Ouvrez Confidentialité et sécurité → Accessibilité et activez Demysto.
welcome-open-accessibility = Ouvrir les réglages d'Accessibilité
welcome-accessibility-later = Vous pouvez l'accorder plus tard : Demysto la demande à chaque exécution. Après une mise à jour, macOS la redemande, car il y voit une autre application.
welcome-autostart-title = Lancer Demysto à l'ouverture de session
welcome-autostart-detail = Demysto attend dans la zone de notification et ne répond que tant qu'il tourne.
welcome-autostart-choice = Le lancer à l'ouverture de session
welcome-done-title = C'est tout
welcome-done-detail = Sélectionnez du texte n'importe où et appuyez sur { $hotkey }. Une liste d'actions s'ouvre près du curseur ; Entrée lance celle qui est en surbrillance.
welcome-done-clipboard = Copiez du texte avec Ctrl+C et appuyez sur { $hotkey }. Une liste d'actions s'ouvre près du curseur ; Entrée lance celle qui est en surbrillance.
welcome-done-tray = Demysto attend désormais dans la zone de notification, et son menu atteint tout ce qu'atteint le raccourci.

## What an update could not do

update-refused = Demysto n'a pas pu demander s'il existe une nouvelle version : { $detail }
update-install-refused = La mise à jour n'a pas pu être installée : { $detail }
update-nothing-found = Rien à installer : recherchez d'abord des mises à jour.

## What the login items would not do

autostart-refused = Demysto n'a pas pu changer son lancement à l'ouverture de session : { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Wayland ne laisse pas une application écrire dans une autre : Demysto ne peut donc pas lire votre sélection. Copiez-la avec Ctrl+C, puis appuyez sur le raccourci.
capture-clipboard-unavailable = Le presse-papiers est indisponible : { $detail }
capture-keystroke-refused = La frappe de copie n'a pas pu être envoyée : { $detail }
capture-no-accessibility = macOS ne laisse pas Demysto lire votre sélection sans l'autorisation Accessibilité. Ouvrez Confidentialité et sécurité → Accessibilité et activez Demysto.
capture-picture-unreadable = Demysto n'a pas pu lire l'image du presse-papiers. Copiez-la de nouveau, ou copiez-en une autre.
accessibility-pane-unreachable = Demysto n'a pas pu ouvrir les Réglages Système : { $detail }. L'autorisation se trouve dans Confidentialité et sécurité → Accessibilité.
accessibility-only-macos = Seul macOS demande une autorisation avant que Demysto puisse lire votre sélection.

## What stopped a Run

run-nothing-to-run = Il n'y a rien sur quoi exécuter une action. Sélectionnez ou copiez du texte, puis appuyez de nouveau sur le raccourci.
run-no-conversation = Il n'y a aucune discussion où poser cette question. Appuyez sur le raccourci pour en ouvrir une.
run-conversation-sealed = L'image de cette discussion n'est plus conservée. Copiez-la et appuyez sur le raccourci pour en ouvrir une nouvelle.
run-no-such-action = Il n'existe aucune action nommée « { $action } » — elle a peut-être été retirée. Appuyez de nouveau sur le raccourci.
run-nothing-to-retry = Il n'y a rien à réessayer. Reposez la question.

# The one warning a Conversation carries, said before the Model is asked so that
# it is on screen while the answer is still being paid for.
run-large-selection =
    Cette sélection fait { $shown } { $characters ->
        [one] caractère
       *[other] caractères
    }, ce qui dépasse les { $limit } fixés par { $setting } dans { $path }. Elle a été envoyée entière — rien n'a été coupé — et coûte donc ce que cela coûte.

## What the settings file could not be made into

config-unreadable = { $path } n'a pas pu être lu : { $detail }
config-unwritable = { $path } n'a pas pu être écrit : { $detail }
config-not-toml-at-line = { $path } n'est pas du TOML valide à la ligne { $line } : { $detail }
config-not-toml = { $path } n'est pas du TOML valide : { $detail }
config-newer-version = { $path } se dit en version { $stated }, et ce Demysto comprend la version { $understood } ; mettez Demysto à jour, ou faites pointer { $variable } vers un autre dossier
config-uneditable = { $path } n'a pas pu être modifié sans perdre ce qui y est écrit, rien n'a donc été enregistré.
config-no-provider = aucun fournisseur n'est configuré ; ouvrez { $path } et remplissez l'exemple qu'il contient
config-in-file = { $reason } dans { $path }
config-provider-no-name = un fournisseur est configuré sans nom
config-provider-name-has-separator = le fournisseur « { $provider } » a un « { $separator } » dans son nom, or c'est ce qui sépare un fournisseur d'un modèle
config-two-providers-named = deux fournisseurs s'appellent « { $provider } », un modèle de l'un ou de l'autre ne peut donc pas être nommé
config-provider-model-no-name = le fournisseur « { $provider } » liste un modèle sans nom
config-provider-model-twice = le fournisseur « { $provider } » liste deux fois le modèle « { $model } »
config-provider-no-base-url = le fournisseur « { $provider } » dans { $path } n'indique ni base_url ni préréglage d'où en prendre une
config-no-key-anywhere = Le fournisseur « { $provider } » n'a pas de clé d'API : donnez-lui api_key dans { $path }, ou nommez une variable d'environnement dans api_key_env.
config-no-key-export = Le fournisseur « { $provider } » n'a pas de clé d'API : exportez { $variables }, ou donnez-lui api_key dans { $path }.
config-no-such-preset = Il n'y a aucun préréglage appelé « { $preset } ».

## Which Model a Run resolves to, when it resolves to none

model-none-configured = Aucun modèle n'est configuré du tout ; ajoutez-en un à un fournisseur là-bas.
model-configured-are = Les modèles configurés là-bas sont : { $models }.
model-action-binds-nothing = Cette action est liée au modèle « { $model } », et aucun fournisseur dans { $path } n'en offre un de ce nom. { $offered }
model-setting-names-nothing = { $setting } dans { $path } nomme le modèle « { $model } », et aucun fournisseur là-bas n'en offre un de ce nom. { $offered }
model-nothing-nominated = Aucun { $setting } n'est désigné dans { $path }. { $offered }
model-no-vision-model = Aucun { $setting } n'est désigné dans { $path }, il n'y a donc aucun modèle à qui montrer une image. { $offered }
model-nomination-none-configured = { $setting } nomme le modèle « { $model } », et aucun modèle n'est configuré du tout.
model-nomination-unknown = { $setting } nomme le modèle « { $model } », et aucun fournisseur n'en offre un de ce nom. Les modèles configurés sont : { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto n'a pas pu ouvrir de connexion : { $detail }
provider-timed-out =
    { $provider } n'a pas répondu en { $seconds ->
        [one] une seconde
       *[other] { $seconds } secondes
    }, alors Demysto a cessé d'attendre.
provider-unreachable = { $provider } n'a pas pu être joint : { $detail }
provider-went-quiet = { $provider } s'est tu en pleine réponse.
provider-stopped-answering = { $provider } a cessé de répondre au milieu de la réponse : { $detail }
provider-closed-early = { $provider } a fermé la connexion avant que la réponse soit terminée.
provider-refused = Le fournisseur a refusé la requête (HTTP { $status }).
provider-refused-saying = Le fournisseur a refusé la requête (HTTP { $status }) : { $detail }
provider-malformed = Demysto n'a pas pu lire la réponse du fournisseur ({ $reason }) : { $body }
provider-no-answer-in-it = elle ne contient aucune réponse

## What an Action could not be made into

action-file-preamble = # Une action que Demysto exécute. Modifiez-la ici, ou dans les réglages de Demysto.
action-needs-name = Une action a besoin d'un nom sous lequel être listée.
action-needs-prompt = Une action a besoin d'un prompt : ce qu'elle dit au modèle, avec {"{{"}selection{"}}"} là où va la sélection.
action-accepts-nothing = Une action qui n'accepte rien ne pourrait jamais être proposée. Choisissez au moins un type de sélection.
action-parameter-needs-name = Un paramètre a besoin d'un nom pour s'écrire {"{{"}like_this{"}}"} dans le prompt.
action-parameter-reserved = Un paramètre ne peut pas s'appeler « { $parameter } » : celui-là, Demysto le remplit lui-même.
action-parameter-needs-label = Le paramètre « { $parameter } » a besoin d'une question pour le demander.
action-parameter-twice = Deux paramètres s'appellent « { $parameter } », donc {"{{"}{ $parameter }{"}}"} dans le prompt pourrait désigner l'un ou l'autre.
action-binds-nothing-configured = Cette action lie le modèle « { $model } », et aucun modèle n'est configuré du tout.
action-binds-unknown-model = Cette action lie le modèle « { $model } », et aucun fournisseur n'en offre un de ce nom. Les modèles configurés sont : { $models }.
action-id-not-a-file-name = « { $action } » ne peut pas être le nom d'un fichier, aucune action ne peut donc être gardée sous ce nom.
action-none-to-remove = Il n'existe aucune action nommée « { $action } » à retirer — elle est peut-être déjà partie. Rouvrez cette fenêtre.
action-file-newer-version = { $path } se dit en version { $stated }, et ce Demysto comprend la version { $understood }. Mettez Demysto à jour, ou sortez le fichier de ce dossier.
action-file-states-no-field = { $path } n'indique aucun { $field }. Une action que Demysto n'a pas déjà doit indiquer son nom et son gabarit.
action-file-unreadable = { $path } n'a pas pu être lu : { $detail }
action-dir-unreadable = { $path } n'a pas pu être lu, les actions qu'il contient ne sont donc pas listées : { $detail }
action-file-unwritable = { $path } n'a pas pu être écrit : { $detail }
action-file-unwritable-shape = { $path } n'a pas pu être écrit en TOML : { $detail }
action-file-invalid-at-line = { $path } n'est pas une action valide à la ligne { $line } : { $detail }
action-file-invalid = { $path } n'est pas une action valide : { $detail }

## Hotkeys the desktop would not give up

hotkey-palette-fell-back = { $why } Demysto utilise { $hotkey } à la place.
hotkey-palette-unclaimable = Demysto n'a pas pu réserver { $hotkey }, qui ouvre la liste des actions : { $detail }. Une autre application le détient peut-être déjà. Le menu de la zone de notification atteint tout ce qu'atteint le raccourci.
hotkey-palette-not-a-combination = Les réglages indiquent le raccourci « { $hotkey } », que Demysto ne comprend pas. Un raccourci, ce sont ses modificateurs puis une touche, écrit comme « Ctrl+Shift+E ».
hotkey-palette-types-something = Les réglages indiquent le raccourci « { $hotkey } », une touche seule qui écrit. Ajoutez Ctrl, Alt ou Shift : seule, elle cesserait de s'écrire partout.
hotkey-palette-refused = Demysto n'a pas pu réserver le raccourci « { $hotkey } » indiqué dans les réglages : { $detail }. Une autre application le détient peut-être déjà.
hotkey-action-not-a-combination = { $action } indique le raccourci « { $hotkey } », que Demysto ne comprend pas. Un raccourci, ce sont ses modificateurs puis une touche, écrit comme « Ctrl+Shift+E ».
hotkey-action-types-something = { $action } indique le raccourci « { $hotkey } », une touche seule qui écrit. Ajoutez Ctrl, Alt ou Shift : seule, elle cesserait de s'écrire partout.
hotkey-action-already-held = { $action } indique le raccourci « { $hotkey } », et { $holder } le tient déjà. Seul { $holder } y répond ; donnez-en un autre à { $action }.
hotkey-action-refused = { $action } indique le raccourci « { $hotkey } », et Demysto n'a pas pu le réserver : { $detail }. Une autre application le détient peut-être déjà.
hotkey-palette-holder = la liste des actions

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — ouvrir la liste des actions
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Le bureau n'a pas encore pris de raccourci pour { $wanted }, rien n'y répond donc encore. Demysto le redemande.
portal-not-taken = Le bureau n'a pas pris de raccourci pour { $wanted }, rien n'y répond donc. Les raccourcis de Demysto s'attribuent dans les réglages de raccourcis clavier du bureau.
portal-held-under-nothing = Le bureau garde un raccourci pour { $wanted } sous aucune combinaison, rien n'y répond donc encore. Donnez-lui-en une dans les réglages de raccourcis clavier du bureau lui-même.
portal-stopped-answering = Le portail GlobalShortcuts du bureau a cessé de répondre, plus aucun raccourci ne répond donc non plus. Redémarrer Demysto les redemande ; en attendant, le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-asking-again = C'est à cela que ressemble un bureau encore en train de démarrer : il accepte la demande de raccourci et ne l'attribue à rien. Demysto continue de demander quelques minutes.
portal-taken-in-the-end = Le bureau a pris les raccourcis lorsqu'on les lui a redemandés.
portal-asked-enough =
    Demysto a demandé ses raccourcis au bureau { $asked ->
        [one] une fois
       *[other] { $asked } fois
    } sur plusieurs minutes, et il ne les a pas tous pris. Il ne redemandera plus tant que Demysto n'aura pas été redémarré — les raccourcis de Demysto s'attribuent dans les réglages de raccourcis clavier du bureau lui-même, et le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-refused = Le bureau n'a pas donné à Demysto les raccourcis qu'il demandait : { $detail }. Rien n'y répond tant qu'il ne le fait pas — ils s'attribuent dans ses réglages de raccourcis clavier, et le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-unreachable = Demysto n'a pas pu joindre le portail GlobalShortcuts du bureau : aucun raccourci ne répond ({ $detail }). Il arrive avec xdg-desktop-portal, sur KDE et sur GNOME à partir de la version 48. Le menu de la zone de notification atteint tout ce qu'atteint le raccourci.

## The log folder

folder-uncreatable = { $path } n'a pas pu être créé : { $detail }
folder-no-file-manager = Demysto n'a pas pu ouvrir de gestionnaire de fichiers : { $detail }. Le dossier est { $path }.

## The settings file a fresh installation is met by
#
# Prose the user reads in their own editor rather than in a window, and
# translated for the same reason the windows are: it is the first thing a new
# installation says, and it says it in a file.

settings-file-preamble =
    # Réglages de Demysto.
    #
    # Lus au démarrage de Demysto, et de nouveau chaque fois que la fenêtre des
    # réglages les écrit — redémarrez donc Demysto après avoir modifié ce fichier à
    # la main.
    #
    # Décommentez l'exemple ci-dessous et remplissez-le.
    #
    # `preset` nomme un service dont Demysto connaît les conventions : il remplit
    # `base_url`, et il dit quelle variable d'environnement la documentation du
    # service lui-même conseille d'exporter. Indiquez `base_url` vous-même pour un
    # service qui n'a pas de préréglage, ou pour remplacer ce qu'un préréglage
    # remplit — un serveur local écoutant sur un port à vous, par exemple.
    #
    # Les préréglages sont :
    #
    { $presets }
    #
    # Un préréglage marqué « sans clé » est un serveur tournant sur cette machine, qui
    # n'a pas de clés du tout : un fournisseur qui en utilise un n'en a besoin
    # d'aucune, et aucune n'est envoyée. Tous les autres préréglages en veulent une.
    #
    # La clé est cherchée dans la variable que nomme `api_key_env`, puis dans la
    # variable propre au préréglage, puis dans `api_key` ici. Laisser `api_key` de
    # côté et exporter la variable à la place garde le secret hors de ce fichier.
    #
    # `models` liste les modèles d'un fournisseur dont vous voulez vous servir.
    # `vision` dit si l'un d'eux accepte les images, et est indiqué plutôt que deviné
    # d'après l'identifiant, parce qu'un nom n'est pas une capacité.
    #
    # Un modèle se nomme "<fournisseur>/<modèle>" partout où l'on en désigne ou en lie
    # un. `default_model` est ce à quoi se ramène une action qui ne lie aucun modèle à
    # elle, et `default_vision_model` ce à quoi elle se ramène pour une image.
    #
    # `palette_hotkey` est la combinaison de touches qui ouvre la liste des actions.
    # Laissez-la de côté pour celle que Demysto propose d'origine. Elle s'écrit comme
    # ses modificateurs puis une touche — "Ctrl+Alt+Space". Une touche qui n'écrit
    # rien — de volume ou de lecture, ou F13 et au-delà — peut tenir seule. La
    # fenêtre des réglages en enregistre une pour vous si vous préférez l'appuyer
    # que l'épeler.
    #
    # `language` est la langue que parle Demysto : "en", "de", "es", "fr" ou "ru".
    # Laissez-la de côté et Demysto suit le système d'exploitation, en se rabattant
    # sur l'anglais. { $languageEnv } l'emporte sur les deux.
    #
    # `large_selection` est le nombre de caractères qu'une sélection peut compter
    # avant que Demysto le dise dans la conversation. Rien n'est jamais coupé et rien
    # n'est jamais refusé : c'est là pour qu'un « tout sélectionner » accidentel ne
    # soit pas payé en silence. Laissez-le de côté pour { $largeSelection }, ou
    # mettez 0 pour ne pas être averti du tout.
    #
    # `welcomed` est la note que Demysto se laisse à lui-même : le parcours du
    # premier lancement a été fait. Retirez la ligne pour le refaire au prochain
    # démarrage.
settings-file-preset = #   { $preset }
settings-file-preset-keyless = #   { $preset } (sans clé)
