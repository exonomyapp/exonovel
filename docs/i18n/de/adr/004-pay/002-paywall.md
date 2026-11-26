# 004-002. Autoren-Paywall-Steuerung & Berechtigungen

Datum: 2026-11-19
Status: Vorgeschlagen (für Interview)
Übergeordnet: [004-pay](../004-pay.md)

## Entscheidungsstruktur (Offen für Interview)

- **Granularität**: Autoren können ganze veröffentlichte Werke, einzelne Kapitel/Episoden oder kuratierte Story-Architektur-Pakete hinter eine Paywall stellen.
- **Berechtigungsspeicherung**: Zahlungs-Webhooks schreiben einen Eintrag `(buyer_did, resource_id, grant_type, expires_at)` in Neon PostgreSQL.
- **Plattformübergreifender Zugriff**: Da Berechtigungen an die `did:peer` gebunden sind, schaltet ein Web-Kauf automatisch Linux, macOS, Windows, Android und iOS frei.

## Offene Interviewfragen

- Welche Paywall-Modelle sollen unterstützt werden (Einmalkauf, Autoren-Abo, Pro-Kapitel, Trinkgelder)?
- Sollen kostenlose Leseproben (z. B. erste N Kapitel oder öffentliche Figurenprofile) je Projekt konfigurierbar sein?
