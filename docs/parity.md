# Parité avec Branlly Windows

La version PowerShell/WPF reste la spécification fonctionnelle. Une fonction n’est considérée portée que si elle est disponible sur une plateforme réelle, testée et présentée dans le HUD.

| Domaine | Fonction | État Branlly Next |
|---|---|---|
| Compagnon | 32 frames transparentes | ✅ |
| Compagnon | déplacement manuel | ✅ |
| Compagnon | humeur et énergie | ✅ cœur persistant et télémétrie HUD |
| Compagnon | suivi du curseur | ✅ Windows et X11 ; désactivé proprement sous Wayland |
| Compagnon | sons et réactions | ✅ sons HUD synthétisés |
| Interface | HUD radial | ✅ |
| Interface | sous-menus complets | ✅ chat, applications, système, recherche et jeux |
| IA | Ollama `qwen2.5:3b` | ✅ |
| IA | streaming et annulation | ✅ |
| IA | personnalité historique | ✅ prompt local |
| IA | mémoire persistante sur disque | ✅ JSON versionné et écriture atomique |
| Applications | raccourcis autorisés | ✅ Discord, Steam, Twitch, YouTube Music, Stremio et Disney+ |
| Applications | fenêtres ouvertes | ✅ Windows et X11 avec `wmctrl` |
| Applications | activation/fermeture | ✅ identifiants validés, sans interpolation shell |
| Système | état réseau/Wi-Fi | ✅ NetworkManager et profils Windows |
| Système | Bluetooth | ✅ BlueZ et PnP Windows |
| Système | périphériques | ✅ USB sous Linux, PnP présents sous Windows et Bluetooth |
| Système | nettoyage | ✅ fichiers temporaires de plus de 24 heures avec confirmation |
| Recherche | recherche Wikipédia | ✅ API française, requête bornée |
| Jeux | Metro Rush | ✅ |
| Jeux | BlockCraft Lite | ✅ |
| Jeux | Pong | ➖ hors périmètre à la demande du propriétaire |
| Jeux | Desktop Whip | ➖ hors périmètre à la demande du propriétaire |
| Distribution | AppImage et DEB | ✅ |
| Distribution | MSI et EXE | ✅ |

Légende : ✅ porté ; 🟡 partiel ; ⬜ à porter ; ➖ volontairement retiré.
