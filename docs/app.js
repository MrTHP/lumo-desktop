"use strict";

const I18N = {
  en: {
    "meta.title": "Lumo Desktop — A local-first desktop client for Proton Lumo",
    "meta.description": "A fast, private and extensible Rust/Tauri desktop client for Proton Lumo via lumo-tamer. Streaming, projects, memory, attachments and tools.",

    "nav.features": "Features",
    "nav.screenshots": "Screenshots",
    "nav.install": "Install",
    "nav.github": "GitHub",
    "nav.download": "Download",

    "hero.eyebrow": "Open source · Rust · Tauri · Linux",
    "hero.title": "Your Lumo.<br><span>On your desktop.</span>",
    "hero.subtitle": "The local-first desktop client for Proton Lumo: streaming, projects, memory, attachments and tools — in a fast, private Rust app.",
    "hero.download": "Download AppImage",
    "hero.github": "View on GitHub",
    "hero.meta1": "Linux x86_64",
    "hero.meta2": "AppImage · .deb",
    "hero.meta3": "MIT licensed",

    "stats.backend": "native backend",
    "stats.telemetry": "telemetry",
    "stats.languages": "interface language",
    "stats.appimage": "AppImage",

    "features.kicker": "Why Lumo Desktop",
    "features.title": "Everything you need to work with Lumo, without leaving your desktop.",
    "features.subtitle": "A fast, discreet native app. Your conversations stay local; Proton Lumo does the thinking.",
    "features.streaming.title": "Streaming & reasoning",
    "features.streaming.text": "Token-by-token replies, extended thinking, Markdown and highlighted code. Stop a generation at any time.",
    "features.local.title": "Local-first history",
    "features.local.text": "Conversations, projects, favorites and search live in local JSON files. No account needed for the client itself.",
    "features.attachments.title": "Attachments & OCR",
    "features.attachments.text": "Drop text, code, PDF, DOCX/ODT or images. Text is extracted locally and images are OCR'd before being sent to Lumo.",
    "features.tools.title": "Tools with permissions",
    "features.tools.text": "Read, write, search and shell tools with a workspace sandbox and confirmation prompts. You stay in control.",
    "features.memory.title": "Memory & personalization",
    "features.memory.text": "Tell Lumo who you are, what you do and how to behave. Durable memories can be extracted automatically and stay local.",
    "features.canvas.title": "Canvas & creations",
    "features.canvas.text": "HTML and SVG creations open in a sandboxed canvas. The library keeps your best artifacts and images one click away.",
    "features.tray.title": "System tray",
    "features.tray.text": "Close to tray, reopen instantly, start a new chat or quit. The app stays out of your way until you need it.",
    "features.lang.title": "French / English",
    "features.lang.text": "The interface follows your system locale, with a manual override in Settings. Dates and labels follow the active language.",
    "features.web.title": "Web search",
    "features.web.text": "Enable lumo-tamer's native web search with one switch. Ask for current weather, news or facts and get live answers.",

    "gallery.kicker": "A closer look",
    "gallery.title": "A real desktop for Lumo.",
    "gallery.subtitle": "Actual screenshots from the app, with a demo profile and sample conversations.",
    "gallery.landing.title": "Canvas & HTML previews",
    "gallery.landing.text": "Open generated pages in a sandboxed canvas.",
    "gallery.attachments.title": "Images & attachments",
    "gallery.attachments.text": "Local OCR and previews.",
    "gallery.projects.title": "Projects",
    "gallery.projects.text": "Instructions and context per project.",
    "gallery.library.title": "Library",
    "gallery.library.text": "Creations and images from your chats.",
    "gallery.memory.title": "Local memory",
    "gallery.memory.text": "Durable facts, under your control.",
    "gallery.tools.title": "Tools at hand",
    "gallery.tools.text": "Web search, creations and visualizations in one menu.",

    "pipeline.kicker": "How it works",
    "pipeline.title": "Three pieces, no middleman cloud.",
    "pipeline.desktop": "Lumo Desktop",
    "pipeline.desktopSub": "Rust · Tauri",
    "pipeline.tamer": "lumo-tamer",
    "pipeline.tamerSub": "local API",
    "pipeline.proton": "Proton Lumo",
    "pipeline.protonSub": "your account",
    "pipeline.note": "The desktop client talks to your local lumo-tamer instance. lumo-tamer handles Proton authentication and the API. No additional server sits in between.",

    "privacy.kicker": "Privacy",
    "privacy.title": "Your data stays with you.",
    "privacy.subtitle": "Lumo Desktop is a local client, not a hosted service.",
    "privacy.local.title": "Local storage",
    "privacy.local.text": "Conversations, projects, memories and settings live in ~/.config/lumo-desktop. No cloud account for the client.",
    "privacy.files.title": "Files processed locally",
    "privacy.files.text": "PDF text, Office documents and image OCR are extracted on your machine. Only the resulting text is sent to Lumo.",
    "privacy.telemetry.title": "No telemetry",
    "privacy.telemetry.text": "No analytics, no tracking, no account required to run the desktop app. The source code is open.",

    "install.kicker": "Download",
    "install.title": "Ready to try it?",
    "install.subtitle": "AppImage or .deb, Linux x86_64. Releases contain both formats.",
    "install.appimage": "AppImage",
    "install.appimageText": "Portable, single file",
    "install.deb": ".deb package",
    "install.debText": "Debian / Ubuntu / derivatives",
    "install.releases": "GitHub Releases",
    "install.releasesText": "All versions and checksums",
    "install.download": "Download →",
    "install.open": "Open →",
    "install.comment": "# Make the AppImage executable, then run it",
    "install.note": "The AppImage uses the system WebKitGTK 4.1. For PDF/OCR attachments, install poppler-utils, tesseract and unzip.",

    "tech.kicker": "Under the hood",
    "tech.title": "Rust, Tauri, and nothing unnecessary.",
    "tech.rust": "SSE streaming, tools, attachments",
    "tech.tauri": "System webview, small binary",
    "tech.webkit": "Linux rendering",
    "tech.storage": "Local, readable history",

    "limits.title": "What lumo-tamer does not expose yet",
    "limits.one": "Native image generation and Custom Lumos are Proton features that lumo-tamer does not transport.",
    "limits.two": "Proton projects and memory are not synced. Lumo Desktop offers local projects, favorites and memory instead.",
    "limits.three": "Web search is a global lumo-tamer setting; enabling it restarts the local service.",

    "faq.kicker": "FAQ",
    "faq.title": "Frequently asked questions",
    "faq.q1": "Is this official?",
    "faq.a1": "No. It is an independent project, not affiliated with Proton. It uses lumo-tamer and Lumo's unofficial API.",
    "faq.q2": "Where is my data stored?",
    "faq.a2": "In ~/.config/lumo-desktop. Conversations, projects, memories and settings stay local.",
    "faq.q3": "Do I need Docker?",
    "faq.a3": "For the one-click installer, yes: Docker, Compose and git. The assistant installs lumo-tamer, opens the Proton login browser and configures the client.",
    "faq.q4": "Can I use my existing lumo-tamer install?",
    "faq.a4": "Yes. Set the lumo-tamer directory and API key in Settings → Connection. The app auto-detects the provider from OpenCode when available.",
    "faq.q5": "Are images sent to Lumo?",
    "faq.a5": "Not the raw images. The client extracts text with local OCR and sends that text. lumo-tamer does not transport native attachments yet.",
    "faq.q6": "What license is it under?",
    "faq.a6": "MIT. The code is open source and contributions are welcome.",

    "cta.title": "Give Lumo a desktop.",
    "cta.text": "Download the AppImage, launch it, and connect your Proton account with the built-in assistant.",
    "cta.download": "Download now",
    "cta.github": "Star on GitHub",

    "footer.readme": "README",
    "footer.disclaimer": "Independent project, not affiliated with Proton. Lumo is a trademark of Proton AG.",
  },

  fr: {
    "meta.title": "Lumo Desktop — Un client desktop local-first pour Proton Lumo",
    "meta.description": "Un client desktop Rust/Tauri rapide, privé et extensible pour Proton Lumo via lumo-tamer. Streaming, projets, mémoire, pièces jointes et outils.",

    "nav.features": "Fonctionnalités",
    "nav.screenshots": "Captures",
    "nav.install": "Installer",
    "nav.github": "GitHub",
    "nav.download": "Télécharger",

    "hero.eyebrow": "Open source · Rust · Tauri · Linux",
    "hero.title": "Ton Lumo.<br><span>Sur ton bureau.</span>",
    "hero.subtitle": "Le client desktop local-first pour Proton Lumo : streaming, projets, mémoire, pièces jointes et outils — dans une app Rust rapide et privée.",
    "hero.download": "Télécharger l'AppImage",
    "hero.github": "Voir sur GitHub",
    "hero.meta1": "Linux x86_64",
    "hero.meta2": "AppImage · .deb",
    "hero.meta3": "Licence MIT",

    "stats.backend": "backend natif",
    "stats.telemetry": "télémétrie",
    "stats.languages": "langues d'interface",
    "stats.appimage": "AppImage",

    "features.kicker": "Pourquoi Lumo Desktop",
    "features.title": "Tout ce qu'il faut pour travailler avec Lumo, sans quitter le bureau.",
    "features.subtitle": "Une app native, rapide et discrète. Tes conversations restent locales ; Proton Lumo fait le reste.",
    "features.streaming.title": "Streaming & raisonnement",
    "features.streaming.text": "Réponses token par token, réflexion étendue, Markdown et code coloré. Arrête une génération à tout moment.",
    "features.local.title": "Historique local-first",
    "features.local.text": "Conversations, projets, favoris et recherche vivent dans des fichiers JSON locaux. Aucun compte requis pour le client.",
    "features.attachments.title": "Pièces jointes & OCR",
    "features.attachments.text": "Dépose du texte, du code, des PDF, DOCX/ODT ou des images. Le texte est extrait localement et les images passent par l'OCR avant d'être envoyées à Lumo.",
    "features.tools.title": "Outils avec permissions",
    "features.tools.text": "Outils de lecture, écriture, recherche et shell avec sandbox de dossier et confirmations. Tu gardes le contrôle.",
    "features.memory.title": "Mémoire & personnalisation",
    "features.memory.text": "Dis à Lumo qui tu es, ce que tu fais et comment te répondre. Les souvenirs durables peuvent être extraits automatiquement et restent locaux.",
    "features.canvas.title": "Canvas & créations",
    "features.canvas.text": "Les créations HTML et SVG s'ouvrent dans un canvas sandboxé. La bibliothèque garde tes meilleurs artefacts et images à portée de clic.",
    "features.tray.title": "Barre système",
    "features.tray.text": "Réduis dans le tray, rouvre instantanément, lance une conversation ou quitte. L'app reste discrète jusqu'à ce que tu en aies besoin.",
    "features.lang.title": "Français / English",
    "features.lang.text": "L'interface suit la locale système, avec un choix manuel dans les paramètres. Les dates et libellés suivent la langue active.",
    "features.web.title": "Recherche web",
    "features.web.text": "Active la recherche web native de lumo-tamer en un clic. Demande la météo, l'actualité ou des faits récents et obtiens des réponses live.",

    "gallery.kicker": "Aperçu",
    "gallery.title": "Un vrai bureau pour Lumo.",
    "gallery.subtitle": "Captures réelles de l'application, avec un profil de démonstration et des conversations d'exemple.",
    "gallery.landing.title": "Canvas & aperçus HTML",
    "gallery.landing.text": "Ouvre les pages générées dans un canvas sandboxé.",
    "gallery.attachments.title": "Images & pièces jointes",
    "gallery.attachments.text": "OCR et aperçus locaux.",
    "gallery.projects.title": "Projets",
    "gallery.projects.text": "Instructions et contexte par projet.",
    "gallery.library.title": "Bibliothèque",
    "gallery.library.text": "Créations et images issues des conversations.",
    "gallery.memory.title": "Mémoire locale",
    "gallery.memory.text": "Des faits durables, sous ton contrôle.",
    "gallery.tools.title": "Outils à portée de main",
    "gallery.tools.text": "Recherche web, créations et visualisations dans un seul menu.",

    "pipeline.kicker": "Comment ça marche",
    "pipeline.title": "Trois briques, zéro cloud intermédiaire.",
    "pipeline.desktop": "Lumo Desktop",
    "pipeline.desktopSub": "Rust · Tauri",
    "pipeline.tamer": "lumo-tamer",
    "pipeline.tamerSub": "API locale",
    "pipeline.proton": "Proton Lumo",
    "pipeline.protonSub": "ton compte",
    "pipeline.note": "Le client desktop parle à ton instance lumo-tamer locale. lumo-tamer gère l'authentification Proton et l'API. Aucun serveur supplémentaire ne s'intercale.",

    "privacy.kicker": "Confidentialité",
    "privacy.title": "Tes données restent chez toi.",
    "privacy.subtitle": "Lumo Desktop est un client local, pas un service hébergé.",
    "privacy.local.title": "Stockage local",
    "privacy.local.text": "Conversations, projets, souvenirs et réglages vivent dans ~/.config/lumo-desktop. Aucun compte cloud pour le client.",
    "privacy.files.title": "Fichiers traités localement",
    "privacy.files.text": "Le texte des PDF, des documents Office et l'OCR des images sont extraits sur ta machine. Seul le texte résultant est envoyé à Lumo.",
    "privacy.telemetry.title": "Pas de télémétrie",
    "privacy.telemetry.text": "Aucune analytique, aucun tracking, aucun compte requis pour lancer l'app. Le code source est ouvert.",

    "install.kicker": "Téléchargement",
    "install.title": "Prêt à essayer ?",
    "install.subtitle": "AppImage ou .deb, Linux x86_64. Les releases contiennent les deux formats.",
    "install.appimage": "AppImage",
    "install.appimageText": "Portable, un seul fichier",
    "install.deb": "Paquet .deb",
    "install.debText": "Debian / Ubuntu / dérivés",
    "install.releases": "GitHub Releases",
    "install.releasesText": "Toutes les versions et sommes de contrôle",
    "install.download": "Télécharger →",
    "install.open": "Ouvrir →",
    "install.comment": "# Rend le fichier exécutable, puis lance-le",
    "install.note": "L'AppImage utilise WebKitGTK 4.1 du système. Pour les pièces jointes PDF/OCR, installe poppler-utils, tesseract et unzip.",

    "tech.kicker": "Sous le capot",
    "tech.title": "Rust, Tauri, et rien d'inutile.",
    "tech.rust": "Streaming SSE, outils, pièces jointes",
    "tech.tauri": "Webview système, binaire léger",
    "tech.webkit": "Rendu Linux",
    "tech.storage": "Historique local et lisible",

    "limits.title": "Ce que lumo-tamer ne permet pas encore",
    "limits.one": "La génération d'images native et Custom Lumos sont des fonctions Proton que lumo-tamer ne transporte pas.",
    "limits.two": "Les projets et la mémoire Proton ne sont pas synchronisés. Lumo Desktop propose des projets, favoris et souvenirs locaux.",
    "limits.three": "La recherche web est un réglage global de lumo-tamer ; l'activer redémarre le service local.",

    "faq.kicker": "FAQ",
    "faq.title": "Questions fréquentes",
    "faq.q1": "Est-ce officiel ?",
    "faq.a1": "Non. C'est un projet indépendant, non affilié à Proton. Il utilise lumo-tamer et l'API non officielle de Lumo.",
    "faq.q2": "Où sont stockées mes données ?",
    "faq.a2": "Dans ~/.config/lumo-desktop. Les conversations, projets, souvenirs et réglages restent locaux.",
    "faq.q3": "Faut-il Docker ?",
    "faq.a3": "Pour l'installateur automatique, oui : Docker, Compose et git. L'assistant installe lumo-tamer, ouvre le navigateur de connexion Proton et configure le client.",
    "faq.q4": "Puis-je utiliser mon installation lumo-tamer existante ?",
    "faq.a4": "Oui. Renseigne le dossier lumo-tamer et la clé API dans Paramètres → Connexion. L'app détecte automatiquement le provider depuis OpenCode si disponible.",
    "faq.q5": "Les images sont-elles envoyées à Lumo ?",
    "faq.a5": "Pas les images brutes. Le client extrait le texte avec l'OCR local et envoie ce texte. lumo-tamer ne transporte pas encore les pièces jointes natives.",
    "faq.q6": "Quelle licence ?",
    "faq.a6": "MIT. Le code est open source et les contributions sont bienvenues.",

    "cta.title": "Donne un bureau à Lumo.",
    "cta.text": "Télécharge l'AppImage, lance-la, et connecte ton compte Proton avec l'assistant intégré.",
    "cta.download": "Télécharger maintenant",
    "cta.github": "Star sur GitHub",

    "footer.readme": "README",
    "footer.disclaimer": "Projet indépendant, non affilié à Proton. Lumo est une marque de Proton AG.",
  },
};

function getLanguage() {
  const requested = new URLSearchParams(location.search).get("lang");
  if (requested === "fr" || requested === "en") return requested;
  const saved = localStorage.getItem("lumo-site-lang");
  if (saved === "fr" || saved === "en") return saved;
  return navigator.language?.toLowerCase().startsWith("fr") ? "fr" : "en";
}

function applyLanguage(lang) {
  const dictionary = I18N[lang] || I18N.en;
  document.documentElement.lang = lang;

  document.querySelectorAll("[data-i18n]").forEach((element) => {
    const value = dictionary[element.dataset.i18n] ?? I18N.en[element.dataset.i18n];
    if (value !== undefined) element.textContent = value;
  });

  document.querySelectorAll("[data-i18n-html]").forEach((element) => {
    const value =
      dictionary[element.dataset.i18nHtml] ?? I18N.en[element.dataset.i18nHtml];
    if (value !== undefined) element.innerHTML = value;
  });

  document.title = dictionary["meta.title"] || I18N.en["meta.title"];
  const description = document.querySelector('meta[name="description"]');
  if (description) {
    description.content =
      dictionary["meta.description"] || I18N.en["meta.description"];
  }

  document.querySelectorAll("[data-lang]").forEach((button) => {
    button.classList.toggle("active", button.dataset.lang === lang);
  });

  localStorage.setItem("lumo-site-lang", lang);
}

document.querySelectorAll("[data-lang]").forEach((button) => {
  button.addEventListener("click", () => applyLanguage(button.dataset.lang));
});

applyLanguage(getLanguage());
