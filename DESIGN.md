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
│            │ │ #tag     │ │ │ ☑ 2/5    │ │ │          │ │       │
│ ＋ Board   │ └──────────┘ │ └──────────┘ │ └──────────┘ │       │
│            │ ┌──────────┐ │              │              │       │
│ Archiv     │ │ Titel    │ │              │              │       │
│            │ └──────────┘ │              │              │       │
│ ⚙          │ ＋ Karte     │ ＋ Karte     │ ＋ Karte     │       │
└────────────┴──────────────┴──────────────┴──────────────┴───────┘
```

- **Seitenleiste links:** Boards (per Griff ⠿ sortierbar), Archiv, Knopf „Einstellungen“. Sie lässt sich über das Symbol im Kopf oder `Mod+B` ausblenden; dann holt ein Symbol am Anfang der Werkzeugleiste sie zurück. Auf macOS ist sie durchscheinend (Systemmaterial „Sidebar“), unter Windows und Linux deckend. Die Einstellungen öffnen sich als Dialog mit den Tabs Allgemein (Sprache, Darstellung, Vault-Ordner, Tastenkürzel), Farben, Tags und Info (Version, Speicherverbrauch von Vault und Datenbank). Unter Farben wählt man ein Preset (Hodie, Ozean, Wald oder eigene) und legt eigene an; jedes hat eine helle und eine dunkle Variante aus Akzentfarbe und Grundton. Die Hauptfläche (Board, Karten, Panel) bleibt dabei immer weiß bzw. dunkel. Zu schwache Kontraste werden direkt am Preset angezeigt.
- **Board:** Spalten nebeneinander mit fester Breite, horizontal scrollbar, auf einem feinen Punktraster. Jede Spalte zeigt einen Farbpunkt (Erledigt-Spalte grün) und die Anzahl ihrer Karten; leere Spalten zeigen einen kurzen Hinweis.
- **Karte:** Titel, darunter nur, was vorhanden ist: Tags und Fortschritt der Teilaufgaben (Zahl und dünner Balken). Die Priorität steckt in der Farbe der Kachel und einem abgerundeten Streifen am Anfang, der mit der Priorität breiter wird. Karten in der Erledigt-Spalte treten zurück und zeigen ein Häkchen. Leere Felder nehmen keinen Platz ein.
- **Kartendetails** öffnen sich in einem **Seitenpanel rechts**. Das Board bleibt sichtbar und bedienbar. `Esc` oder ein Klick daneben schließt das Panel; ein Klick auf eine andere Karte wechselt zu ihr.
- **Schlicht:** Aktionen sind reine Icon-Knöpfe ohne Beschriftung (Name als Tooltip und für Screenreader). Die Suche ist ein Lupen-Icon, das sich beim Fokussieren zum Feld öffnet; Filter, „＋ Board“, „＋ Karte“ (im Spaltenkopf), „＋ Spalte“, Archiv und Einstellungen sind Icons.
- **Vault nicht verfügbar:** Lässt sich der Vault-Ordner nicht öffnen (z. B. Laufwerk getrennt), zeigt das Fenster statt Seitenleiste und Board nur Titel, Grund, Pfad und zwei Knöpfe: „Erneut versuchen“ (Akzent) und „Anderen Ordner wählen …“.
- **Schnelleingabe:** kleines, randloses Fenster mit runden Ecken (`--radius-window`), mittig auf dem Bildschirm: Logo, Eingabefeld und runder Senden-Knopf in Akzentfarbe. `Enter` legt die Karte an, `Esc` schließt.

## Interaktion

- **Akzent:** Knöpfe zum Anlegen tragen ein rundes Akzent-Symbol, die geöffnete oder fokussierte Karte leuchtet dezent in der Akzentfarbe. Eine Karte aus der Schnelleingabe pulsiert kurz auf, das Kartenpanel gleitet mit leichtem Nachfedern herein.
- **Drag & Drop** für Karten und Spalten, mit sichtbarer Einfügemarke und weicher Animation der verdrängten Karten.
- **Tastatur gleichwertig:** Pfeiltasten bewegen den Fokus zwischen Karten und Spalten, `Leertaste` hebt eine Karte auf, Pfeiltasten verschieben sie, `Leertaste` legt sie ab, `Esc` bricht ab. `Enter` öffnet die Details.
- **Rechtsklick** auf Board, Spalte oder Karte öffnet an der Maus dieselben Aktionen wie die drei Punkte (Karten: Öffnen, Umbenennen, Archivieren, Löschen). Auch die Kontextmenü-Taste und `Umschalt+F10` funktionieren. Das Browser-Menü (Zurück, Aktualisieren, Untersuchen …) gibt es nicht; nur Textfelder behalten das Systemmenü für Ausschneiden, Kopieren und Einfügen.
- **Inline bearbeiten:** Titel von Karten, Spalten und Boards per Doppelklick oder `F2` direkt umbenennen.
- **Sofort speichern:** Es gibt keinen Speichern-Button. Jede Änderung wird direkt übernommen.
- **Rückgängig statt Nachfrage:** Löschen fragt nicht nach, sondern zeigt kurz einen Hinweis mit „Rückgängig“ (auch `Mod+Z`). Bestätigungsdialoge nur, wenn viel auf einmal verloren ginge (Board oder Spalte mit Karten löschen).
- **Erledigen:** Karte in die Erledigt-Spalte ziehen, oder per Kontextmenü bzw. Tastenkürzel dorthin schicken.

### Tastenkürzel

| Kürzel | Aktion |
|---|---|
| `Mod+N` | Neue Karte in der fokussierten Spalte |
| `Mod+Umschalt+Leertaste` (systemweit, in den Einstellungen änderbar) | Schnelleingabe öffnen |
| `Mod+F` | Suche |
| `Mod+B` | Seitenleiste aus- und einblenden |
| `Enter` | Kartendetails öffnen |
| `F2` | Umbenennen |
| `Entf` | Löschen (mit Rückgängig) |
| `Mod+Z` | Rückgängig |
| `Esc` | Panel bzw. Eingabe schließen |

`Mod` ist `Strg` unter Windows und Linux, `Cmd` unter macOS.

## Priorität und Tags

- **Priorität** färbt die ganze Kachel leicht ein (niedrig blau, mittel gelb, hoch rot) und gibt ihr links einen farbigen Rand, der mit der Priorität breiter wird. So ist sie auch ohne Farbwahrnehmung unterscheidbar, Screenreader lesen sie als Text vor. „Keine“ zeigt nichts an.
- Notizen werden auf der Karte nicht angezeigt, nur im Panel.
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
- **Akzentfarbe:** warmes Orange (`#c0501c` hell, `#f0a06a` dunkel), bewusst röter als das Ocker der mittleren Priorität. Sie ist ein einzelner Token und lässt sich jederzeit austauschen.

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
- **Titelleiste:** Unter Windows und Linux ist das Hauptfenster rahmenlos. Minimieren, Maximieren und Schließen sitzen rechts in der Kopfzeile (nach „Filter“), das Logo steht links neben „Hodie“. Freie Flächen der Kopfzeilen und der Seitenleistenkopf verschieben das Fenster, Doppelklick maximiert. Unter macOS bleiben die Ampelknöpfe des Systems links, die Titelleiste geht ins App-Design über (`tauri.macos.conf.json`).
- Öffnet sich eine Karte, bekommt das Panel den Fokus, nicht das Titelfeld. Bearbeiten beginnt erst mit einem Klick oder Tab.
- Neue CSS-Funktionen erst nutzen, wenn WebView2, WKWebView und WebKitGTK sie unterstützen.
- Logische CSS-Eigenschaften (`margin-inline-start` statt `margin-left`) verwenden.

## Icons und App-Icon

- **Icons:** Lucide, als Linien-Icons in Textfarbe. Größe und Strichstärke kommen aus Tokens (`--icon-size-*`, `--icon-stroke`), nicht aus Props. Icons neben Text sind `aria-hidden`, reine Icon-Buttons haben eine Beschriftung.
- **App-Icon:** Häkchen vor einer aufgehenden Sonne („heute erledigt“) auf orangem Verlauf. Quelle: `src-tauri/icons/app-icon.svg`.
