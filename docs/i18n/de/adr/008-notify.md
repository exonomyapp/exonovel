# 008. Plattformübergreifendes Benachrichtigungssystem (5 Zielplattformen, Web & Brevo-E-Mail)

Datum: 2026-11-21
Status: Akzeptiert (Brevo aktiv; GitHub-Stil-Aufteilung Profil vs. Präferenzen & Android FCM+UnifiedPush bestätigt)

## Kontext

ExoNovel läuft im **Web** sowie nativ auf **Linux, macOS, Windows, Android und iOS**. Benachrichtigungen umfassen Zusammenarbeit, **Autoren-Paywall-Einnahmen**, Sicherheit/Abrechnung sowie **Reifegrad-Prozess**-Meldungen bei neuen Bibliotheks-Releases.

## Entscheidung

### 1. Multi-Plattform-Zustellarchitektur
Ereignisse aus `notifications`, `notification_devices` und `notification_preferences` verteilen sich gleichzeitig über:
- **In-App-Live-Feed**: Sofortige Zustellung über **Axum Server-Sent Events (SSE)** bzw. lokale `iroh`-Ereignisse im Modus 2.
- **Web Push API**: Signiert mit Rust `web-push` (`0.11.0`).
- **Desktop (`Linux`, `macOS`, `Windows`)**: Vordergrund über `tauri-plugin-notification`; Hintergrund über **UnifiedPush + D-Bus** (Linux), **APNs** (`a2`, macOS) und **WNS** (Windows).
- **Mobil (`Android`, `iOS`)**: **Android** nutzt **FCM HTTP v1** + **UnifiedPush**, beginnend mit der reinen **F-Droid-`foss`-Variante** (nur UnifiedPush, ohne proprietäre Google-SDKs); **iOS** nutzt **APNs** (`a2`).
- **Transaktionale E-Mails (Brevo API v3 auf `exosystems.net`)**:
  - `notes@exosystems.net` (`"Notes from ExoNovel"`) — Benachrichtigungen, Zusammenfassungen, Reifegrad-Hinweise.
  - `noreply@exosystems.net` (`"Exosystems"`) — Authentifizierungs- und allgemeine System-Mails.
  - `support@exosystems.net` (`"ExoSupport"`) — Support-Anfragen und Antworten.

### 2. Strikte Trennung Profil vs. Präferenzen (GitHub-Stil)
- **Profil**: Enthält **100 % aller kommerziellen, Wallet-, Paywall- und Kauf-Funktionen** sowie soziale und Umsatz-Benachrichtigungen (`notify_author_earnings`, `notify_reader_purchases`, Einladungen, Empfehlungen, Kommentare, Erwähnungen/Forks).
- **Präferenzen**: Enthält **technische Präferenzen, Sprachen, Sync-Modus, Benachrichtigungskanäle & Admin-Reifegrad** (`notify_security_signins`, `notify_maturity_releases`, `notify_maturity_unblocked`), niemals Zahlungs- oder Wallet-Verwaltung.

### 3. Neon + Willow Speicherparität
Alle Benachrichtigungstabellen führen identische `willow_*`-Koordinaten für offlinefähige native Installationen.
