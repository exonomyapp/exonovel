# 002-003. Raport decyzyjny: Globalne silniki współedycji w czasie rzeczywistym (CRDT vs OT)

Data: 2026-11-21
Status: Zaakceptowano (Opcja A — `loro 1.18.0` wybrana przez właściciela; zobacz [005-loro](005-loro.md))
Nadrzędny: [002-data](../002-data.md)

## Cel raportu

ExoNovel wymaga jednoczesnej współedycji wielu autorów w stylu Google Docs w przeglądarce (WASM/Leptos), na desktopie (Linux, macOS, Windows) oraz urządzeniach mobilnych (Android, iOS), wraz z edycją offline spójną z Rustem i **protokołem Willow (`willow25`)**.

---

## Ocenieni kandydaci Open Source

### 1. Loro (`loro` — Globalne Open Source z Chin, Rust + WASM)
- **Pochodzenie i wersja**: Zespół Loro (licencja MIT). Zweryfikowana biblioteka Rust: **`1.18.0`**.
- **Działanie**: Natywny rdzeń Rust kompilowany do WASM oraz binariów natywnych; algorytm rich-text **Fugue** plus przyczynowy graf historii (DAG).
- **Zalety dla ExoNovel**:
  - **Natywne drzewa ruchome (Movable Trees)**: Równoczesne przenoszenie rozdziałów między częściami i porządkowanie scen/Biblii Fabuły bez duplikatów.
  - **Wbudowana historia i cofanie zmian**: Podgląd migawek historycznych, cofanie per autor i gałęzie/scalanie.
  - **Rich Text (Peritext)** i **kompaktowe delty binarne** dopasowane do **`iroh-blobs` / `willow25`**.
- **Kompromisy**: Większy moduł WASM (~150–300 kB gzipped vs ~18 kB w Yjs).

### 2. Yrs / Yjs (`yrs` — Open Source z Niemiec, Rust + JS/WASM)
- **Wersja**: **`0.28.1`**, najbogatszy ekosystem edytorów (TipTap, ProseMirror, CodeMirror), brak natywnego CRDT drzew ruchomych.

### 3. Automerge (`automerge` — Wielka Brytania / Ink & Switch, Rust + WASM)
- **Wersja**: **`0.12.0`**, niezmienniczy graf zmian w stylu Git, wyższe zużycie pamięci w dużych dokumentach.

### 4. Diamond Types (`diamond-types`) & 5. Cola (`cola`)
- Rekordowa wydajność dla czystego tekstu bez znaczników formatowania i drzew.

### 6. Centralny Operational Transform (`ShareDB`)
- Wymaga centralnego serwera koordynującego i utrudnia pracę offline w `iroh`.

---

## Opcje decyzyjne

- **Opcja A — Loro (`1.18.0`)**: Rekomendowana dla **reorganizacji drzew rozdziałów, rich text, podróży w czasie oraz jednolitego kodu Rust+WASM**.
- **Opcja B — Yrs (`0.28.1`)**: Najmniejszy rozmiar WASM i gotowe wtyczki edytorów.
- **Opcja C — Automerge (`0.12.0`)**: Pełna historia commitów każdego naciśnięcia klawisza.
- **Opcja D — Hybryda**: `Loro`/`Yrs` dla powieści oraz czysty Markdown/Fountain dla scenariuszy.
