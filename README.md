# CommandHub

CommandHub wordt een configuratiegestuurde MCP-server in Rust die externe commandline-tools via HTTP toegankelijk maakt. De server moet parameters controleren, processen begrenzen en resultaten teruggeven zonder maatwerk per tool.

## Huidige status

Actief is [F05 — MCP-integratie](docs/features/F05-mcp-integratie.md). Streamable HTTP op `/mcp` deelt registry, inputschemaopbouw en uitvoering met REST. Een echte SDK-client ontdekt en voert een geconfigureerde testtool via TCP uit. Foutmapping, Host/Origin, bodygrenzen en streamannulering zijn gericht getest op Windows.

F05 staat op Review. Protocolannulering, productie-shutdown met een MCP-procesboom, gedeelde REST/MCP-capaciteit, dynamische stringconsistentie en negatieve transportrequests zijn op Windows getest. De SDK-client onderhandelt `2025-11-25`. De eigenaar bevestigt formatter, Clippy en 156 geslaagde tests met 7 proceshelpers. De uitgebreide ketentest vergelijkt ook geldige en afgewezen paden tussen REST en MCP. De laatste volledige Windows-suite slaagde in 9,77 s. F05-platform-CI en PR-review/merge staan nog open. Zie [voortgang](docs/voortgang.md) en de [F05-notitie](notities/F05-mcp-integratie.md).

De vorige F04-stand heeft geslaagde lokale controles en drieplatform-CI. Dit geldt niet als bewijs voor F05; review/merge is tijdens deze werksessie niet afzonderlijk bevestigd. De voorbeeldtools staan in [config/tools.json](config/tools.json); zie de [F04-testhandleiding](docs/F04-handmatig-testen.md).

## Installeren en gebruiken

Zie de instructies hieronder voor installatie en Cargo-commando's. De featurevolgorde en branchwerkwijze staan in [de roadmap](docs/roadmap.md).

Benodigd: recente stabiele Rust-toolchain met ondersteuning voor editie 2024, Cargo en een platformgeschikte linker. Gebruik op Windows de MSVC-buildtools en op macOS de Xcode Command Line Tools. Git is ook nodig voor de Git-voorbeeldtool en de bestaande tests. De macOS-verificatie is via GitHub Actions geslaagd.

Voer vanuit de projecthoofdmap uit:

```powershell
cargo build
cargo run
```

Open [de statusroute](http://127.0.0.1:3000/status):

```json
{"status":"ok","applicatie":"commandhub"}
```

De server luistert alleen lokaal op poort 3000. Stop met Ctrl+C. Apache, PHP en MySQL zijn niet nodig.

Voer met de server actief in een tweede PowerShell-terminal de geconfigureerde parameterloze tool uit:

```powershell
Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/tools/git_version" -ContentType "application/json" -Body '{}' | ConvertTo-Json
```

Op macOS kies je de aparte configuratie vanuit de projectmap:

```sh
export COMMANDHUB_CONFIG=config/tools.macos.json
cargo run --locked
```

Voer met de server actief in een tweede terminal de native voorbeeldtool uit:

```sh
curl -X POST http://127.0.0.1:3000/tools/echo_ready -H 'Content-Type: application/json' -d '{}'
```

De standaardconfiguratie bevat een Windows-pingvoorbeeld. Kies op macOS expliciet de macOS-configuratie; automatische platformselectie is niet toegevoegd.

De response bevat `success`, `stdout`, `stderr`, `exitCode` en `duration` in milliseconden. De route accepteert een JSON-object met de parameters van de gekozen tool; parameterloze tools verwachten een leeg object. De requestbody mag maximaal 16 KiB zijn. Zie [het API-contract](docs/api-en-configuratie.md) voor foutresponses.

## MCP handmatig testen

Installeer Node.js met npm om de MCP Inspector CLI te gebruiken. Start vanuit de projecthoofdmap de server in de eerste terminal. Gebruik op macOS eerst de hierboven getoonde instelling voor COMMANDHUB_CONFIG.

```sh
cargo run --locked
```

Vraag in een tweede terminal de geconfigureerde tools en hun inputschema’s op. De eerste npx-aanroep downloadt zo nodig de vastgezette Inspector-versie.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --method tools/list
```

Voer daarna de parameterloze Git-tool uit. Deze staat in beide meegeleverde configuraties. Git moet geïnstalleerd zijn.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --method tools/call --tool-name git_version
```

Verwacht een toolresultaat met isError gelijk aan false en uitvoervelden success, stdout, stderr, exitCode en duration in structuredContent en als JSON-tekst. stdout bevat de Git-versie.

De standaardconfiguratie config/tools.json bevat ook rust_details. Test daarmee een toegestane parameterwaarde en daarna een afgewezen waarde.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --method tools/call --tool-name rust_details --tool-arg detail=-vV
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --method tools/call --tool-name rust_details --tool-arg detail=--help
```

De eerste call toont uitgebreide Rust-versie-informatie. De tweede levert isError gelijk aan true met code invalid_parameters. rust_details staat uitsluitend in de standaardconfiguratie. Stop de server na afloop met Ctrl+C.

Deze stappen zijn reproduceerbare handmatige controles. Voor de huidige F05-stand is nog geen Inspector-resultaat aangeleverd. De automatische tests gebruiken een echte rmcp-client via TCP. De CLI ondersteunt ontdekking en toolcalls via Streamable HTTP. [Inspector CLI-documentatie](https://github.com/modelcontextprotocol/inspector/blob/main/clients/cli/README.md).

## Documentatie

| Bestand | Doel |
| --- | --- |
| [Plan van aanpak](docs/plan-van-aanpak.md) | Doel, scope, aanpak en evaluatie |
| [Requirements](docs/requirements.md) | Eisen met MoSCoW en koppeling naar features |
| [Architectuur](docs/architectuur.md) | Modules en gegevensstroom |
| [Security](docs/security.md) | Vertrouwensgrenzen en beveiligingskeuzes |
| [API en configuratie](docs/api-en-configuratie.md) | Voorlopige contracten en open keuzes |
| [Testplan](docs/testplan.md) | Bewijs per requirement |
| [Roadmap](docs/roadmap.md) | Bouwvolgorde en afhankelijkheden |
| [Voortgang](docs/voortgang.md) | Korte actuele overdracht |
| [Features](docs/features/README.md) | Scope, todo's en acceptatiecriteria |
| [Notities](notities/README.md) | Uitleg en geschiedenis per feature |
| Beslissingen: [ADR-001](docs/beslissingen/ADR-001-rust-en-axum.md), [ADR-002](docs/beslissingen/ADR-002-mcp-proef.md) | Vastgelegde ontwerpkeuzes en F00-proefresultaat |

## Werkwijze

Features krijgen een vast ID. Dat ID verandert niet wanneer de planning verschuift. Gedetailleerde todo's staan uitsluitend in het featurebestand. GitHub Issues en het Project bevatten planning en verwijzen naar dat bestand.

Status: Backlog → Ready → In uitvoering → Review → Klaar. Klaar betekent: acceptatiecriteria aangetoond, relevante tests geslaagd, documentatie bijgewerkt en PR gemerged.

Per issue beschrijf ik wat, waarom en hoe. Een feature kan meerdere kleine issues en PR's hebben. Branches gebruiken bijvoorbeeld `feature/F02-toolconfiguratie`. Afgerond werk gaat naar `develop`; opleveringen gaan via PR naar `main`.

Per feature houd ik één notitie bij met uitleg, keuzes, problemen en testresultaten. Nieuwe sessies voeg ik als gedateerde sectie toe aan diezelfde notitie.

## Controles

```powershell
cargo fmt --check
cargo check --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

De features voegen gerichte automatische tests toe. `cargo test --locked` voert de huidige tests uit; bekijk de testuitvoer om te zien welke tests daadwerkelijk zijn uitgevoerd.

Commit `Cargo.lock` na de eerste build. Dit is een applicatie, dus de lockfile hoort bij de repository.

## Bronnen

- [Educom CommandHub](https://e-learning.educom.nu/cases/commandhub/)
- [Educom werkwijze](https://e-learning.educom.nu/cases/case-workflow/)
- [Axum](https://docs.rs/axum/0.8/axum/)
- [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)

## CI-afronding — 2026-11-04

De aangeleverde screenshot van run #39 voor commit 1fb8d71 op feature/F04-parameters toont Success: macOS 57 s, Ubuntu 40 s en Windows 1 m 50 s; totale duur 1 m 54 s. De workflow voert formatter, Clippy, tests en build uit. Hiermee is ook de macOS-platformverificatie bevestigd; de eerdere runnercapaciteitsmelding is geen open blokkade meer. Exacte CI-testtotalen zijn niet zichtbaar. De werking is lokaal bevestigd. Alleen PR-review/merge en branchafsluiting resten; status pas Klaar na bevestigde merge. Bij deze documentatie-update zijn geen broncode gewijzigd, tests herhaald of Git-mutaties uitgevoerd.


