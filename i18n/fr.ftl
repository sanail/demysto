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

## The Palette

palette-reading-selection = Lecture de ce que vous avez sélectionné…
palette-reading-clipboard = Lecture du presse-papiers…
palette-origin-selection = Sélection
palette-origin-clipboard = Depuis le presse-papiers
palette-nothing-captured = Rien n'est sélectionné et le presse-papiers est vide. Sélectionnez du texte et appuyez de nouveau sur le raccourci.
palette-filter = Filtrer les actions…
palette-no-action-matches = Aucune action ne porte ce nom.
palette-open-accessibility = Ouvrir les réglages d'Accessibilité
palette-keys-collecting = Entrée pour exécuter · Échap pour revenir
palette-keys-choosing = ↑↓ pour choisir · Entrée pour exécuter · Échap pour fermer
palette-keys-closing = Échap pour fermer

## The Conversation window

result-conversations = Conversations
result-conversation-unnamed = Conversation
result-nothing-asked-yet = Rien n'a encore été demandé.
result-quotation-label = Le texte dont cette conversation parle
result-show-more = Afficher plus
result-show-less = Afficher moins
result-asking = Interrogation du modèle…
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
settings-unreadable-file = Les réglages n'écrasent pas un fichier qu'ils n'ont pas pu lire : rien ne peut donc être modifié ici tant que ce fichier n'est pas réparé. Ouvrez-le, corrigez ce qu'il dit, puis rouvrez cette fenêtre.

### Providers

settings-providers = Fournisseurs
settings-add-provider = Ajouter un fournisseur
settings-remove-provider = Retirer ce fournisseur
settings-no-providers = Aucun fournisseur n'est encore configuré. Ajoutez-en un pour commencer à poser des questions.
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
settings-key-in-file = Conservée dans le fichier de réglages — saisissez-en une pour la remplacer
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
settings-add-model = Ajouter un modèle
settings-remove-model = Retirer
settings-model-sees-images = Voit les images
settings-model-verify-with = Vérifier avec
settings-no-models = Pas encore de modèle. Récupérez la liste, ou ajoutez-en un à la main.
settings-asking-provider = Interrogation du fournisseur…
settings-provider-offers-nothing = Il n'offre aucun modèle.
settings-provider-answered = { $model } a répondu.

### Defaults

settings-defaults = Valeurs par défaut
settings-default-model = Modèle par défaut — celui qu'utilise une action qui n'a pas le sien
settings-default-vision-model = Modèle de vision par défaut — celui qu'une image utilise à la place
settings-model-none = Aucun
settings-model-does-not-see = { $model } (ne voit pas les images)
settings-large-selection = Avertir au-delà de — combien de caractères une sélection peut compter avant que Demysto le dise
settings-large-selection-default = { $characters } — ce que Demysto propose d'origine
settings-large-selection-detail = Rien n'est jamais coupé et rien n'est jamais refusé : l'avertissement est là pour qu'un « tout sélectionner » accidentel ne soit pas payé en silence. Laissez le champ vide pour le chiffre de Demysto, ou mettez 0 pour ne pas être averti du tout.

### Language

settings-language = Langue
settings-language-field = La langue que parle Demysto
settings-language-follows-system = Suivre le système d'exploitation
settings-language-detail = Enregistrée par le bouton Enregistrer ci-dessous, et parlée dès qu'elle l'est — le menu de la barre d'état comme cette fenêtre.
settings-language-from-environment = { $variable } vaut { $value } : c'est donc la langue que parle Demysto, quoi qu'on choisisse ici.

### Hotkeys

settings-hotkeys = Raccourcis
settings-palette-hotkey = La palette — ce qui l'ouvre par-dessus ce que vous lisez
settings-hotkey-record = Saisir
settings-hotkey-clear = Effacer
settings-hotkey-recording = Appuyez sur une combinaison… Échap pour arrêter
settings-hotkey-default = { $hotkey } — ce que Demysto propose d'origine
settings-hotkey-none = Aucun — cette action s'atteint par la palette
settings-hotkey-rule = Tenez au moins un modificateur, ou appuyez sur une touche qui n'écrit rien par elle-même — F13 et au-delà sont celles que la plupart des claviers savent envoyer.
settings-palette-hotkey-detail = Enregistré par le bouton Enregistrer ci-dessous, et répond dès qu'il l'est.
settings-action-hotkey-detail = Les paramètres ne sont pas demandés par ce chemin — chacun prend ce qu'il propose.
settings-wayland-hotkeys = Wayland ne laisse pas non plus une application réserver un raccourci pour elle. Demysto demande les combinaisons ci-dessous au portail GlobalShortcuts du bureau, et c'est le bureau qui décide à quoi chacune répond — changez-les dans les réglages de raccourcis clavier du bureau lui-même, où elles sont listées sous Demysto.

### Logs

settings-logs = Journaux
settings-logs-detail = Demysto tient un journal local de ce qu'il a fait — quelle action, quel modèle, ce qui a échoué — et jamais de ce que vous regardiez ni de ce qu'un modèle a dit. Rien n'est envoyé nulle part. Joignez ces fichiers à un rapport de bogue.
settings-open-logs = Ouvrir le dossier des journaux

### Updates

settings-updates = Mises à jour
settings-updates-detail = Demysto cherche une nouvelle version au démarrage et propose ce qu'il trouve — rien n'est installé tant que vous ne le dites pas. Chaque mise à jour est signée avec la clé propre à Demysto et vérifiée avec elle avant d'être installée.
settings-version = Ceci est Demysto { $version }.
settings-check-for-update = Chercher une nouvelle version
settings-checking = Recherche…
settings-up-to-date = C'est la version la plus récente qui existe.
settings-update-found = Demysto { $version } est prêt à être installé.
settings-install-update = Installer et redémarrer
settings-installing = Installation…

### Actions

settings-actions = Actions
settings-write-action = Écrire une action
settings-actions-detail = Chaque action est un fichier à elle dans <code>actions</code> : on peut donc la sauvegarder ou l'envoyer à quelqu'un. Les actions intégrées n'y sont pas écrites : en modifier une ne garde que ce que vous avez changé, et la réinitialiser efface cela. Une action s'enregistre pour elle-même, et non par le bouton Enregistrer ci-dessous.
settings-action-changed = Modifiée
settings-action-yours = À vous
settings-action-edit = Modifier
settings-action-reset = Réinitialiser
settings-action-delete = Supprimer
settings-action-name = Nom — ce que la palette liste
settings-action-name-example = Réécrire simplement
settings-action-model = Modèle — laissez la valeur par défaut à moins que cette action n'ait besoin du sien
settings-action-model-default = Ce que disent les valeurs par défaut
settings-action-hotkey = Raccourci — exécute cette action sur ce que vous avez sélectionné, sans passer par la palette
settings-action-prompt = Prompt
settings-action-prompt-example =
    Explique le texte ci-dessous. Le texte est en {"{{"}selection_language{"}}"} ; réponds en {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> est ce que vous avez sélectionné ; <code>{"{{"}ui_language{"}}"}</code> et <code>{"{{"}selection_language{"}}"}</code> sont la langue que vous lisez et celle dans laquelle le texte s'est trouvé être. Tout le reste entre accolades doubles est un paramètre, que la palette demande avant l'exécution — déclarez-le ci-dessous.
settings-parameters = Paramètres
settings-declare-parameter = Déclarer un paramètre
settings-remove-parameter = Retirer
settings-no-parameters = Aucun. Cette action s'exécute dès qu'elle est choisie.
settings-parameter-id-example = target
settings-parameter-label-example = Vers quelle langue ?
settings-parameter-default-example = Ce qu'elle propose
settings-save-action = Enregistrer cette action
settings-cancel = Annuler
settings-reset-by-saving = L'enregistrer sans rien avoir changé remet l'action intégrée.

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
welcome-language-detail = C'est la langue dans laquelle vous lisez, d'après votre système d'exploitation. Changez-la ici si ce n'est pas le cas, et de nouveau dans les réglages quand vous voulez.
welcome-provider-title = D'où viennent les réponses
welcome-provider-detail = Demysto interroge un modèle que vous choisissez, sur votre propre compte. Choisissez le service, collez la clé qu'il vous a donnée, et demandez-lui quels modèles il propose.
welcome-provider-model = Le modèle que Demysto interroge, sauf si une action en dit autrement
welcome-provider-verify-first = La clé est présentée au fournisseur avant la fin de cette étape, pour qu'une clé erronée se voie maintenant plutôt qu'à votre première question.
welcome-accessibility-title = Laisser Demysto lire ce que vous avez sélectionné
welcome-accessibility-detail = Demysto lit une sélection en envoyant la frappe de copie à ce que vous êtes en train de lire, et macOS conditionne cela à l'autorisation d'Accessibilité. Ouvrez Confidentialité et sécurité → Accessibilité et activez Demysto.
welcome-open-accessibility = Ouvrir les réglages d'Accessibilité
welcome-accessibility-later = Demysto le demande à macOS à chaque exécution : l'accorder plus tard fonctionne tout aussi bien. Il est redemandé après une mise à jour, que macOS traite comme une autre application.
welcome-autostart-title = Lancer Demysto à l'ouverture de session
welcome-autostart-detail = Demysto attend le raccourci dans la barre d'état : il ne peut répondre que tant qu'il tourne. Rien n'est inscrit sans que vous le demandiez ici, et les réglages de votre système l'en retirent quand vous voulez.
welcome-autostart-choice = Le lancer à l'ouverture de session
welcome-done-title = C'est tout
welcome-done-detail = Sélectionnez du texte n'importe où et appuyez sur { $hotkey }. La palette s'ouvre à votre curseur avec ce que Demysto peut en faire, et Entrée exécute ce qui est en surbrillance.
welcome-done-clipboard = Copiez du texte avec Ctrl+C, puis appuyez sur { $hotkey }. La palette s'ouvre à votre curseur avec ce que Demysto peut en faire, et Entrée exécute ce qui est en surbrillance.
welcome-done-tray = Demysto attend désormais dans la barre d'état, et son menu mène à la palette, aux actions et aux réglages : le raccourci est le chemin rapide, pas le seul.

## What an update could not do

update-refused = Demysto n'a pas pu demander s'il existe une nouvelle version : { $detail }
update-install-refused = La mise à jour n'a pas pu être installée : { $detail }
update-nothing-found = Il n'y a aucune mise à jour à installer : cherchez d'abord une nouvelle version.

## What the login items would not do

autostart-refused = Demysto n'a pas pu changer son lancement à l'ouverture de session : { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Ceci est une session Wayland, et Wayland ne laisse pas une application écrire dans une autre. Demysto ne peut pas lire ce que vous avez sélectionné : copiez-le vous-même avec Ctrl+C, puis appuyez sur le raccourci, et Demysto lira le presse-papiers.
capture-clipboard-unavailable = Le presse-papiers est indisponible : { $detail }
capture-keystroke-refused = La frappe de copie n'a pas pu être envoyée : { $detail }
capture-no-accessibility = macOS ne laisse pas Demysto lire ce que vous avez sélectionné : Demysto a besoin de l'autorisation d'Accessibilité. Ouvrez Confidentialité et sécurité → Accessibilité et activez Demysto.
accessibility-pane-unreachable = Demysto n'a pas pu ouvrir les Réglages Système : { $detail }. L'autorisation se trouve dans Confidentialité et sécurité → Accessibilité.
accessibility-only-macos = Seul macOS demande une autorisation avant que Demysto puisse lire ce que vous avez sélectionné.

## What stopped a Run

run-nothing-to-run = Il n'y a rien sur quoi exécuter une action : sélectionnez du texte, ou copiez-le, et appuyez de nouveau sur le raccourci.
run-no-conversation = Il n'y a aucune conversation où poser cette question. Appuyez sur le raccourci pour en commencer une.
run-no-such-action = Il n'y a aucune action appelée « { $action } ». Elle a peut-être été retirée depuis que la palette s'est ouverte ; appuyez de nouveau sur le raccourci.
run-nothing-to-retry = Il n'y a aucun tour à reprendre. Reposez la question pour en commencer un nouveau.

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
provider-went-quiet = { $provider } s'est tu au milieu de la réponse, alors Demysto a cessé d'attendre.
provider-stopped-answering = { $provider } a cessé de répondre au milieu de la réponse : { $detail }
provider-closed-early = { $provider } a fermé la connexion avant que la réponse soit terminée.
provider-refused = Le fournisseur a refusé la requête (HTTP { $status }).
provider-refused-saying = Le fournisseur a refusé la requête (HTTP { $status }) : { $detail }
provider-malformed = La réponse du fournisseur n'était pas une que Demysto puisse lire ({ $reason }) : { $body }
provider-no-answer-in-it = elle ne contient aucune réponse

## What an Action could not be made into

action-file-preamble = # Une action que Demysto exécute. Modifiez-la ici, ou dans les réglages de Demysto.
action-needs-name = Une action a besoin d'un nom sous lequel être listée.
action-needs-prompt = Une action a besoin d'un prompt : ce qu'elle dit au modèle, avec {"{{"}selection{"}}"} là où va la sélection.
action-accepts-nothing = Une action qui n'accepte aucune sorte de sélection ne pourrait jamais paraître dans la palette.
action-parameter-needs-name = Un paramètre a besoin d'un nom pour s'écrire {"{{"}like_this{"}}"} dans le prompt.
action-parameter-reserved = Un paramètre ne peut pas s'appeler « { $parameter } » : c'est ainsi qu'un prompt atteint quelque chose que Demysto remplit lui-même, et rien ne viendrait donc jamais le demander.
action-parameter-needs-label = Le paramètre « { $parameter } » a besoin d'un libellé, qui est ce par quoi la palette le demande.
action-parameter-twice = Deux paramètres s'appellent « { $parameter } », donc {"{{"}{ $parameter }{"}}"} dans le prompt pourrait désigner l'un ou l'autre.
action-binds-nothing-configured = Cette action lie le modèle « { $model } », et aucun modèle n'est configuré du tout.
action-binds-unknown-model = Cette action lie le modèle « { $model } », et aucun fournisseur n'en offre un de ce nom. Les modèles configurés sont : { $models }.
action-id-not-a-file-name = « { $action } » ne peut pas être le nom d'un fichier, aucune action ne peut donc être gardée sous ce nom.
action-none-to-remove = Il n'y a aucune action appelée « { $action } » à retirer. Elle a peut-être déjà été supprimée ; rouvrez cette fenêtre.
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
hotkey-palette-unclaimable = Demysto n'a pas pu réserver { $hotkey }, le raccourci qui ouvre la palette : { $detail }. Une autre application le tient peut-être déjà. Le menu de la barre d'état atteint tout ce que le raccourci atteint.
hotkey-palette-not-a-combination = Les réglages indiquent le raccourci « { $hotkey } » pour la palette, or ce n'est pas une combinaison que Demysto comprenne.
hotkey-palette-types-something = Les réglages indiquent le raccourci « { $hotkey } » pour la palette, or c'est une seule touche qui écrit quelque chose. Un raccourci est réservé partout : une touche seule doit donc être une qui n'atteint rien de ce dans quoi vous écriviez.
hotkey-palette-refused = Les réglages indiquent le raccourci « { $hotkey } » pour la palette, et Demysto n'a pas pu le réserver : { $detail }. Une autre application le tient peut-être déjà.
hotkey-action-not-a-combination = { $action } indique le raccourci « { $hotkey } », or ce n'est pas une combinaison que Demysto comprenne. Un raccourci, ce sont ses modificateurs puis une touche, écrits comme « Ctrl+Shift+E ».
hotkey-action-types-something = { $action } indique le raccourci « { $hotkey } », or c'est une seule touche qui écrit quelque chose. Un raccourci est réservé partout : une touche seule doit donc être une qui n'atteint rien de ce dans quoi vous écriviez — Pause, ScrollLock, PrintScreen, F13 et au-delà, ou une touche de volume ou de lecture. Tout le reste a besoin d'un modificateur.
hotkey-action-already-held = { $action } indique le raccourci « { $hotkey } », et { $holder } le tient déjà. Seul { $holder } y répond ; donnez-en un autre à { $action }.
hotkey-action-refused = { $action } indique le raccourci « { $hotkey } », et Demysto n'a pas pu le réserver : { $detail }. Une autre application le tient peut-être déjà.
hotkey-palette-holder = la palette

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — ouvrir la palette
portal-action-description = Demysto — { $action }
portal-not-taken-yet = Le bureau n'a pas encore pris de raccourci pour { $wanted }, rien n'y répond donc encore. Demysto le redemande.
portal-not-taken = Le bureau n'a pas pris de raccourci pour { $wanted }, rien n'y répond donc. Les raccourcis de Demysto s'attribuent dans les réglages de raccourcis clavier du bureau.
portal-held-under-nothing = Le bureau garde un raccourci pour { $wanted } sous aucune combinaison, rien n'y répond donc encore. Donnez-lui-en une dans les réglages de raccourcis clavier du bureau lui-même.
portal-stopped-answering = Le portail GlobalShortcuts du bureau a cessé de répondre, plus aucun raccourci ne répond donc non plus. Redémarrer Demysto les redemande ; en attendant, le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-asking-again = C'est à cela que ressemble un bureau encore en train de démarrer : il prend la demande d'un raccourci et ne la donne à rien, ou n'y répond jamais. Demysto continue de demander quelques minutes, puis laisse le bureau tranquille.
portal-taken-in-the-end = Le bureau a pris les raccourcis que Demysto demandait quand on les lui a redemandés.
portal-asked-enough =
    Demysto a demandé ses raccourcis au bureau { $asked ->
        [one] une fois
       *[other] { $asked } fois
    } sur plusieurs minutes, et il ne les a pas tous pris. Il ne redemandera plus tant que Demysto n'aura pas été redémarré — les raccourcis de Demysto s'attribuent dans les réglages de raccourcis clavier du bureau lui-même, et le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-refused = Le bureau n'a pas donné à Demysto les raccourcis qu'il demandait : { $detail }. Rien n'y répond tant qu'il ne le fait pas — ils s'attribuent dans ses réglages de raccourcis clavier, et le menu de la barre d'état atteint tout ce que le raccourci atteint.
portal-unreachable = Ceci est une session Wayland, où Demysto doit demander un raccourci au portail GlobalShortcuts du bureau — et il n'a pu en joindre aucun : { $detail }. Aucun raccourci ne répond. Le portail arrive avec xdg-desktop-portal, sur KDE et sur GNOME à partir de la version 48. Le menu de la barre d'état atteint tout ce que le raccourci atteint.

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
    # `palette_hotkey` est la combinaison de touches qui ouvre la palette. Laissez-la
    # de côté pour celle que Demysto propose d'origine. Elle s'écrit comme ses
    # modificateurs puis une touche — "Ctrl+Alt+Space" — et une touche qui n'écrit
    # rien, telle que F13, peut tenir seule. La fenêtre des réglages en enregistre une
    # pour vous si vous préférez l'appuyer que l'épeler.
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
