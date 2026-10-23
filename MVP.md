# MVP

Dieses Dokument legt fest, was die erste nutzbare Version von Hodie können muss und was bewusst noch nicht dazugehört.

## Ziel

Eine **To-Do-App als Kanban-Board**, die nur diesen einen Zweck erfüllt und sich dabei intuitiv, modern und schnell anfühlt. Aufgaben werden als Karten in Spalten organisiert und per Drag & Drop oder Tastatur verschoben. Alle Daten liegen lokal auf dem Rechner, ohne Konto und ohne Internet. Läuft unter Windows, macOS und Linux, in Deutsch und Englisch.

## Im Umfang

### Boards und Spalten
- Mehrere Boards (z. B. „Arbeit“, „Privat“), in der Seitenleiste umschaltbar
- Neues Board startet mit den Spalten **Offen**, **In Arbeit** und **Erledigt**
- Spalten anlegen, umbenennen, umsortieren und löschen
- Eine Spalte ist als **Erledigt-Spalte** markiert: Karten darin gelten als erledigt
- Boards anlegen, umbenennen, umsortieren und löschen

### Karten (Aufgaben)
- Titel (Pflicht)
- **Notizen** als Freitext
- **Teilaufgaben** als Checkliste, Fortschritt auf der Karte sichtbar (z. B. „2/5“)
- **Priorität:** keine, niedrig, mittel, hoch
- **Tags:** frei vergebbar, mit einer Farbe aus einer festen Palette, boardübergreifend wiederverwendbar
- Karten innerhalb einer Spalte und zwischen Spalten verschieben, per Drag & Drop **und** per Tastatur
- Details in einem Seitenpanel bearbeiten, das Board bleibt dabei sichtbar
- Erledigte Karten archivieren, Archiv ansehen und Karten wiederherstellen
- Löschen mit **Rückgängig**

### Schnelleingabe
- Neue Karte direkt in einer Spalte per Klick oder `Mod+N`
- **Globales Tastenkürzel**, das auch bei minimierter App ein kleines Eingabefenster öffnet
- Kurzschreibweise im Titel: `#tag` vergibt einen Tag, `!` / `!!` / `!!!` setzt die Priorität (niedrig / mittel / hoch)
- Ziel der Schnelleingabe ist die erste Spalte des zuletzt benutzten Boards

### Finden
- Suche über Titel und Notizen im aktuellen Board
- Filter nach Tag und Priorität

### Darstellung
- **Hell- und Dunkelmodus**, standardmäßig nach Systemeinstellung, manuell umschaltbar
- Umschaltbare Sprache (Deutsch und Englisch)

### Daten
- Lokale SQLite-Datenbank, jede Änderung wird sofort gespeichert
- Kein Konto, keine Netzwerkverbindung

### Vault (Erweiterung nach dem MVP)
- Boards, Spalten, Karten und Tags liegen als Dateien in einem frei wählbaren Ordner, dem **Vault**, eine Datei pro Karte. Das Format steht in [`VAULT.md`](VAULT.md).
- Immer genau ein Vault geöffnet, wechselbar in den Einstellungen
- Mehrere Personen können den Vault über einen Sync-Dienst (Dropbox, iCloud Drive, OneDrive, Syncthing, Netzlaufwerk, Git) gemeinsam nutzen. Hodie bemerkt Änderungen von außen und zeigt Konflikte an.
- Bestehende Daten werden beim ersten Start einmalig aus der Datenbank in `Dokumente/Hodie` übernommen

## Nicht im Umfang

- Fälligkeitsdaten, Kalender- oder „Heute“-Ansicht
- Erinnerungen und Benachrichtigungen
- Wiederkehrende Aufgaben
- Eigene Synchronisation, Konten, Server (geteilt wird nur über den Vault-Ordner und einen fremden Sync-Dienst)
- Zusammenarbeit in Echtzeit (gleichzeitiges Bearbeiten derselben Karte, Anzeige anderer Nutzer)
- Anhänge und Bilder in Karten
- Import und Export (auch Backup-Export)
- Mobile Apps

## Meilensteine

1. **Gerüst:** Tauri-2-App mit React/TypeScript, i18n, Design-Tokens und CI für Windows, macOS und Linux. ✅
2. **Datenmodell und Speicher:** SQLite-Schema, Migrationen, Tauri-Befehle für Boards, Spalten, Karten, Teilaufgaben und Tags, mit Rust-Tests. ✅
3. **Board-Ansicht:** Seitenleiste mit Boards, Spalten mit Karten, Drag & Drop und Tastaturbedienung. ✅
4. **Kartendetails:** Seitenpanel mit Notizen, Teilaufgaben, Priorität und Tags, Archiv, Rückgängig. ✅
5. **Schnelleingabe und Finden:** `Mod+N`, globales Tastenkürzel mit Eingabefenster, Kurzschreibweise, Suche und Filter. ✅
6. **Dunkelmodus und Feinschliff:** zweiter Token-Satz, Theme-Umschalter, Icons, App-Icon, Animationen, Release-Build. ✅
7. **Vault:** Speicher-Schicht auf Dateien nach [`VAULT.md`](VAULT.md) umstellen, Vault wählen und Daten übernehmen, Änderungen von außen beobachten, Konflikte anzeigen. ✅
