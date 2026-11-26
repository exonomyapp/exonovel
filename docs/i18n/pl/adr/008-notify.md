# 008. Wieloplatformowy system powiadomień (5 platform docelowych, Web i Brevo E-mail)

Data: 2026-11-21
Status: Zaakceptowano (Brevo aktywne; podział Profil vs Preferencje w stylu GitHub i Android FCM+UnifiedPush potwierdzone)

## Kontekst

ExoNovel działa w **przeglądarce** oraz na **5 platformach natywnych: Linux, macOS, Windows, Android i iOS**. Powiadomienia obejmują współpracę, **przychody z paywalli autorskich**, bezpieczeństwo/rozliczenia oraz aktualizacje **Procesu Dojrzałości**.

## Decyzja

### 1. Architektura wielokanałowego dostarczania
Zdarzenia z `notifications`, `notification_devices` i `notification_preferences` trafiają równolegle przez:
- **Kanał na żywo w aplikacji**: **Axum Server-Sent Events (SSE)** lub lokalne zdarzenia `iroh` w Trybie 2.
- **Web Push API**: Podpisywane przez bibliotekę Rust `web-push` (`0.11.0`).
- **Desktop (`Linux`, `macOS`, `Windows`)**: Pierwszy plan przez `tauri-plugin-notification`; tło przez **UnifiedPush + D-Bus** (Linux), **APNs** (`a2`, macOS) i **WNS** (Windows).
- **Mobile (`Android`, `iOS`)**: **Android** obsługuje **FCM HTTP v1** + **UnifiedPush**, rozpoczynając od czystej wersji **F-Droid `foss`** (wyłącznie UnifiedPush); **iOS** używa **APNs** (`a2`).
- **Poczta transakcyjna (Brevo API v3 na `exosystems.net`)**:
  - `notes@exosystems.net` (`"Notes from ExoNovel"`) — powiadomienia, podsumowania i komunikaty dojrzałości.
  - `noreply@exosystems.net` (`"Exosystems"`) — wiadomości logowania i systemowe.
  - `support@exosystems.net` (`"ExoSupport"`) — obsługa zgłoszeń wsparcia.

### 2. Ścisły podział Profil vs Preferencje (Styl GitHub)
- **Profil**: Zawiera **100% wszystkich funkcji finansowych, portfeli, paywalli i zakupów** oraz powiadomienia o zarobkach i współpracy (`notify_author_earnings`, `notify_reader_purchases`, zaproszenia, polecenia, komentarze, wzmianki/forki).
- **Preferencje**: Zawierają **preferencje techniczne, języki, tryb synchronizacji, kanały powiadomień i panel admina** (`notify_security_signins`, `notify_maturity_releases`, `notify_maturity_unblocked`), bez żadnych opcji finansowych.

### 3. Parytet zapisu Neon + Willow
Wszystkie tabele powiadomień zachowują współrzędne `willow_*` dla pracy lokalnej bez dostępu do sieci.
