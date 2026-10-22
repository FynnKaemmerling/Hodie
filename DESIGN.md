# Design

Dieses Dokument legt die Designrichtung von Hodie fest. Es beschreibt Prinzipien und Grundentscheidungen, kein fertiges Aussehen. Das konkrete Design entsteht an der echten Board-Ansicht.

## Leitidee

**Ein Kanban-Board, das nur To-Dos kann, das aber richtig: intuitiv, modern, funktional.**

- **Intuitiv:** Alles Wichtige ist sichtbar und direkt bedienbar. Karten zieht man dorthin, wo sie hingehören, Texte bearbeitet man durch Anklicken. Keine Einstellungsseiten für Dinge, die man auch direkt am Board erledigen kann.
- **Modern:** Großzügiger Weißraum, weiche Flächen statt harter Rahmen, dezente Schatten auf Karten, flüssige Animationen beim Verschieben.
- **Funktional:** Schnell erfassen, schnell finden. Jede Aktion ist auch per Tastatur erreichbar, die App startet sofort und reagiert ohne Verzögerung.

## Layout

```
┌────────────┬────────────────────────────────────────────────────┐
│ Hodie      │ Board-Name          Suche · Filter · ＋ Karte       │
│            ├──────────────┬──────────────┬──────────────┬───────┤
│ Boards     │ Offen     3  │ In Arbeit 2  │ Erledigt  4  │ ＋    │
│  ● Arbeit  │ ┌──────────┐ │ ┌──────────┐ │ ┌──────────┐ │Spalte │
│  ○ Privat  │ │ Titel    │ │ │ Titel    │ │ │ Titel    │ │       │
│            │ │ #tag  !! │ │ │ ☑ 2/5    │ │ │          │ │       │
│ ＋ Board   │ └──────────┘ │ └──────────┘ │ └──────────┘ │       │
│            │ ┌──────────┐ │              │              │       │
│ Archiv     │ │ Titel    │ │              │              │       │
│            │ └──────────┘ │              │              │       │
│ ⚙          │ ＋ Karte     │ ＋ Karte     │ ＋ Karte     │       │
└────────────┴──────────────┴──────────────┴──────────────┴───────┘
```

- **Seitenleiste links:** Boards, Archiv, Einstellungen (Sprache, Theme). Einklappbar.
- **Board:** Spalten nebeneinander mit fester Breite, horizontal scrollbar. Jede Spalte zeigt die Anzahl ihrer Karten.
- **Karte:** Titel, darunter nur, was vorhanden ist: Tags, Prioritätsmarkierung, Fortschritt der Teilaufgaben, Hinweis auf Notizen. Leere Felder nehmen keinen Platz ein.
- **Kartendetails** öffnen sich in einem **Seitenpanel rechts**. Das Board bleibt sichtbar und bedienbar. `Esc` schließt das Panel.
- **Schnelleingabe:** kleines, randloses Fenster mit einem Eingabefeld, mittig auf dem Bildschirm. `Enter` legt die Karte an, `Esc` schließt.

## Interaktion

- **Drag & Drop** für Karten und Spalten, mit sichtbarer Einfügemarke und weicher Animation der verdrängten Karten.
- **Tastatur gleichwertig:** Pfeiltasten bewegen den Fokus zwischen Karten und Spalten, `Leertaste` hebt eine Karte auf, Pfeiltasten verschieben sie, `Leertaste` legt sie ab, `Esc` bricht ab. `Enter` öffnet die Details.
- **Inline bearbeiten:** Titel von Karten, Spalten und Boards per Doppelklick oder `F2` direkt umbenennen.
- **Sofort speichern:** Es gibt keinen Speichern-Button. Jede Änderung wird direkt übernommen.
- **Rückgängig statt Nachfrage:** Löschen fragt nicht nach, sondern zeigt kurz einen Hinweis mit „Rückgängig“ (auch `Mod+Z`). Bestätigungsdialoge nur, wenn viel auf einmal verloren ginge (Board oder Spalte mit Karten löschen).
- **Erledigen:** Karte in die Erledigt-Spalte ziehen, oder per Kontextmenü bzw. Tastenkürzel dorthin schicken.

### Tastenkürzel

| Kürzel | Aktion |
|---|---|
| `Mod+N` | Neue Karte in der fokussierten Spalte |
| global (einstellbar) | Schnelleingabe öffnen |
| `Mod+F` | Suche |
| `Enter` | Kartendetails öffnen |
| `F2` | Umbenennen |
| `Entf` | Löschen (mit Rückgängig) |
| `Mod+Z` | Rückgängig |
| `Esc` | Panel bzw. Eingabe schließen |

`Mod` ist `Strg` unter Windows und Linux, `Cmd` unter macOS.

## Priorität und Tags

- **Priorität** wird als kleine Markierung an der Karte gezeigt, mit Symbol **und** Farbe (nie nur Farbe). „Keine“ zeigt nichts an.
- **Tags** sind kleine, abgerundete Etiketten. Ihre Farben stammen aus einer festen Palette von etwa acht Tönen, die in beiden Themes ausreichend Kontrast haben. Jeder Ton ist ein eigener Token.

## Design-Tokens

Alle visuellen Grundwerte sind **CSS-Variablen** in `src/styles/tokens.css`. Komponenten verwenden ausschließlich diese Variablen, nie feste Werte.

| Gruppe | Beispiele |
|---|---|
| Farben | Hintergrund (App, Fläche, Spalte, Karte), Text (primär, sekundär, deaktiviert), Rahmen, Akzent, Prioritäten, Tag-Palette |
| Abstände | feste Skala, z. B. 2, 4, 8, 12, 16, 24, 32 px |
| Radien | klein, mittel, groß |
| Schatten | Karte, angehobene Karte beim Ziehen, Panel |
| Schrift | Größen, Zeilenhöhen, Gewichte |
| Maße | Spaltenbreite, Seitenleiste, Panelbreite |
| Bewegung | Dauer und Kurven von Animationen |

- Bedeutungen statt Farbnamen: `--color-text-secondary` statt `--gray-600`.
- **Akzentfarbe:** noch offen. Sie ist ein einzelner Token und lässt sich jederzeit austauschen.

## Themes

- **Hell und Dunkel** gehören zum MVP. Ein Theme ist ein Satz derselben semantischen Variablen, Komponenten wissen nichts vom Theme.
- Standard folgt der Systemeinstellung (`prefers-color-scheme`), manuell überschreibbar (System, Hell, Dunkel).
- Keine Grafiken mit eingebackenen Farben. Icons als SVG mit `currentColor`.

## Schrift

- **Systemschrift** des jeweiligen Betriebssystems (`system-ui`), keine mitgelieferten Schriften.
- Schriftgrößen in `rem`, damit die Systemeinstellung respektiert wird.

## Komponenten

- Eigene, schlanke Komponenten auf Basis von **React Aria Components**, die nur Verhalten und Barrierefreiheit liefern, kein Aussehen. Drag & Drop über `@dnd-kit`.
- **Keine** Bibliothek mit festem Look (Material UI, Ant Design o. Ä.).
- Styling über **CSS Modules** neben der Komponente, ausschließlich mit Variablen aus `tokens.css`.
- Gemeinsame Bausteine (Button, Menü, Dialog, Eingabefeld, Tag, Checkbox) liegen an einer Stelle und werden wiederverwendet.

## Barrierefreiheit

Feste Regeln, keine spätere Verbesserung:

- Kontrast mindestens **WCAG AA** in beiden Themes: 4,5 : 1 für Text, 3 : 1 für große Schrift und Bedienelemente
- Sichtbarer **Tastaturfokus** an jedem bedienbaren Element
- Vollständige Bedienung per Tastatur, inklusive Drag & Drop
- Screenreader-Ansagen beim Verschieben von Karten („Karte X nach In Arbeit, Position 2“)
- Beschriftungen an Icon-Schaltflächen
- Information nie nur über Farbe vermitteln
- Animationen respektieren `prefers-reduced-motion`

## Plattform

- Die Oberfläche sieht auf allen drei Systemen gleich aus, passt sich im Verhalten an (Tastenkürzel, Menüleiste, Fenstersteuerung).
- Neue CSS-Funktionen erst nutzen, wenn WebView2, WKWebView und WebKitGTK sie unterstützen.
- Logische CSS-Eigenschaften (`margin-inline-start` statt `margin-left`) verwenden.

## Noch offen

- Akzentfarbe und Gesamtstimmung der Farbpalette
- Icon-Set (Vorschlag: Lucide)
- Logo und App-Icon
