# Vault-Format

**Status:** In Umsetzung (Meilenstein 7 in [`MVP.md`](MVP.md)). Die App speichert Boards, Spalten, Karten, Teilaufgaben und Tags im Vault (`src-tauri/src/vault/`). Noch offen: Ordner in den Einstellungen wählen, Änderungen von außen beobachten, Konflikte anzeigen.

Ein **Vault** ist ein Ordner, den der Nutzer in der App auswählt, ähnlich wie bei Obsidian. Er enthält alle Boards als gewöhnliche Dateien. Der Ordner kann über Dropbox, iCloud Drive, OneDrive, Syncthing, ein Netzlaufwerk oder Git mit anderen geteilt werden. Hodie synchronisiert selbst nichts. Es liest und schreibt nur Dateien und bemerkt, wenn andere sie ändern.

## Ziele

- **Konflikte klein halten:** Jede Karte ist eine eigene Datei. Zwei Personen, die verschiedene Karten bearbeiten, kommen sich nicht in die Quere.
- **Kein Datenverlust:** Dateien werden atomar geschrieben. Unlesbare oder unbekannte Dateien werden nie überschrieben oder gelöscht.
- **Lesbar und von Hand bearbeitbar:** UTF-8-Text, Notizen als Markdown. Der Vault lässt sich auch in Obsidian oder einem Texteditor öffnen.
- **Stabile Dateinamen:** Umbenennen in der App benennt keine Dateien oder Ordner um, weil Sync-Dienste daraus oft Doppelungen machen.
- **Plattformübergreifend:** Dateinamen sind unter Windows, macOS und Linux gültig.

## Aufbau

```
Mein Vault/
├── .hodie/
│   ├── vault.json                 Formatversion, Kennung des Vaults
│   └── tags.json                  Tags des Vaults
├── Arbeit/
│   ├── board.json                 Name, Position, Spalten
│   └── cards/
│       ├── 01JB7Q4ZK3M8X2V5T9R6N0PWYH.md
│       └── 01JB7Q6A1C4D7E0F3G6H9J2K5M.md
└── Privat/
    ├── board.json
    └── cards/
        └── …
```

- Ein Board ist ein Ordner direkt im Vault, der eine `board.json` enthält. Andere Ordner und Dateien ignoriert Hodie.
- Der **Ordnername** wird beim Anlegen aus dem Board-Namen gebildet (siehe [Dateinamen](#dateinamen)). Benennt man das Board später um, ändert sich nur `name` in `board.json`, der Ordner behält seinen Namen. Erkannt wird ein Board an seiner `id`, nicht am Ordner.
- Eine Karte ist eine Datei `cards/<id>.md`. Der Dateiname ändert sich nie.
- Dateien und Ordner, die mit `.` beginnen, ignoriert Hodie, außer `.hodie/`.

## Gemeinsame Regeln

- **Kodierung:** UTF-8 ohne BOM. Hodie schreibt Zeilenenden als LF und liest auch CRLF.
- **IDs:** [ULID](https://github.com/ulid/spec), 26 Zeichen in Crockford-Base32, z. B. `01JB7Q4ZK3M8X2V5T9R6N0PWYH`. Sie lassen sich ohne Absprache auf mehreren Rechnern erzeugen, ohne dass sie kollidieren, und sind nach Erstellungszeit sortierbar.
- **Positionen:** Fractional-Index-Schlüssel wie bisher (`src-tauri/src/position.rs`, Base-62). Verschieben ändert nur den Schlüssel der bewegten Karte. Haben zwei Einträge nach einer Zusammenführung denselben Schlüssel, entscheidet die `id`.
- **Zeitpunkte:** ISO 8601 in UTC mit Millisekunden, z. B. `2026-10-23T09:14:05.123Z`.
- **Unbekannte Felder** bleiben beim Zurückschreiben erhalten. Damit verliert eine ältere App-Version keine Daten, die eine neuere angelegt hat.

## `.hodie/vault.json`

```json
{
  "formatVersion": 1,
  "id": "01JB7Q2M5N8P1Q4R7S0T3V6W9X"
}
```

- `formatVersion` erhöht sich nur, wenn sich das Format so ändert, dass ältere Versionen es nicht mehr sicher lesen können. Eine App, die eine **höhere** Version findet, öffnet den Vault nicht (`vault.too_new`).
- Auf eine neue Formatversion umgestellt wird nur nach Rückfrage. Der Hinweis erklärt, dass Mitnutzer mit älteren Versionen den Vault dann nicht mehr öffnen können.
- `id` unterscheidet Vaults, z. B. damit das zuletzt geöffnete Board nach einem Wechsel zum richtigen Vault gehört.

## `.hodie/tags.json`

```json
{
  "tags": [
    { "id": "01JB7Q3A…", "name": "Steuern", "color": "orange" },
    { "id": "01JB7Q3B…", "name": "Einkauf", "color": "green" }
  ]
}
```

- Tags gelten für den ganzen Vault und damit für alle seine Boards, wie bisher.
- `color` ist ein Schlüssel aus der festen Palette (`TAG_COLORS` in `model.rs`).
- Namen sind ohne Unterschied von Groß- und Kleinschreibung eindeutig. Treffen nach einer Zusammenführung zwei gleiche Namen aufeinander, zeigt die App beide an, bis einer umbenannt wird.
- Karten verweisen auf die **ID** eines Tags. Benennt man einen Tag um, ändert sich daher nur diese Datei und keine Karte.

## `<Board>/board.json`

```json
{
  "id": "01JB7Q4A…",
  "name": "Arbeit",
  "position": "V",
  "createdAt": "2026-10-23T09:10:00.000Z",
  "columns": [
    { "id": "01JB7Q4B…", "name": "Offen",     "position": "V", "isDone": false },
    { "id": "01JB7Q4C…", "name": "In Arbeit", "position": "k", "isDone": false },
    { "id": "01JB7Q4D…", "name": "Erledigt",  "position": "t", "isDone": true }
  ]
}
```

- `position` bestimmt die Reihenfolge der Boards in der Seitenleiste. Sie ist für alle Nutzer des Vaults gleich. Eine eigene Reihenfolge je Nutzer bräuchte zusätzlichen lokalen Zustand und Regeln für Boards, die andere neu anlegen. Die geteilte Reihenfolge kommt ohne beides aus, und ein Board zu verschieben ändert nur dessen eigene `board.json`.
- Genau eine Spalte hat `isDone: true`. Hat eine Datei keine oder mehrere, etwa nach einer Zusammenführung von Hand, gilt beim Lesen die letzte markierte bzw. die letzte Spalte. Geschrieben wird diese Korrektur erst bei der nächsten Änderung am Board.

## `<Board>/cards/<id>.md`

Markdown mit YAML-Frontmatter, also derselbe Aufbau wie bei Obsidian, Jekyll und Hugo:

```markdown
---
id: "01JB7Q4ZK3M8X2V5T9R6N0PWYH"
column: "01JB7Q4B…"
position: "V"
title: "Steuererklärung 2025"
priority: high
tags: ["01JB7Q3A…"]
subtasks:
  - id: "01JB7Q5A…"
    title: "Belege sammeln"
    done: true
  - id: "01JB7Q5B…"
    title: "Formular ausfüllen"
    done: false
createdAt: "2026-10-23T09:14:05.123Z"
updatedAt: "2026-10-23T11:02:40.010Z"
---
Unterlagen liegen im Ordner **Finanzen**.

Frist: Ende Juli.
```

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `id` | ja | ID der Karte; maßgeblich ist dieses Feld, nicht der Dateiname |
| `column` | ja | ID der Spalte im selben Board |
| `position` | ja | Sortierschlüssel in der Spalte; bleibt beim Archivieren erhalten |
| `title` | ja | nicht leer |
| `priority` | nein | `none`, `low`, `medium`, `high`; fehlt = `none` |
| `tags` | nein | Liste von Tag-IDs |
| `subtasks` | nein | Liste in Anzeigereihenfolge, jeweils `id`, `title`, `done` |
| `createdAt`, `updatedAt` | ja | Zeitpunkte |
| `archivedAt` | nein | gesetzt = archiviert |

- **Notizen** sind alles nach dem schließenden `---`, unverändert gespeichert. Leere Notizen bedeuten einen leeren Rumpf.
- **Teilaufgaben** stehen in einer Liste im Frontmatter. Ihre Reihenfolge ist die Listenreihenfolge, ein eigener Sortierschlüssel ist nicht nötig, weil ohnehin die ganze Datei neu geschrieben wird. Bewusst keine Markdown-Checkliste (`- [ ]`) im Rumpf, weil die sich nicht sicher von Checklisten in den Notizen trennen lässt.
- **Zeichenketten**, die YAML sonst als etwas anderes lesen würde (z. B. ein Titel `no` oder `1`), setzt Hodie beim Schreiben in Anführungszeichen. Beim Lesen werden Zahlen in Textfeldern als Text übernommen, `position: 1` von Hand ist also gültig.
- **Unbekannte Tag-IDs** (Tag gelöscht, `tags.json` noch nicht synchronisiert) werden nicht angezeigt, bleiben aber in der Datei.
- **Unbekannte Spalte** (Spalte von jemand anderem gelöscht): Die Karte erscheint in der ersten Spalte. Die Datei wird erst geändert, wenn man die Karte bearbeitet oder verschiebt.

## Dateinamen

Gilt nur für Board-Ordner, denn Karten heißen immer nach ihrer ID.

- Diese Zeichen werden durch `-` ersetzt: `< > : " / \ | ? *` und Steuerzeichen.
- Leerzeichen und Punkte am Anfang und Ende werden entfernt (ein Punkt am Anfang würde den Ordner verstecken). Unter Windows reservierte Namen (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`) bekommen ein `-` angehängt.
- Länge höchstens 64 Zeichen. Ist der Name danach leer, heißt der Ordner `Board`.
- Eindeutig ohne Unterschied von Groß- und Kleinschreibung, weil macOS und Windows nicht danach trennen. Bei Gleichheit wird ` 2`, ` 3` … angehängt.

## Schreiben

- **Atomar:** Hodie schreibt zuerst `.<name>.tmp` im selben Ordner, ruft `fsync` auf und benennt die Datei dann über die alte um. Wer gerade mitliest, sieht entweder die alte oder die neue Datei, nie eine halbe. Übrig gebliebene `.tmp`-Dateien werden beim Öffnen gelöscht.
- **Nur was sich ändert:** Eine Karte zu bearbeiten oder zu verschieben schreibt nur ihre eigene Datei. Spalten und Board-Namen ändern nur `board.json`, Tags nur `tags.json`.
- **Notizen** werden wie bisher kurz nach dem Tippen, beim Verlassen des Felds und beim Schließen des Panels gespeichert, nicht bei jedem Tastendruck.

## Lesen und Änderungen von außen

- Beim Öffnen liest Hodie alle Boards in den Speicher. Bei To-do-Mengen sind das wenige Megabyte.
- Hodie beobachtet den Ordner. Ändert sich eine Datei von außen, lädt die App das betroffene Board neu, ohne Neustart. Eigene Schreibvorgänge erkennt sie am zuletzt geschriebenen Inhalt und ignoriert sie.
- Ist die gerade im Panel geöffnete Karte von außen geändert worden, übernimmt das Panel die neue Fassung. Ausnahme: Man tippt gerade in den Notizen. Dann bleibt die eigene Fassung stehen, und ein Hinweis bietet an, die fremde Fassung zu laden.
- **Unlesbare Dateien** (Frontmatter kaputt, Pflichtfeld fehlt) werden übersprungen und in einem Hinweis genannt, aber nie verändert.
- **Suche** läuft über die Karten im Speicher. Die bisherige SQLite-Suche entfällt.

## Konflikte

Sync-Dienste legen bei gleichzeitigen Änderungen Kopien an, mit Namen je nach Dienst (`… (Konfliktkopie …).md`, `… 2.md`, `….sync-conflict-….md`). Hodie erkennt sie nicht am Namen, sondern am Inhalt: **Haben zwei Dateien dieselbe `id`, ist das ein Konflikt.**

- Angezeigt wird die Datei, die `<id>.md` heißt. Die Karte bekommt einen Konflikt-Hinweis.
- Im Kartenpanel lassen sich die Fassungen vergleichen. Man wählt eine aus, die andere Datei wird gelöscht.
- Für `board.json` und `tags.json` gilt dasselbe: Hodie liest die Hauptdatei und weist auf die Kopie hin.

## Löschen und Rückgängig

- Gelöschte Karten und Boards werden im Vault **versteckt umbenannt**: `cards/.<id>.md.deleted` bzw. der Board-Ordner zu `.<Ordner>.<ULID>.deleted`. Für alle anderen (und für Obsidian) sind sie damit sofort weg, weil versteckte Dateien ignoriert werden. Dateien im Board-Ordner, die Hodie nicht kennt, bleiben so ebenfalls erhalten.
- Eine gelöschte Spalte verschwindet aus `board.json`, ihre Karten werden wie oben versteckt.
- **Rückgängig** benennt zurück. Ein Board bekommt dabei einen freien Ordnernamen, falls inzwischen ein anderes Board den alten trägt.
- Versteckte gelöschte Einträge werden beim nächsten Öffnen des Vaults endgültig entfernt, wie bisher die markierten Zeilen. Öffnet jemand anderes den Vault in dieser Zeit, entfernt dessen App sie schon früher; Rückgängig geht dann nicht mehr.
- Teilaufgaben und Tags werden wie bisher sofort gelöscht. Ein Tag wird dabei auch aus allen Karten entfernt, die ihn tragen (nach Rückfrage mit der Anzahl).

## Vault öffnen und wechseln

- Es ist immer **genau ein Vault** geöffnet. Gearbeitet wird vor allem mit mehreren Boards in einem Vault, nicht mit mehreren Vaults.
- In den Einstellungen (Tab Allgemein) steht der Pfad des Vaults. Dort lässt sich ein anderer Ordner öffnen oder ein neuer Vault anlegen. Ein leerer Ordner wird zum Vault, ein Ordner mit `.hodie/vault.json` wird geöffnet, alles andere wird abgelehnt (`vault.not_a_vault`).
- Beim Wechsel lädt die App alle Boards des neuen Vaults. Die Schnelleingabe schreibt immer in den geöffneten Vault.
- Fehlt der Vault beim Start (`vault.not_found`, z. B. Laufwerk nicht verbunden), zeigt die App einen Hinweis mit den Möglichkeiten „Erneut versuchen“ und „Anderen Ordner wählen“. Sie legt dabei nie still einen neuen Vault an.

## Was lokal bleibt

Nicht im Vault, sondern weiter im App-Datenverzeichnis (Tabelle `settings` bzw. `localStorage`):

- Pfad des geöffneten Vaults
- Globales Tastenkürzel, Farb-Presets, Sprache, Theme
- Zuletzt geöffnetes Board und eingeklappte Seitenleiste (je Vault)

Die SQLite-Datenbank bleibt dafür bestehen, enthält aber keine Boards mehr.

## Übernahme der bestehenden Daten

Beim ersten Start der neuen Version:

1. Ist noch kein Vault eingestellt, legt Hodie ohne Rückfrage `Dokumente/Hodie` an (über Tauris Pfad-API; ist der Name durch einen fremden Ordner belegt, `Hodie 2` usw.) und merkt sich den Pfad (`vault_path` in `settings`). Verschieben lässt er sich später in den Einstellungen.
2. Enthält `hodie.db` noch Boards, schreibt Hodie alle Boards, Spalten, Karten, Teilaufgaben und Tags in den Vault. Die bisherigen Zahlen-IDs werden auf neue ULIDs abgebildet, Positionsschlüssel bleiben unverändert, Zeitpunkte werden umgerechnet. Tags mit gleichem Namen werden zusammengeführt. Schlägt das fehl, werden die schon geschriebenen Boards wieder entfernt und der nächste Start versucht es erneut.
3. Danach wird `legacy_imported` in `settings` gesetzt, damit das nur einmal passiert. Die alten Tabellen bleiben unverändert als Sicherung in `hodie.db`. Gelöscht werden sie frühestens in einer späteren Version.

## Neue Fehlercodes

| Code | Bedeutung |
|---|---|
| `vault.not_found` | Der Ordner existiert nicht (mehr), z. B. weil ein Laufwerk fehlt. Ließ sich der Vault beim Start nicht öffnen, antworten alle Board-Befehle mit diesem Code (bzw. dem Code, mit dem das Öffnen scheiterte) |
| `vault.not_a_vault` | Der Ordner hat keine `.hodie/vault.json` und ist nicht leer |
| `vault.too_new` | Formatversion höher als unterstützt |
| `vault.read_only` | Keine Schreibrechte im Ordner |
| `vault.io` | Sonstiger Dateifehler, z. B. Laufwerk während der Arbeit getrennt |
| `file.invalid` | Eine Datei ließ sich nicht lesen. Beim Laden nur ein Hinweis mit Dateiname, beim Schreiben wird das Überschreiben abgelehnt |

## Abhängigkeiten (abgestimmt)

| Zweck | Vorschlag |
|---|---|
| ULIDs erzeugen | `ulid` (installiert) |
| YAML-Frontmatter lesen und schreiben | `serde_yaml_ng` (installiert) |
| Ordner beobachten | `notify`, auf allen drei Systemen nativ (noch nicht installiert) |
| Ordner auswählen | `tauri-plugin-dialog` (noch nicht installiert) |

## Entscheidungen

- Die Reihenfolge der Boards ist für alle Nutzer gleich (einfacher, siehe `board.json`).
- Es ist immer nur ein Vault geöffnet.
- Das Frontmatter ist YAML.
- Der Vault gehört zum Umfang (Meilenstein 7 in `MVP.md`).
- Die Abhängigkeiten oben sind freigegeben.
