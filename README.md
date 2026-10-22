# Hodie

Eine Desktop-App. *(Beschreibung folgt.)*

Läuft unter **Windows**, **macOS** und **Linux**.

> **Status:** In Entwicklung. Das App-Gerüst steht, fachliche Funktionen fehlen noch.

## Über den Namen

**Hodie** ist Latein und bedeutet „heute“.

## Ziele

- **Plattformübergreifend:** Ein Programm für Windows, macOS und Linux.
- **Mehrsprachig:** Sprache zur Laufzeit umschaltbar, Deutsch als Standard.

## Geplante Funktionen

Die Festlegung mit Abgrenzung und Meilensteinen steht in [`MVP.md`](MVP.md).

## Tech-Stack

| Bereich | Technik |
|---|---|
| App-Gerüst | [Tauri 2](https://tauri.app) |
| Backend | Rust |
| Oberfläche | React + TypeScript + Vite |
| Mehrsprachigkeit | i18next |

Tauri nutzt den Browser-Baustein des Betriebssystems statt eines mitgelieferten Chromium. Dadurch bleibt Hodie klein und sparsam, während die Oberfläche mit HTML und CSS frei gestaltet werden kann.

## Bauen und Starten

**Voraussetzungen:** [Node.js](https://nodejs.org) (LTS), [Rust](https://rustup.rs) sowie die [Tauri-Voraussetzungen](https://tauri.app/start/prerequisites/) für das jeweilige Betriebssystem.

```sh
npm install              # Abhängigkeiten installieren
npm run tauri dev        # App im Entwicklungsmodus starten
npm run tauri build      # Programm und Installer bauen
npm test                 # Tests ausführen
```

`npm run tauri build` erzeugt das Programm für das System, auf dem gebaut wird:

| System | Ergebnis |
|---|---|
| Windows | `.exe`, Installer als `.msi` / `.exe` |
| macOS | `.app` in einer `.dmg` |
| Linux | `.AppImage`, `.deb`, `.rpm` |

Bei jedem Push auf `main` baut GitHub Actions die App für alle drei Systeme. Die Installer liegen danach als Artefakte am jeweiligen Workflow-Lauf.

## Entwicklung

Hinweise für die Arbeit am Projekt (Konventionen, Architektur, Arbeitsweise) stehen in [`CLAUDE.md`](CLAUDE.md).

## Lizenz

Noch nicht festgelegt.
