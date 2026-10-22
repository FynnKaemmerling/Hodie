# Design

Dieses Dokument legt die Designrichtung von Hodie fest. Es beschreibt Prinzipien und Grundentscheidungen, kein fertiges Aussehen.

## Leitidee

*(Folgt.)*

## Grundregeln

- Alle visuellen Werte (Farben, Abstände, Schriftgrößen, Radien, Animationen) als CSS-Variablen in `src/styles/tokens.css`. Komponenten verwenden ausschließlich diese Variablen.
- Keine UI-Bibliothek mit festem Look.
- Systemschrift des jeweiligen Betriebssystems.
- Barrierefreiheit: mindestens WCAG AA, sichtbarer Tastaturfokus, vollständige Tastaturbedienung.
