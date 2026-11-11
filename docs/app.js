"use strict";

const I18N = {
  en: {
    "meta.title": "Model Desk — A local-first multi-provider AI workspace",
    "brand.subtitle": "Multi-provider AI workspace",
    "meta.description": "A fast, private and extensible Rust/Tauri desktop AI workspace. Connect Lumo through lumo-tamer, DeepSeek, or any OpenAI-compatible provider. Includes an OpenCode coding agent, a local Bonsai image studio, projects, memory, attachments, tools and a sandboxed canvas.",

    "nav.features": "Features",
    "nav.screenshots": "Screenshots",
    "nav.install": "Install",
    "nav.github": "GitHub",
    "nav.download": "Download",

    "hero.eyebrow": "Open source · Rust · Tauri · Linux",
    "hero.title": "All your models.<br><span>One desk.</span>",
    "hero.subtitle": "A local-first AI workspace for Lumo, DeepSeek and any OpenAI-compatible provider — with an OpenCode coding agent, a Bonsai image studio, projects, attachments, memory and tools.",
    "hero.download": "Download AppImage",
    "hero.github": "View on GitHub",
    "hero.meta1": "Linux x86_64",
    "hero.meta2": "AppImage · .deb",
    "hero.meta3": "MIT licensed",

    "stats.backend": "native backend",
    "stats.telemetry": "telemetry",
    "stats.languages": "interface language",
    "stats.appimage": "AppImage",

    "features.kicker": "Why Model Desk",
    "features.title": "Everything you need to work with your models.",
    "features.subtitle": "Switch providers, keep your history local, generate images with Bonsai, and hand coding tasks to OpenCode without leaving your desk.",
    "features.providers.title": "Multi-provider",
    "features.providers.text": "Lumo through lumo-tamer, DeepSeek, or any OpenAI-compatible endpoint. Switch providers in one click and keep one history.",
    "features.agent.title": "OpenCode agent",
    "features.agent.text": "Run OpenCode inside a project folder, stream tool calls, choose the model and agent, and approve actions yourself or automatically.",
    "features.streaming.title": "Streaming & reasoning",
    "features.streaming.text": "Token-by-token replies, provider-aware reasoning, Markdown and highlighted code. Stop a generation at any time.",
    "features.images.title": "Bonsai image studio",
    "features.images.text": "Generate images locally with Bonsai Image: prompt, model variant, resolution, seed, steps, guidance, batch of 4, stop and a persistent gallery. Unload the GPU model manually or automatically on quit. Connects to your own PrismML backend.",
    "features.local.title": "Local-first history",
    "features.local.text": "Conversations, projects, favorites and search live in local JSON files. Switch providers without losing your history.",
    "features.attachments.title": "Attachments & OCR",
    "features.attachments.text": "Drop text, code, PDF, DOCX/ODT or images. Text is extracted locally and images are OCR'd before being sent to the selected provider.",
    "features.tools.title": "Tools with permissions",
    "features.tools.text": "Read, write, search and shell tools with a workspace sandbox and confirmation prompts. You stay in control.",
    "features.memory.title": "Memory & personalization",
    "features.memory.text": "Tell the assistant who you are, what you do and how to behave. Durable memories can be extracted automatically and stay local.",
    "features.canvas.title": "Canvas & creations",
    "features.canvas.text": "HTML and SVG creations open in a sandboxed canvas. The library keeps your best artifacts and images one click away.",
    "features.tray.title": "System tray",
    "features.tray.text": "Close to tray, reopen instantly, start a new chat or quit. The app stays out of your way until you need it.",
    "features.lang.title": "French / English",
    "features.lang.text": "The interface follows your system locale, with a manual override in Settings. Dates and labels follow the active language.",
    "features.web.title": "Web search",
    "features.web.text": "Enable lumo-tamer's native web search with one switch. Ask for current weather, news or facts and get live answers.",

    "gallery.kicker": "A closer look",
    "gallery.title": "A real desktop for your AI.",
    "gallery.subtitle": "Actual screenshots from the app: providers, chat, the Bonsai image studio and the OpenCode agent.",
    "gallery.chat.title": "Chat & providers",
    "gallery.chat.text": "Switch between Lumo, DeepSeek and custom endpoints.",
    "gallery.images.title": "Bonsai image studio",
    "gallery.images.text": "Prompt, seed, resolution, batch and a local gallery.",
    "gallery.agent.title": "OpenCode agent",
    "gallery.agent.text": "A coding agent with streamed tool calls.",
    "gallery.providers.title": "Provider settings",
    "gallery.providers.text": "Keys, URLs and models in one place.",
    "gallery.agentSettings.title": "Agent settings",
    "gallery.agentSettings.text": "Workspace, model, agent and permissions.",
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
    "pipeline.title": "One desk, many providers.",
    "pipeline.desktop": "Model Desk",
    "pipeline.desktopSub": "Rust · Tauri",
    "pipeline.lumo": "Lumo Tamer",
    "pipeline.lumoSub": "local API → Proton Lumo",
    "pipeline.deepseek": "DeepSeek",
    "pipeline.deepseekSub": "direct API",
    "pipeline.opencode": "OpenCode",
    "pipeline.opencodeSub": "local coding agent",
    "pipeline.bonsai": "Bonsai Image",
    "pipeline.bonsaiSub": "local image server",
    "pipeline.tamer": "lumo-tamer",
    "pipeline.tamerSub": "local API",
    "pipeline.proton": "Proton Lumo",
    "pipeline.protonSub": "your account",
    "pipeline.note": "Model Desk talks to Lumo through your local lumo-tamer, to DeepSeek directly, to OpenCode for coding tasks and to Bonsai for local image generation. There is no Model Desk cloud.",

    "privacy.kicker": "Privacy",
    "privacy.title": "Your data stays with you.",
    "privacy.subtitle": "Model Desk is a local client, not a hosted service.",
    "privacy.local.title": "Local storage",
    "privacy.local.text": "Conversations, projects, memories, settings and generated images live in ~/.config/model-desk. No cloud account for the client.",
    "privacy.files.title": "Files processed locally",
    "privacy.files.text": "PDF text, Office documents and image OCR are extracted on your machine. Only the resulting text is sent to the provider you selected.",
    "privacy.telemetry.title": "No telemetry",
    "privacy.telemetry.text": "No analytics, no tracking, no account required to run the desktop app. The source code is open.",

    "install.kicker": "Download",
    "install.title": "Ready to try it?",
    "install.subtitle": "AppImage or .deb, Linux x86_64. Version 0.4.0 adds the Bonsai image studio, Lumo Tamer, DeepSeek and the OpenCode agent.",
    "install.appimage": "AppImage",
    "install.appimageText": "Portable, single file",
    "install.deb": ".deb package",
    "install.debText": "Debian / Ubuntu / derivatives",
    "install.releases": "GitHub Releases",
    "install.releasesText": "All versions and checksums",
    "install.download": "Download →",
    "install.open": "Open →",
    "install.comment": "# Make the AppImage executable, then run it",
    "install.note": "The AppImage uses the system WebKitGTK 4.1. For Lumo, run lumo-tamer. For the agent, install OpenCode. Bonsai images require a local Bonsai Image Demo server (NVIDIA CUDA or Apple Silicon). For PDF/OCR attachments, install poppler-utils, tesseract and unzip.",

    "tech.kicker": "Under the hood",
    "tech.title": "Rust, Tauri, and nothing unnecessary.",
    "tech.rust": "SSE streaming, tools, attachments",
    "tech.tauri": "System webview, small binary",
    "tech.webkit": "Linux rendering",
    "tech.storage": "Local, readable history",

    "limits.title": "Good to know",
    "limits.one": "Bonsai image generation runs in a separate local server and needs an NVIDIA/CUDA or Apple Silicon GPU; Custom Lumos is not exposed by lumo-tamer.",
    "limits.two": "DeepSeek and custom endpoints have their own context limits, pricing and availability.",
    "limits.three": "Web search is a global lumo-tamer setting; enabling it restarts the local service.",
    "limits.four": "The OpenCode agent runs with the permissions you grant it; auto-approval gives it your user rights.",

    "faq.kicker": "FAQ",
    "faq.title": "Frequently asked questions",
    "faq.q1": "Is this official?",
    "faq.a1": "No. Model Desk is independent and not affiliated with Proton. Lumo is a trademark of Proton AG. Lumo support goes through lumo-tamer's unofficial API.",
    "faq.q2": "Where is my data stored?",
    "faq.a2": "In ~/.config/model-desk. Conversations, projects, memories and settings stay local.",
    "faq.q3": "Do I need Docker?",
    "faq.a3": "Only if you want Lumo through lumo-tamer. The installer uses Docker, Compose and git. DeepSeek and OpenCode do not require Docker.",
    "faq.q4": "Can I use my existing lumo-tamer install?",
    "faq.a4": "Yes. Set the lumo-tamer directory and API key in Settings → Connection. The app auto-detects the provider from OpenCode when available.",
    "faq.q5": "Are images sent to the provider?",
    "faq.a5": "Not the raw images. The client extracts text with local OCR and sends that text for chat. Bonsai image prompts go only to your local Bonsai backend.",
    "faq.q6": "What license is it under?",
    "faq.a6": "MIT. The code is open source and contributions are welcome.",
    "faq.q7": "Which providers are supported?",
    "faq.a7": "Lumo through lumo-tamer, DeepSeek, and any OpenAI-compatible endpoint. OpenCode can run as a coding agent with its own model selection.",
    "faq.q8": "Can I use Model Desk without Lumo?",
    "faq.a8": "Yes. Use DeepSeek or a custom endpoint for chat, and OpenCode for coding tasks. Lumo is optional.",

    "cta.title": "Give your models a desk.",
    "cta.text": "Download the AppImage, pick Lumo, DeepSeek or your own endpoint, then chat or start an OpenCode agent task.",
    "cta.download": "Download now",
    "cta.github": "Star on GitHub",

    "footer.readme": "README",
    "footer.disclaimer": "Independent project, not affiliated with Proton. Lumo is a trademark of Proton AG.",
  },

  fr: {
    "meta.title": "Model Desk — Un espace IA local et multi-fournisseurs",
    "brand.subtitle": "Espace IA multi-fournisseurs",
    "meta.description": "Un espace IA desktop Rust/Tauri rapide, privé et extensible. Connecte Lumo via lumo-tamer, DeepSeek, ou tout fournisseur OpenAI-compatible. Inclut un agent de code OpenCode, un studio d'images Bonsai local, projets, mémoire, pièces jointes, outils et canvas sandboxé.",

    "nav.features": "Fonctionnalités",
    "nav.screenshots": "Captures",
    "nav.install": "Installer",
    "nav.github": "GitHub",
    "nav.download": "Télécharger",

    "hero.eyebrow": "Open source · Rust · Tauri · Linux",
    "hero.title": "Tous tes modèles.<br><span>Un seul bureau.</span>",
    "hero.subtitle": "Un espace IA local-first pour Lumo, DeepSeek et tout fournisseur OpenAI-compatible — avec un agent OpenCode, un studio d'images Bonsai, projets, pièces jointes, mémoire et outils.",
    "hero.download": "Télécharger l'AppImage",
    "hero.github": "Voir sur GitHub",
    "hero.meta1": "Linux x86_64",
    "hero.meta2": "AppImage · .deb",
    "hero.meta3": "Licence MIT",

    "stats.backend": "backend natif",
    "stats.telemetry": "télémétrie",
    "stats.languages": "langues d'interface",
    "stats.appimage": "AppImage",

    "features.kicker": "Pourquoi Model Desk",
    "features.title": "Tout ce qu'il faut pour travailler avec tes modèles.",
    "features.subtitle": "Change de fournisseur, garde ton historique en local, génère des images avec Bonsai, et confie les tâches de code à OpenCode sans quitter ton bureau.",
    "features.providers.title": "Multi-fournisseurs",
    "features.providers.text": "Lumo via lumo-tamer, DeepSeek, ou tout endpoint OpenAI-compatible. Change de fournisseur en un clic et garde un seul historique.",
    "features.agent.title": "Agent OpenCode",
    "features.agent.text": "Lance OpenCode dans un dossier projet, suis les appels d'outils, choisis le modèle et l'agent, et approuve les actions toi-même ou automatiquement.",
    "features.streaming.title": "Streaming & raisonnement",
    "features.streaming.text": "Réponses token par token, raisonnement selon le fournisseur, Markdown et code coloré. Arrête une génération à tout moment.",
    "features.images.title": "Studio d'images Bonsai",
    "features.images.text": "Génère des images en local avec Bonsai Image : prompt, variante de modèle, résolution, seed, étapes, guidage, lot de 4, arrêt et galerie persistante. Décharge le modèle GPU manuellement ou automatiquement à la fermeture. Connecté à ton propre backend PrismML.",
    "features.local.title": "Historique local-first",
    "features.local.text": "Conversations, projets, favoris et recherche vivent dans des fichiers JSON locaux. Change de fournisseur sans perdre ton historique.",
    "features.attachments.title": "Pièces jointes & OCR",
    "features.attachments.text": "Dépose du texte, du code, des PDF, DOCX/ODT ou des images. Le texte est extrait localement et les images passent par l'OCR avant d'être envoyées au fournisseur sélectionné.",
    "features.tools.title": "Outils avec permissions",
    "features.tools.text": "Outils de lecture, écriture, recherche et shell avec sandbox de dossier et confirmations. Tu gardes le contrôle.",
    "features.memory.title": "Mémoire & personnalisation",
    "features.memory.text": "Dis à l'assistant qui tu es, ce que tu fais et comment te répondre. Les souvenirs durables peuvent être extraits automatiquement et restent locaux.",
    "features.canvas.title": "Canvas & créations",
    "features.canvas.text": "Les créations HTML et SVG s'ouvrent dans un canvas sandboxé. La bibliothèque garde tes meilleurs artefacts et images à portée de clic.",
    "features.tray.title": "Barre système",
    "features.tray.text": "Réduis dans le tray, rouvre instantanément, lance une conversation ou quitte. L'app reste discrète jusqu'à ce que tu en aies besoin.",
    "features.lang.title": "Français / English",
    "features.lang.text": "L'interface suit la locale système, avec un choix manuel dans les paramètres. Les dates et libellés suivent la langue active.",
    "features.web.title": "Recherche web",
    "features.web.text": "Active la recherche web native de lumo-tamer en un clic. Demande la météo, l'actualité ou des faits récents et obtiens des réponses live.",

    "gallery.kicker": "Aperçu",
    "gallery.title": "Un vrai bureau pour ton IA.",
    "gallery.subtitle": "Captures réelles de l'app : fournisseurs, chat, studio d'images Bonsai et agent OpenCode.",
    "gallery.chat.title": "Chat & fournisseurs",
    "gallery.chat.text": "Passe de Lumo à DeepSeek ou à un endpoint personnalisé.",
    "gallery.images.title": "Studio d'images Bonsai",
    "gallery.images.text": "Prompt, seed, résolution, lot et galerie locale.",
    "gallery.agent.title": "Agent OpenCode",
    "gallery.agent.text": "Un agent de code avec appels d'outils en streaming.",
    "gallery.providers.title": "Réglages fournisseurs",
    "gallery.providers.text": "Clés, URLs et modèles au même endroit.",
    "gallery.agentSettings.title": "Réglages agent",
    "gallery.agentSettings.text": "Dossier, modèle, agent et permissions.",
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
    "pipeline.title": "Un seul bureau, plusieurs fournisseurs.",
    "pipeline.desktop": "Model Desk",
    "pipeline.desktopSub": "Rust · Tauri",
    "pipeline.lumo": "Lumo Tamer",
    "pipeline.lumoSub": "API locale → Proton Lumo",
    "pipeline.deepseek": "DeepSeek",
    "pipeline.deepseekSub": "API directe",
    "pipeline.opencode": "OpenCode",
    "pipeline.opencodeSub": "agent de code local",
    "pipeline.bonsai": "Bonsai Image",
    "pipeline.bonsaiSub": "serveur d'images local",
    "pipeline.tamer": "lumo-tamer",
    "pipeline.tamerSub": "API locale",
    "pipeline.proton": "Proton Lumo",
    "pipeline.protonSub": "ton compte",
    "pipeline.note": "Model Desk parle à Lumo via ton lumo-tamer local, à DeepSeek directement, à OpenCode pour les tâches de code et à Bonsai pour générer des images en local. Aucun cloud Model Desk.",

    "privacy.kicker": "Confidentialité",
    "privacy.title": "Tes données restent chez toi.",
    "privacy.subtitle": "Model Desk est un client local, pas un service hébergé.",
    "privacy.local.title": "Stockage local",
    "privacy.local.text": "Conversations, projets, souvenirs, réglages et images générées vivent dans ~/.config/model-desk. Aucun compte cloud pour le client.",
    "privacy.files.title": "Fichiers traités localement",
    "privacy.files.text": "Le texte des PDF, des documents Office et l'OCR des images sont extraits sur ta machine. Seul le texte résultant est envoyé au fournisseur que tu as choisi.",
    "privacy.telemetry.title": "Pas de télémétrie",
    "privacy.telemetry.text": "Aucune analytique, aucun tracking, aucun compte requis pour lancer l'app. Le code source est ouvert.",

    "install.kicker": "Téléchargement",
    "install.title": "Prêt à essayer ?",
    "install.subtitle": "AppImage ou .deb, Linux x86_64. La version 0.4.0 ajoute le studio d'images Bonsai, Lumo Tamer, DeepSeek et l'agent OpenCode.",
    "install.appimage": "AppImage",
    "install.appimageText": "Portable, un seul fichier",
    "install.deb": "Paquet .deb",
    "install.debText": "Debian / Ubuntu / dérivés",
    "install.releases": "GitHub Releases",
    "install.releasesText": "Toutes les versions et sommes de contrôle",
    "install.download": "Télécharger →",
    "install.open": "Ouvrir →",
    "install.comment": "# Rend le fichier exécutable, puis lance-le",
    "install.note": "L'AppImage utilise WebKitGTK 4.1 du système. Pour Lumo, lance lumo-tamer. Pour l'agent, installe OpenCode. Les images Bonsai nécessitent un serveur Bonsai Image Demo local (NVIDIA CUDA ou Apple Silicon). Pour les pièces jointes PDF/OCR, installe poppler-utils, tesseract et unzip.",

    "tech.kicker": "Sous le capot",
    "tech.title": "Rust, Tauri, et rien d'inutile.",
    "tech.rust": "Streaming SSE, outils, pièces jointes",
    "tech.tauri": "Webview système, binaire léger",
    "tech.webkit": "Rendu Linux",
    "tech.storage": "Historique local et lisible",

    "limits.title": "Bon à savoir",
    "limits.one": "La génération d'images Bonsai tourne dans un serveur local séparé et nécessite un GPU NVIDIA/CUDA ou Apple Silicon ; Custom Lumos n'est pas exposé par lumo-tamer.",
    "limits.two": "DeepSeek et les endpoints personnalisés ont leurs propres limites de contexte, tarifs et disponibilité.",
    "limits.three": "La recherche web est un réglage global de lumo-tamer ; l'activer redémarre le service local.",
    "limits.four": "L'agent OpenCode s'exécute avec les permissions que tu lui accordes ; l'auto-approbation lui donne les droits de ton compte utilisateur.",

    "faq.kicker": "FAQ",
    "faq.title": "Questions fréquentes",
    "faq.q1": "Est-ce officiel ?",
    "faq.a1": "Non. Model Desk est indépendant et non affilié à Proton. Lumo est une marque de Proton AG. Le support Lumo passe par l'API non officielle de lumo-tamer.",
    "faq.q2": "Où sont stockées mes données ?",
    "faq.a2": "Dans ~/.config/model-desk. Les conversations, projets, souvenirs et réglages restent locaux.",
    "faq.q3": "Faut-il Docker ?",
    "faq.a3": "Seulement si tu veux Lumo via lumo-tamer. L'installateur utilise Docker, Compose et git. DeepSeek et OpenCode ne nécessitent pas Docker.",
    "faq.q4": "Puis-je utiliser mon installation lumo-tamer existante ?",
    "faq.a4": "Oui. Renseigne le dossier lumo-tamer et la clé API dans Paramètres → Connexion. L'app détecte automatiquement le provider depuis OpenCode si disponible.",
    "faq.q5": "Les images sont-elles envoyées au fournisseur ?",
    "faq.a5": "Pas les images brutes. Pour le chat, le client extrait le texte avec l'OCR local et envoie ce texte. Les prompts d'images Bonsai ne vont qu'à ton backend local.",
    "faq.q6": "Quelle licence ?",
    "faq.a6": "MIT. Le code est open source et les contributions sont bienvenues.",
    "faq.q7": "Quels fournisseurs sont supportés ?",
    "faq.a7": "Lumo via lumo-tamer, DeepSeek, et tout endpoint OpenAI-compatible. OpenCode peut servir d'agent de code avec son propre choix de modèle.",
    "faq.q8": "Peut-on utiliser Model Desk sans Lumo ?",
    "faq.a8": "Oui. Utilise DeepSeek ou un endpoint personnalisé pour le chat, et OpenCode pour les tâches de code. Lumo est optionnel.",

    "cta.title": "Donne un bureau à tes modèles.",
    "cta.text": "Télécharge l'AppImage, choisis Lumo, DeepSeek ou ton propre endpoint, puis discute ou lance une tâche avec l'agent OpenCode.",
    "cta.download": "Télécharger maintenant",
    "cta.github": "Star sur GitHub",

    "footer.readme": "README",
    "footer.disclaimer": "Projet indépendant, non affilié à Proton. Lumo est une marque de Proton AG.",
  },
};

function getLanguage() {
  const requested = new URLSearchParams(location.search).get("lang");
  if (requested === "fr" || requested === "en") return requested;
  try {
    const saved = localStorage.getItem("model-desk-site-lang");
    if (saved === "fr" || saved === "en") return saved;
  } catch {
    // Storage can be unavailable in private contexts.
  }
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

  try {
    localStorage.setItem("model-desk-site-lang", lang);
  } catch {
    // Ignore unavailable storage.
  }
}

document.querySelectorAll("[data-lang]").forEach((button) => {
  button.addEventListener("click", () => applyLanguage(button.dataset.lang));
});

applyLanguage(getLanguage());
