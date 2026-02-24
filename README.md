# Dashboarder #
Event-driven microservices architektura pro domácí dashboard, přepsaná z Go do Rustu. Systém sbírá data z různých zdrojů (API, web scraping, externí MQTT), normalizuje je a ukládá do vyměnitelných databází (PostgreSQL, SQLite, ValkeyDB). Data jsou následně prezentována přes REST API a minimalistické webové UI.

### Architektura a Design ###
Celý systém je postaven na asynchronním běhovém prostředí Tokio a využívá centrální MQTT broker (Mosquitto) pro komunikaci mezi nezávislými mikroslužbami. Tento přístup zajišťuje vysokou odolnost proti selhání a minimální paměťovou náročnost, což je ideální pro běh na Raspberry Pi a nasazení v K3s clusteru.

## Klíčové komponenty

### Fetchers:### 
Nezávislé procesy, které v daných intervalech stahují surová data (OpenWeather, scraping, ext-MQTT) a publikují je do raw/# MQTT topiků.

### Parsers:### 
Služby poslouchající na raw/# topicích. Deserializují JSON, validují data, transformují je a publikují do normalized/# topiků.

### Storagers:### 
Moduly přijímající normalizovaná data. Pomocí definovaných rozhraní (Traits) je ukládají do příslušných databází (PostgreSQL s TimescaleDB, ValkeyDB, SQLite).

### API Gateway:### 
REST API poskytující CRUD přístup k uloženým datům.

### Web UI:####
Odlehčený webový server servírující statické HTML s minimem JavaScriptu, zobrazující dlaždice na dashboardu.

#### Controller:#### 
Služba monitorující stav (Health check) všech ostatních komponent pomocí zachytávání "heartbeat" MQTT zpráv.

## Implementované návrhové vzory (Design Patterns)
**Actor Pattern:** Každá mikroslužba (nebo její část) funguje jako nezávislý aktor. Využíváme MPSC (Multi-Producer, Single-Consumer) kanály v Tokiu pro předávání zpráv uvnitř služby bez nutnosti sdílení stavu a zamykání paměti (Mutex).
**Publish-Subscribe (Pub/Sub):** Veškerá meziprocesová komunikace je řešena přes MQTT broker. Služby neznají jedna druhou, pouze produkují a konzumují události na definovaných topicích.
**Strategy Pattern:** Pro ukládání dat využíváme Rust Traits (ekvivalent Go interfaces). Umožňuje to snadnou a transparentní výměnu databázového backendu bez zásahu do aplikační logiky.
**Heartbeat / Health Monitoring:** Služby periodicky publikují svůj stavový JSON na specifický topic. Controller tyto zprávy agreguje pro zobrazení dostupnosti celého systému.
**Fail-Fast & Error Handling:** Silný typový systém Rustu (Result, Option) a knihovna serde zaručují, že špatně naformátovaná data z externích zdrojů nezpůsobí pád služby, ale jsou bezpečně zalogována a zahozena.

## Struktura projektu (Cargo Workspace)
Projekt je rozdělen do tzv. Cargo Workspace, což umožňuje sdílet závislosti (např. verze knihoven) a společné doménové modely, ale kompilovat aplikaci do oddělených binárních souborů.

```bash
dashboarder-rs/
├── Cargo.toml               (Hlavní definice workspace)
├── libs/
│   ├── common_models/       (Sdílené datové struktury a MQTT typy)
│   └── storage_traits/      (Rozhraní pro databáze)
├── fetchers/                (Služby pro sběr dat)
├── parsers/                 (Zpracování a normalizace dat)
├── storagers/               (Implementace ukládání do DB)
├── api/                     (Axum REST API)
├── web/                     (Axum + Askama web UI)
└── controller/              (Monitoring stavu služeb)
```

## Technologický Stack (Rust Crates)
**tokio:** Asynchronní běhové prostředí (runtime).
**rumqttc:** Rychlý a čistě v Rustu napsaný MQTT klient.
**serde & serde_json:** Serializace a deserializace dat (z/do JSON).
**reqwest:** Asynchronní HTTP klient pro dotazování externích API.
**scraper:** Parsování HTML a CSS selektorů pro web scraping.
**sqlx:** Asynchronní, typově bezpečné dotazování do PostgreSQL/SQLite.
**axum:** Extrémně rychlý a ergonomický webový framework pro API a Web.
**askama:** Typově bezpečný šablonovací systém pro HTML (kompilovaný do binárky).

## Lokální spuštění
Zajistěte, že máte spuštěný lokální MQTT broker (Mosquitto) na portu 1883.

## Sestavení všech služeb ve workspace
cargo build

## Spuštění konkrétní služby (např. weather-fetcher)
cargo run --bin weather-fetcher
