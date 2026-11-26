# 002-005. Architektura współedycji w czasie rzeczywistym i historii Loro CRDT

Data: 2026-11-21
Status: Zaakceptowano (wybrano przez właściciela; etap powiązany z `003-content`)
Nadrzędny: [002-data](../002-data.md)

## Kontekst

Zgodnie z [002-003](003-crdt-report.md) właściciel wybrał **Loro (`loro` `1.18.0`)** jako główny silnik CRDT do jednoczesnej współpracy i bezkonfliktowego scalania offline na Webie, Desktopie i Mobile.

## Decyzja: Prymitywy Loro i integracja ze stosem

### 1. Mapowanie na obiekty ExoNovel
- **`LoroTree` (Drzewo ruchome)**: Reprezentuje strukturę **`Książka -> Części -> Rozdziały -> Sceny`** oraz foldery **Biblii Fabuły** bez duplikacji węzłów przy równoległych zmianach.
- **`LoroText` (Fugue + Peritext Rich Text)**: Proza rozdziałów, dialogi i opisy wraz ze znacznikami formatowania, linkami i komentarzami.
- **`LoroMap`, `LoroList`, `LoroMovableList`**: Cechy postaci, osie czasu, znaczniki POV/lokacji i wątki komentarzy z walidacją `loro 1.18.0`.
- **Graf DAG i podróże w czasie (`Frontiers` / `VersionVector` / `checkout`)**: Historia szkiców, autorstwo fragmentów i rozgałęzianie (forking).
- **`EphemeralStore`**: Bieżące kursory, zaznaczenia i wskaźniki pisania bez powiększania trwałej historii dokumentu.

### 2. Podwójna trwałość i transport (`Neon` oraz `willow25` + `iroh`)
- **W Neon PostgreSQL**: `crdt_updates` (przyrostowe delty binarne) i `crdt_snapshots` (kompaktowe migawki).
- **Transport na żywo**: Klienci natywni przez **`iroh-gossip` (`v0.102.0`)**, przeglądarki przez **`wss://`** mostkowane do tego samego kanału `iroh-gossip`.
- **Spójność Rust+WASM**: Zarówno `crates/server`, jak i `crates/frontend` używają tej samej wersji **`loro 1.18.0`**.
