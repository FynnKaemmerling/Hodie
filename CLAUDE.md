# CLAUDE.md

Leitfaden für Claude Code bei der Arbeit an diesem Repository.

## Projekt

**Hodie** (lat. „heute“) ist eine Desktop-To-Do-App als Kanban-Board: Boards mit Spalten, Aufgaben als Karten mit Notizen, Teilaufgaben, Priorität und Tags. Sie soll intuitiv, modern und funktional sein und nur diesen einen Zweck erfüllen. Alle Daten liegen lokal.

**Status:** Meilenstein 2 (Datenmodell und Speicher) ist fertig: SQLite-Schema mit Migrationen und alle Tauri-Befehle für Boards, Spalten, Karten, Teilaufgaben und Tags. Die Oberfläche nutzt sie noch nicht, sie ist noch das Gerüst aus Meilenstein 1.

**Umfang:** Was zum MVP gehört und was bewusst nicht, steht in [`MVP.md`](MVP.md). Keine Funktionen aus „Nicht im Umfang“ bauen, solange der Nutzer das nicht ausdrücklich verlangt.

**Zielplattformen:** Windows, macOS und Linux gleichberechtigt. Keine Lösung darf nur auf einem System funktionieren.

## Tech-Stack

| Bereich | Wahl |
|---|---|
| App-Gerüst | Tauri 2 |
| Backend | Rust |
| Frontend | React + TypeScript + Vite |
| Lokaler Speicher | SQLite über `rusqlite` (gebündelt) |
| Drag & Drop | `@dnd-kit` (mit Tastatur- und Screenreader-Unterstützung) *(geplant)* |
| Globales Tastenkürzel | `tauri-plugin-global-shortcut` *(geplant)* |
| i18n | i18next / react-i18next |
| Plattform-Funktionen | Offizielle Tauri-Plugins |

Mit *(geplant)* markierte Abhängigkeiten sind vorgeschlagen, aber noch nicht installiert.

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
src-tauri/src/commands.rs     Tauri commands (the only interface to the frontend)
src-tauri/src/model.rs        Types exchanged with the frontend (camelCase JSON)
src-tauri/src/error.rs        Error codes sent to the frontend
src-tauri/src/position.rs     Fractional-index sort keys
src-tauri/src/store/          Storage layer, one module per entity
src-tauri/src/store/migrations/  SQL migrations, applied in order via user_version
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

1. **Klare Schichten trennen**
   - *Speicher-Schicht (Rust):* SQLite-Datenbank, Schema-Migrationen, alle Lese- und Schreibzugriffe. Keine UI-Abhängigkeiten.
   - *Befehls-Schicht (Rust):* Tauri-Befehle als einzige Schnittstelle zum Frontend. Sie validieren Eingaben und geben Daten oder Fehlercodes zurück.
   - *UI-Schicht (React):* Spricht ausschließlich über diese Befehle mit dem Backend, nie direkt mit der Datenbank.
2. **Datenmodell:** Board → Spalte → Karte → Teilaufgabe. Tags gehören keinem Board und werden Karten über eine Zuordnungstabelle zugewiesen. Genau eine Spalte pro Board ist die Erledigt-Spalte.
3. **Reihenfolge stabil speichern:** Position von Boards, Spalten, Karten und Teilaufgaben als eigenes Sortierfeld, das beim Verschieben nur die bewegte Zeile ändert (z. B. Fractional Indexing), nicht alle Geschwister neu nummeriert.
4. **Optimistische Oberfläche:** Änderungen erscheinen sofort, werden im Hintergrund gespeichert und bei einem Fehler zurückgenommen, mit Hinweis an den Nutzer.
5. **Kein Datenverlust:** Jede Änderung wird sofort gespeichert. Gelöschtes bleibt kurz wiederherstellbar (Rückgängig). Schema-Änderungen nur über Migrationen, nie durch Neuanlegen der Datenbank.
6. **Reine Logik testbar halten:** Kurzschreibweise der Schnelleingabe, Sortierlogik und Filter als reine Funktionen mit Tests.
7. **Datenbankpfad** über Tauris Pfad-API (App-Datenverzeichnis), nie fest eintragen.

### Speicher im Detail

- **Migrationen:** `src-tauri/src/store/migrations/NNN_name.sql`, in `MIGRATIONS` (`store/mod.rs`) eingetragen. Veröffentlichte Migrationen nie ändern, nur neue anhängen. Eine Datenbank aus einer neueren Version wird mit `db.too_new` abgelehnt.
- **Löschen und Rückgängig:** Boards, Spalten und Karten werden nur markiert (`deleted_at`) und lassen sich mit `restore_*` zurückholen. Beim nächsten Start werden markierte Zeilen endgültig gelöscht. Teilaufgaben und Tags werden sofort gelöscht.
- **Archiv:** Archivierte Karten (`archived_at`) haben keinen Platz in der Spaltenreihenfolge. Beim Zurückholen kommen sie an ihre alte Stelle, falls frei, sonst direkt dahinter.
- **Reihenfolge:** Verschiebe-Befehle bekommen den Zielindex. Das Backend berechnet daraus den Sortierschlüssel zwischen den Nachbarn.
- **Fehlercodes:** `not_found`, `validation.empty`, `board.no_columns`, `column.is_done`, `card.archived`, `card.not_archived`, `tag.duplicate_name`, `tag.invalid_color`, `db.too_new`, `db.error`, `internal`.

## Plattformübergreifend

- Tauri nutzt den Browser-Baustein des jeweiligen Systems: WebView2 (Chromium) unter Windows, WKWebView (Safari) unter macOS, WebKitGTK unter Linux. Neue CSS- und JS-Funktionen nur verwenden, wenn alle drei sie unterstützen.
- Die Systemschrift des jeweiligen Betriebssystems verwenden, keine fest eingestellte Windows-Schrift.
- Tastenkürzel zentral über ein „Mod“-Kürzel definieren: `Strg` unter Windows und Linux, `Cmd` unter macOS.
- Menüs: auf macOS in der globalen Menüleiste, unter Windows und Linux im Fenster.
- Dateipfade (Datenbank, Cache, Einstellungen, Logs) ausschließlich über Tauris Pfad-APIs ermitteln, nie fest eintragen.
- Gebaut wird jedes System auf sich selbst, nicht per Cross-Compiling. Releases laufen über GitHub Actions mit `tauri-action`.

## UI-Leitlinien

Die Designrichtung steht in [`DESIGN.md`](DESIGN.md). Kernregeln: alle visuellen Werte über CSS-Variablen (Tokens), keine festen Farben in Komponenten, keine Bibliothek mit festem Look, Hell- und Dunkelmodus über denselben Satz semantischer Tokens, Barrierefreiheit (mindestens WCAG AA in beiden Themes, vollständige Tastaturbedienung auch für Drag & Drop, Screenreader-Beschriftungen) von Anfang an.

- Kanban-Layout: Seitenleiste mit Boards links, Spalten nebeneinander, Kartendetails im Seitenpanel rechts.
- Sofort speichern, kein Speichern-Button. Löschen mit Rückgängig statt Bestätigungsdialog.
- Tastenkürzel zentral definieren, siehe Tabelle in `DESIGN.md`.

## Arbeitsweise

- Kleine, fokussierte Änderungen. Vor größeren Architektur- oder Abhängigkeitsentscheidungen kurz Rücksprache halten.
- Neue Funktionen mit Tests absichern, besonders Speicher-Schicht, Migrationen und Sortierlogik.
- Commits nur auf ausdrückliche Anweisung. Commit-Messages auf Englisch im Imperativ (z. B. `Add settings view`).
- Diese Datei aktuell halten, sobald sich Stack, Befehle oder Struktur ändern.
