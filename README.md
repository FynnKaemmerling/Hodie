# Hodie

Eine To-Do-App als Kanban-Board: intuitiv, modern und funktional.

Läuft unter **Windows**, **macOS** und **Linux**.

> **Status:** In Entwicklung. Das App-Gerüst steht, fachliche Funktionen fehlen noch.

## Über den Namen

**Hodie** ist Latein und bedeutet „heute“.

## Ziele

- **Nur To-Dos:** Ein Werkzeug für genau einen Zweck, ohne Kalender, Chat oder Wiki drumherum.
- **Intuitiv:** Karten dorthin ziehen, wo sie hingehören. Alles direkt am Board bearbeitbar.
- **Schnell:** Sofortiger Start, Aufgaben per globalem Tastenkürzel von überall erfassen.
- **Vollständig per Tastatur bedienbar,** inklusive Verschieben von Karten.
- **Lokal und privat:** Alle Daten bleiben auf dem Rechner, kein Konto, kein Internet nötig.
- **Plattformübergreifend:** Ein Programm für Windows, macOS und Linux.
- **Mehrsprachig:** Sprache zur Laufzeit umschaltbar, Deutsch als Standard.

## Geplante Funktionen

### Kern (MVP)

Die vollständige Festlegung mit Abgrenzung und Meilensteinen steht in [`MVP.md`](MVP.md).

- Mehrere Boards mit frei definierbaren Spalten (Standard: Offen, In Arbeit, Erledigt)
- Karten mit Notizen, Teilaufgaben, Priorität und Tags
- Drag & Drop und Tastaturbedienung für Karten und Spalten
- Schnelleingabe mit globalem Tastenkürzel und Kurzschreibweise (`#tag`, `!!`)
- Suche und Filter nach Tag und Priorität
- Archiv für erledigte Karten, Rückgängig beim Löschen
- Hell- und Dunkelmodus
- Deutsch und Englisch

### Später

- Fälligkeitsdaten und Erinnerungen
- Wiederkehrende Aufgaben
- Import, Export und Backups
- Synchronisation zwischen Geräten

## Oberfläche

```
┌────────────┬──────────────┬──────────────┬──────────────┐
│ Boards     │ Offen     3  │ In Arbeit 2  │ Erledigt  4  │
│  ● Arbeit  │ ┌──────────┐ │ ┌──────────┐ │ ┌──────────┐ │
│  ○ Privat  │ │ Titel    │ │ │ Titel    │ │ │ Titel    │ │
│            │ │ #tag     │ │ │ ☑ 2/5    │ │ │          │ │
│ ＋ Board   │ └──────────┘ │ └──────────┘ │ └──────────┘ │
└────────────┴──────────────┴──────────────┴──────────────┘
```

Details zur Designrichtung stehen in [`DESIGN.md`](DESIGN.md).

## Tech-Stack

| Bereich | Technik |
|---|---|
| App-Gerüst | [Tauri 2](https://tauri.app) |
| Backend | Rust |
| Oberfläche | React + TypeScript + Vite |
| Lokaler Speicher | SQLite |
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
