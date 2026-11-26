# 003. Inhaltsmodelle & Teilen von Story-Architektur

Datum: 2026-11-19
Status: Vorgeschlagen (offen für Detail-Interviews)

## Kontext

Schreibende strukturieren Entwürfe je nach Publikationsformat unterschiedlich; gleichzeitig basiert ExoNovel auf modularen Story-Bausteinen, die ohne Verlust der Urheberschaft freigegeben, gemeinsam bearbeitet und projektübergreifend wiederverwendet werden können.

## Übergreifende Architektur

1. **Formatgerechtes Verfassen**: Unterstützung der fünf Publikationsformate (**Romane**, **Artikel & Kreative Sachliteratur**, **Kurzgeschichten & Flash Fiction**, **Drehbücher & Skripte**, **Lyrik**) mit Trennung von **Planungsstruktur** und **veröffentlichter Ausgabestruktur**.
2. **Modulare Story-Architektur**: **Charaktere**, **Plots**, **Zeitachsen / Handlungsstränge** und **Themen** als eigenständige, gezielt teilbare Entitäten im Eigentum einer `did:peer`.
3. **Berechtigtes Forking**: Mit expliziter Freigabe durch die besitzende `did:peer` können andere Schreibende jeden Baustein unter eigener `owner_did` in ihr Projekt **forken**, wobei die Herkunft (`origin_owner_did`) lückenlos erhalten bleibt.

## Unter-ADRs

- [001-formats](003-content/001-formats.md) — Top-5-Publikationsformate & Planungs- vs. Ausgabestruktur
- [002-forking](003-content/002-forking.md) — Story-Architektur-Teile (Charaktere, Plots, Zeitachsen, Themen) & berechtigtes Forking
