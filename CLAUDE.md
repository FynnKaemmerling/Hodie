# CLAUDE.md

Leitfaden für Claude Code bei der Arbeit an diesem Repository.

## Projekt

**Hodie** (lat. „heute“) ist eine Desktop-To-Do-App als Kanban-Board: Boards mit Spalten, Aufgaben als Karten mit Notizen, Teilaufgaben, Priorität und Tags. Sie soll intuitiv, modern und funktional sein und nur diesen einen Zweck erfüllen. Alle Daten liegen lokal.

**Status:** Der MVP ist vollständig umgesetzt (alle sechs Meilensteine in [`MVP.md`](MVP.md)). Weitere Funktionen nur nach Absprache, Ideen stehen unter „Später“ in der README.

**Umfang:** Was zum MVP gehört und was bewusst nicht, steht in [`MVP.md`](MVP.md). Keine Funktionen aus „Nicht im Umfang“ bauen, solange der Nutzer das nicht ausdrücklich verlangt.

**Zielplattformen:** Windows, macOS und Linux gleichberechtigt. Keine Lösung darf nur auf einem System funktionieren. Mobile Plattformen sind nicht vorgesehen; Desktop-Plugins werden deshalb ohne Plattform-Weiche eingebunden.

## Tech-Stack

| Bereich | Wahl |
|---|---|
| App-Gerüst | Tauri 2 |
| Backend | Rust |
| Frontend | React + TypeScript + Vite |
| Lokaler Speicher | SQLite über `rusqlite` (gebündelt) |
| UI-Bausteine | React Aria Components (Verhalten und Barrierefreiheit, kein eigenes Aussehen) |
| Icons | Lucide (`lucide-react`), Größe und Strichstärke über Tokens |
| Drag & Drop | `@dnd-kit` (mit Tastatur- und Screenreader-Unterstützung) |
| Globales Tastenkürzel | `tauri-plugin-global-shortcut` |
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
src/api/                Typed wrappers for the Tauri commands, error codes
src/board/              Board, card and archive state: pure update functions (tested), hooks
src/notifications/      Toasts with undo (also Mod+Z)
src/quickAdd/           The quick entry window (label "quick-add", opened by the global shortcut)
src/theme.ts            Theme preference (system/light/dark) and data-theme
src/palette.ts          Colour presets: built-ins, token derivation, contrast checks (tested)
src/colorPreset.ts      Applying the active preset, its cache and the settings hook
src/shortcut.ts         Recording and displaying global shortcuts (tested)
src/sidebar.ts          Collapsed state of the sidebar (localStorage)
src/format.ts           Locale-aware formatting helpers, e.g. byte sizes (tested)
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
src-tauri/src/quick_add.rs    Global shortcut: registration, settings commands
src-tauri/src/panel.rs        macOS only: quick entry window as non-activating panel
src-tauri/src/store/          Storage layer, one module per entity
src-tauri/src/store/migrations/  SQL migrations, applied in order via user_version
src-tauri/capabilities/ Tauri permissions for the frontend
.github/workflows/      CI builds for all three platforms
```

Styling: CSS Modules (`*.module.css`) neben der Komponente, ausschließlich mit Variablen aus `tokens.css`.

Optimistische Änderungen laufen über `useBoard`: Erst wird der Zustand mit einer reinen Funktion aus `boardState.ts` geändert, dann der Befehl aufgerufen. Schlägt er fehl, erscheint der übersetzte Fehler und das Board wird neu geladen. Das Kartenpanel arbeitet genauso über `useCardDetail` (`cardState.ts`) und spiegelt jede Änderung per `patchCard` in Board und Archiv. Notizen werden kurz nach dem Tippen, beim Verlassen des Felds und beim Schließen des Panels gespeichert.

**Fensterrahmen:** Das Hauptfenster ist unter Windows und Linux rahmenlos (`decorations: false`); `WindowControls` zeichnet die Knöpfe, Ziehflächen sind mit `data-tauri-drag-region` markiert (gilt nur für das Element selbst, nicht für Kinder). macOS überschreibt das in `tauri.macos.conf.json` (Ampelknöpfe, `titleBarStyle: Overlay`); `<html data-platform="mac">` schafft dort Platz im Seitenleistenkopf. Die Seitenleiste ist auf macOS durchscheinend: Das Hauptfenster ist dort transparent mit `windowEffects` (`sidebar`, braucht `macOSPrivateApi` und das Cargo-Feature `macos-private-api`), `<html data-vibrancy>` macht den Seitenleisten-Hintergrund durchsichtig; der Hauptbereich bleibt deckend. Wird das Hauptfenster geschlossen, beendet `lib.rs` die App, sonst liefe sie wegen des versteckten Schnelleingabe-Fensters unsichtbar weiter.

**Zwei Fenster, ein Frontend:** `main.tsx` zeigt je nach Fensterlabel (`main` oder `quick-add`, siehe `tauri.conf.json`) die App oder die Schnelleingabe. Das Tastenkürzel registriert das Backend (`quick_add.rs`) beim Start aus der Tabelle `settings`, Standard ist `Mod+Umschalt+Leertaste`. Es lässt sich in den Einstellungen ändern; ist ein neues Kürzel belegt, bleibt das alte aktiv (`shortcut.unavailable`). Ist es schon beim Start belegt, läuft die App ohne es weiter und die Einstellungen zeigen einen Hinweis. Legt die Schnelleingabe eine Karte an, sendet sie das Ereignis `cards-changed` (`src/api/events.ts`), und das Hauptfenster lädt neu.

**Suche und Filter:** Die Textsuche läuft im Backend (`search_cards`), weil Notizen nicht in den Kartenübersichten stecken. Tag- und Prioritätsfilter wirken im Frontend (`filter.ts`). Solange ein Filter aktiv ist, ist Drag & Drop gesperrt, weil Positionen unter den sichtbaren Karten nicht den echten entsprechen.

**Themes:** `tokens.css` enthält einen hellen Satz in `:root` und einen dunklen unter `:root[data-theme="dark"]` mit denselben Namen. `src/theme.ts` setzt `data-theme` aus der Einstellung (Wie System, Hell, Dunkel), folgt bei „Wie System“ der Systemeinstellung und färbt auch den Fensterrahmen. Neue Farben immer in **beiden** Sätzen anlegen.

**Farb-Presets:** Ein Preset besteht aus Akzent und Grundton für Hell und Dunkel. „Hodie“ ist der handabgestimmte Satz in `tokens.css`; für alle anderen (eingebaut „Ozean“, „Wald“ sowie eigene) leitet `derivePalette` (`palette.ts`) Flächen, Rahmen und Akzent-Töne ab und setzt sie als Inline-Variablen auf `<html>`. Die Hauptfläche (`--color-bg-surface`, `--color-bg-card`) sowie Text-, Status-, Prioritäts- und Tag-Farben bleiben unverändert. Eigene Presets liegen in der Tabelle `color_presets`, die Auswahl in `settings` (`color_preset`: Name eines eingebauten oder `custom:<id>`). Die aufgelösten Farben werden im `localStorage` zwischengespeichert, damit beide Fenster sofort richtig malen und das Schnellfenster Änderungen mitbekommt.

**App-Icon:** Quelle ist `src-tauri/icons/app-icon.svg`. Nach Änderungen alle Größen mit `npx tauri icon src-tauri/icons/app-icon.svg` neu erzeugen und die Ordner `android/` und `ios/` wieder löschen.

**Einstellungen** liegen im Dialog „Einstellungen“ (Knopf unten in der Seitenleiste) mit den Tabs Allgemein (Sprache, Darstellung, Tastenkürzel), Farben (Presets), Tags und Info (Version über `getVersion`, Speicher über `get_storage_usage`). Ein Tag, der an Karten hängt, wird erst nach Rückfrage mit der Anzahl gelöscht (`count_tag_cards`).

Das Tastenkürzel steht in der Tabelle `settings` im Backend, weil es vor dem Frontend gebraucht wird. Sprache, Theme, das zuletzt geöffnete Board und die eingeklappte Seitenleiste liegen vorerst im `localStorage`; beide Fenster teilen ihn und hören über das `storage`-Ereignis auf Änderungen des anderen. Neue Einstellungen gehören in die Tabelle `settings`.

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
- **Fehlercodes:** `not_found`, `validation.empty`, `board.no_columns`, `column.is_done`, `card.archived`, `card.not_archived`, `tag.duplicate_name`, `tag.invalid_color`, `preset.invalid_color`, `shortcut.invalid`, `shortcut.unavailable`, `db.too_new`, `db.error`, `internal`.

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
- Schlicht: Aktionen als Icon-Knöpfe ohne Beschriftung über `IconButton` (Name als `aria-label` und Tooltip).
- Tastenkürzel zentral definieren, siehe Tabelle in `DESIGN.md`.

## Arbeitsweise

- Kleine, fokussierte Änderungen. Vor größeren Architektur- oder Abhängigkeitsentscheidungen kurz Rücksprache halten.
- Neue Funktionen mit Tests absichern, besonders Speicher-Schicht, Migrationen und Sortierlogik.
- Commits nur auf ausdrückliche Anweisung. Commit-Messages auf Englisch im Imperativ (z. B. `Add settings view`).
- Diese Datei aktuell halten, sobald sich Stack, Befehle oder Struktur ändern.
