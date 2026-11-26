# ExoNovel — Produktumfang

> **Status**: Entwurf aus dem ersten Nutzerbriefing und der angeforderten Publikationsformat-Recherche. Offen für weitere Interview-Verfeinerungen.

---

## 1. Produktvision & Kernkonzept

**ExoNovel** ist eine kollaborative Plattform und Anwendung für kreatives Schreiben, bei der **jede beitragende Person das Eigentum an ihren eigenen Inhalten behält**, selbst wenn gemeinsam in geteilten Projekten geschrieben wird.

### Grundpfeiler

- **Individuelles Inhaltseigentum in Gemeinschaftsprojekten**: Autorinnen und Autoren besitzen die kreativen Elemente und Beiträge, die sie selbst verfassen, auch innerhalb eines gemeinsamen Projekts.
- **Modulares / dimensionales Teilen**: Die Architektur fördert das unabhängige Freigeben einzelner Projektdimensionen—etwa das Veröffentlichen oder Teilen einer einzelnen Charakterbiografie, sodass andere Schreibende sie entdecken, aufgreifen und nach Freigabe in eigene Romane übernehmen können.
- **Kollaboration im Stil von Google Docs & Live-Co-Editing**: Standardisierte Freigabe- und Berechtigungsfunktionen kombiniert mit gleichzeitigem Live-Co-Editing.
- **Autorenzentrierte Planung & Erstellung**: Benutzeroberfläche und Arbeitsabläufe sind darauf ausgelegt, wie Schreibende Werke (Romane, Drehbücher und andere Formate) tatsächlich planen, strukturieren und verfassen, statt generische Texteditoren aufzuzwingen.

---

## 2. Plattform- & Technik-Umfang

### Zielplattformen

1. **Website (Webanwendung)** — Primäre Erstveröffentlichung.
2. **Desktop-Installationen** (alle drei großen Desktop-Betriebssysteme):
   - **Linux**
   - **macOS**
   - **Windows**
3. **Mobile-Installationen** (beide großen Mobil-Betriebssysteme):
   - **Android**
   - **iOS**

### Datenspeicherung

- **Datenbank**: Gehostete **PostgreSQL**-Lösung (`pgsql`).

---

## 3. Top-5-Publikationsformate & Natürliche Strukturen

Basierend auf Branchenreichweite, Aktivität der Schreib-Community und Publikationsvolumen werden die fünf grundlegenden Publikationsformate für ExoNovel—samt ihrer natürlichen **Planungs-/Autorenstruktur** und **Veröffentlichungsstruktur**—wie folgt definiert.

### Format 1: Romane (Langform Belletristik & Sachbuch)

Umfangreiche narrative Werke, die das kommerzielle Rückgrat klassischer, unabhängiger und eigenveröffentlichter Publikationen bilden.

#### Natürliche Planungs- & Schreibstruktur

- **Fundamentale Story-Bibel**:
  - **Charaktere**: Profile, Biografien, Handlungsbögen, Motivationen, Schwächen, Verwundungen und Beziehungsgeflechte.
  - **Worldbuilding / Setting-Bibel**: Orte, Zeitachsen, Lore, Regeln/Systeme (beim Entwerfen weitgehend als „Eisberg“ im Hintergrund gehalten).
  - **Plot / Gliederung**: Übergreifender Fahrplan (z. B. Drei-Akt-Struktur, Beat-Sheets, Schneeflocken-Methode oder Stichpunkt-Gliederungen) mit Auslöser, Wendepunkt, Höhepunkt und Auflösung.
- **Entwurfshierarchie**:
  - **Teile / Akte** (große Wechsel von Zeit, Perspektive oder Erzählphase)
  - **Kapitel** (Takteinheiten von typischerweise 1.500–5.000 Wörtern, endend auf Cliffhangern oder Übergängen)
  - **Szenen & Sequels** (atomare Aktions-/Konflikteinheiten gepaart mit Reaktions-/Verarbeitungseinheiten der Figuren)

#### Veröffentlichte Ausgabestruktur

- Titelei (Titelseite, Widmung, Epigraph, Inhaltsverzeichnis bei Bedarf)
- Sequenzierte **Teile** (optional) und **Kapitel** mit geglätteten Übergängen und Szenentrennern
- Anhang (Danksagung, Autorenbiografie, Reihenhinweise)

---

### Format 2: Artikel & Kreative Sachliteratur (Essays, Memoiren, Reportagen)

Reichweitenstarke Sachliteratur von narrativem Journalismus über persönliche Essays und Kulturkommentare bis zu Memoiren.

#### Natürliche Planungs- & Schreibstruktur

- **Kernperspektive & Ziel**: Zentrale These (Essays), Nachrichten-/Nutzwert-Aufhänger (Artikel) oder zugrundeliegende thematische Wahrheit (kreative Sachliteratur/Memoiren).
- **Strukturierter Aufbau**:
  1. **Einstieg / Lead**: Sensorischer Szeneneinstieg, überraschende Kennzahl/These oder kompakter Aufmacher.
  2. **Überleitung / Kontext**: Übergang vom Einstieg zum Kernthema.
  3. **Hauptabschnitte**: Argumente, Belege, Anekdoten oder narrative Szenen unter klaren Zwischenüberschriften (`H2` / `H3`), wahlweise als umgekehrte Pyramide (Journalismus), logische Argumentation (Essay) oder verschränkte Szenen-und-Reflexions-Bögen.
  4. **Wendepunkt / Synthese**: Entkräftung von Gegenargumenten oder zentrale intellektuelle/emotionale Erkenntnis.
  5. **Schluss**: Synthese, Kernaussage oder nachhallendes Schlussbild.
- **Recherche- & Quellennotizen**: Zitate, Interviews, Faktenprüfungen und Belegauszüge direkt an den Abschnitten.

#### Veröffentlichte Ausgabestruktur

- Überschrift & Untertitel / Teaser
- Autorenzeile & Metadaten
- Einleitungsabsatz
- Gegliederte Hauptabschnitte (mit Zitathervorhebungen, Abbildungen oder Quellenangaben)
- Abschließendes Fazit / Handlungsaufforderung

---

### Format 3: Kurzgeschichten & Flash Fiction

Kompakte Erzählprosa von Flash Fiction (typischerweise unter 1.000 Wörtern) bis zur klassischen Kurzgeschichte (ca. 3.000–8.000 Wörter) für Literaturmagazine, Anthologien und digitale Newsletter.

#### Natürliche Planungs- & Schreibstruktur

- **Einzelkonflikt-Anker**: Eine zentrale Spannung, komprimierte Zeitachse und enge Figurenauswahl (Vermeidung ausufernder Nebenstränge).
- **Fokussierte Gliederung**:
  1. ***In Media Res*-Einstieg & Verortung**: Unmittelbare Figur, Umgebung und akuter Druck.
  2. **Auslösender Druck & angedeutete Vorgeschichte**: Hintergrund wird über konkrete Details statt Exposition vermittelt („Eisberg-Technik“).
  3. **Eskalation / Mikro-Wendepunkt**: Wachsende Reibung bis zur Verschiebung der Dynamik.
  4. **Höhepunkt / Brennpunkt**: Entscheidende Wahl oder Erkenntnis.
  5. **Nachhall**: Bleibendes Bild oder innerer Wandel statt übererklärendem Abschluss.
- **Sammlungs- / Anthologie-Gruppierung** (optional): Bündelung verwandter Geschichten unter einem gemeinsamen Thema oder Universum.

#### Veröffentlichte Ausgabestruktur

- Titel & Autorenzeile
- Verdichtete, durchgehende Prosa (einzelne Szene oder minimale Abschnittstrenner `* * *`)
- Einzelveröffentlichung (Magazine/Zines) oder sequenziert in einer **Anthologie / Kurzgeschichtensammlung**

---

### Format 4: Drehbücher & Skripte (Film, Fernsehen, Audio/Podcast)

Visuelle und auditive Baupläne nach strenger Branchenformatierung, bei der grob **1 Skriptseite 1 Minute Spielzeit** entspricht.

#### Natürliche Planungs- & Schreibstruktur

- **Entwicklungsdokumente vor dem Skript**:
  - **Logline & Prämisse**: Kernaufhänger in ein bis zwei Sätzen.
  - **Treatment**: Prosafassung im Präsens (2–15+ Seiten), die die Geschichte komplett ohne ausgeschriebene Dialoge erzählt.
  - **Beat-Sheet & Szenenkarten**: Szenenweise Karten über Akte hinweg (Akt I Setup, Akt II Konfrontation/Midpoint, Akt III Auflösung) oder episodische A/B/C-Handlungsstränge.
  - **Figuren- & Schauplatzübersichten**: Sprechrollen, Stimmmerkmale, wiederkehrende Sets/Orte.
- **Skript-Blockelemente**:
  - **Szenenüberschriften (Sluglines)**: `[INT./EXT.] [SCHAUPLATZ] - [TAGESZEIT]`
  - **Aktionszeilen**: Präsens, knappe visuelle/akustische Beschreibungen
  - **Figurennamen** (inkl. `(V.O.)`, `(O.S.)`)
  - **Dialoge** & **Regieanweisungen (Parentheticals)**
  - **Übergänge** (sparsam eingesetzt)

#### Veröffentlichte / Produktionsreife Ausgabe

- Titelseite (Titel, Verfasser, Entwurfsdatum / Kontakt)
- Branchenübliches Festbreiten-Layout (12pt Courier / Courier Prime, 1,5" linker Rand, 1" oben/unten/rechts)
- Nummerierte oder unnummerierte Szenenfolge für Leseproben, Pitch-Unterlagen oder Drehplanung

---

### Format 5: Lyrik (Einzelgedichte, Chapbooks & Gedichtbände)

Vers- und strophengetriebene Dichtung von gebundenen Formen (Sonette, Villanellen, Haikus) über freie Verse bis zu Spoken-Word- und Digitalformaten, einzeln veröffentlicht oder als Chapbook bzw. Gesamtband kuratiert.

#### Natürliche Planungs- & Schreibstruktur

- **Vers- & Strophenhandwerk**:
  - **Verse & Zeilenumbrüche**: Steuerung von Atem, Tempo, *Enjambement* und zeilenendender Betonung.
  - **Strophen**: Strukturelle Gruppierungen (Zweizeiler, Terzette, Quartette usw.) und bewusster Weißraum.
  - **Klang- & Formnotizen**: Metrum (z. B. jambischer Pentameter), Reimschemata oder freie rhythmische Kadenz; thematischer Funke -> Umschlag (*Volta*) -> Nachklang.
- **Manuskript- / Bandkuratierung**:
  - Sammlung von Gedichten und Sequenzierung nach thematischem, emotionalem oder erzählerischem Bogen.
  - Ausbalancieren von Satzdichte und Weißraum über Seiten hinweg.
  - Umfangsstufen: **Mikro-Chapbook** (~5–10 Gedichte), **Chapbook** (~15–30 Gedichte) oder **Gedichtband** (~40–80+ Gedichte).

#### Veröffentlichte Ausgabestruktur

- **Einzelgedicht**: Titel, Widmung/Motto (optional), exakt erhaltener Zeilenfall, Einrückung und Strophenabstand.
- **Chapbook / Gedichtband**: Titelseite, Inhaltsverzeichnis, optionale nummerierte/betitelte Abschnitte, Beginn jedes Gedichts auf neuer Seite sowie Anmerkungen/Danksagung.

---

## 4. Offene Interviewfragen (Nächste Schritte)

- Sollen einzelne dieser **fünf** Publikationsformate angepasst, ersetzt oder neu geordnet werden?
- Zu welchem der fünf Formate soll das **Detail-Interview** zuerst stattfinden (Planungsdimensionen, wiederverwendbare Komponenten und kollaborative Eigentumsregeln)?
