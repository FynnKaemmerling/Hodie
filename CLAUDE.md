# CLAUDE.md

Leitfaden für Claude Code bei der Arbeit an diesem Repository.

## Projekt

**Hodie** ist eine Desktop-App. *(Beschreibung folgt.)*

**Status:** Meilenstein 1 (Gerüst) ist aufgesetzt: Tauri-2-App mit React/TypeScript, i18n und Design-Tokens. Es gibt noch keine fachlichen Funktionen.

**Umfang:** Was zum MVP gehört und was bewusst nicht, steht in [`MVP.md`](MVP.md). Keine Funktionen aus „Nicht im Umfang“ bauen, solange der Nutzer das nicht ausdrücklich verlangt.

**Zielplattformen:** Windows, macOS und Linux gleichberechtigt. Keine Lösung darf nur auf einem System funktionieren.

## Tech-Stack

| Bereich | Wahl |
|---|---|
| App-Gerüst | Tauri 2 |
| Backend | Rust |
| Frontend | React + TypeScript + Vite |
| i18n | i18next / react-i18next |
| Plattform-Funktionen | Offizielle Tauri-Plugins |

Weitere Abhängigkeiten nur nach Rücksprache hinzufügen. Bevorzugt werden gepflegte Bibliotheken, die auf allen drei Systemen funktionieren.

### Befehle

```sh
npm install                       # Install dependencies
npm run tauri dev                 # Start the app in development mode
npm run tauri build               # Build executables and installers for the current OS
npm run dev                       # Frontend only in the browser (no Tauri APIs available)

npm run typecheck                 # TypeScript check
npm test                          # Frontend tests (Vitest)
npx vitest run src/locales        # Run a single test file or folder

cargo fmt --check                 # Rust formatting (in src-tauri/)
cargo clippy --all-targets -- -D warnings   # Rust lint (in src-tauri/)
cargo test                        # Rust tests (in src-tauri/)
cargo test <name>                 # Run a single Rust test
```

CI (`.github/workflows/build.yml`) führt all diese Prüfungen auf Windows, macOS und Linux aus und baut dort die Installer. Vor einem Commit mindestens `npm run typecheck` und `npm test` ausführen, bei Rust-Änderungen zusätzlich `cargo fmt`, `cargo clippy` und `cargo test`.

### Verzeichnisstruktur

```
src/                    React frontend
src/components/         UI components (one component + its CSS module per file pair)
src/i18n/               i18next setup and language switching
src/locales/            Translation files (de.json, en.json); tests check that keys match
src/styles/tokens.css   Design tokens (the only place for raw colors, sizes, etc.)
src/styles/global.css   Global base styles
src-tauri/              Rust backend
src-tauri/capabilities/ Tauri permissions for the frontend
.github/workflows/      CI builds for all three platforms
```

Styling: CSS Modules (`*.module.css`) neben der Komponente, ausschließlich mit Variablen aus `tokens.css`.

Die Sprachwahl wird vorerst im `localStorage` gespeichert. Sobald es Einstellungen im Rust-Backend gibt, zieht sie dorthin um.

## Sprache

- Kommunikation mit dem Nutzer (Antworten, Rückfragen, Zusammenfassungen): immer **Deutsch**
- Dokumentation: **Deutsch**
- Code (Bezeichner) und Commit-Messages: **Englisch**
- Code-Kommentare: Englisch, sparsam, nur wo das *Warum* nicht offensichtlich ist

## Mehrsprachigkeit (i18n)

Die App muss von Anfang an zur Laufzeit zwischen Sprachen umschaltbar sein, ohne Neustart.

- **Keine** UI-Texte im Code hartkodieren. Alle Texte über i18next-Schlüssel aus `src/locales/<sprache>.json` laden.
- Deutsch ist Standard und Referenzsprache. Neue Schlüssel immer in **allen** vorhandenen Sprachdateien anlegen.
- Das Rust-Backend gibt keine fertigen Texte an die Oberfläche zurück, sondern nur **Fehler- bzw. Statuscodes**. Die Übersetzung passiert ausschließlich im Frontend.
- Datum, Uhrzeit, Zahlen und Größenangaben mit `Intl` passend zur gewählten Sprache formatieren, nie manuell zusammensetzen.
- Pluralformen über i18next abbilden, nicht über `if (n === 1)`.

## Architektur-Leitlinien

*(Folgt, sobald der fachliche Umfang feststeht.)*

## Plattformübergreifend

- Tauri nutzt den Browser-Baustein des jeweiligen Systems: WebView2 (Chromium) unter Windows, WKWebView (Safari) unter macOS, WebKitGTK unter Linux. Neue CSS- und JS-Funktionen nur verwenden, wenn alle drei sie unterstützen.
- Die Systemschrift des jeweiligen Betriebssystems verwenden, keine fest eingestellte Windows-Schrift.
- Tastenkürzel zentral über ein „Mod“-Kürzel definieren: `Strg` unter Windows und Linux, `Cmd` unter macOS.
- Menüs: auf macOS in der globalen Menüleiste, unter Windows und Linux im Fenster.
- Dateipfade (Datenbank, Cache, Einstellungen, Logs) ausschließlich über Tauris Pfad-APIs ermitteln, nie fest eintragen.
- Gebaut wird jedes System auf sich selbst, nicht per Cross-Compiling. Releases laufen über GitHub Actions mit `tauri-action`.

## UI-Leitlinien

Die Designrichtung steht in [`DESIGN.md`](DESIGN.md). Kernregeln: alle visuellen Werte über CSS-Variablen (Tokens), keine festen Farben in Komponenten, keine Bibliothek mit festem Look, Barrierefreiheit (mindestens WCAG AA, vollständige Tastaturbedienung, Screenreader-Beschriftungen) von Anfang an.

## Arbeitsweise

- Kleine, fokussierte Änderungen. Vor größeren Architektur- oder Abhängigkeitsentscheidungen kurz Rücksprache halten.
- Neue Funktionen mit Tests absichern.
- Commits nur auf ausdrückliche Anweisung. Commit-Messages auf Englisch im Imperativ (z. B. `Add settings view`).
- Diese Datei aktuell halten, sobald sich Stack, Befehle oder Struktur ändern.
