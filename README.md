# CommandHub

CommandHub is een configuratiegestuurde MCP-server in Rust die externe commandline-tools via HTTP toegankelijk maakt. De server valideert parameters, begrenst processen en geeft resultaten terug. Nieuwe tools worden via JSON-configuratie en een herstart toegevoegd.

## Huidige status

Actief is [F06 — Platformcontrole en demonstratie](docs/features/F06-oplevering.md). De eigenaar heeft de demo met Claude Code als werkend bevestigd. De gezamenlijke README-doorloop en definitieve opleveringscontrole volgen. De CI-matrix bouwt en test CommandHub vanaf een schone checkout op Windows, Ubuntu en macOS. Een echte Inspector-client controleert op elke runner de toollijst, een geslaagde uitvoering en een validatiefout. De optionele toolafhankelijkheden hebben eigen installatievereisten. Zie de [voortgang](docs/voortgang.md) en de [F06-notitie](notities/F06-oplevering.md).

## Installeren en gebruiken

Zie de instructies hieronder voor installatie en Cargo-commando's. De featurevolgorde en branchwerkwijze staan in [de roadmap](docs/roadmap.md).

Clone deze repository en open de projecthoofdmap. Gebruik een recente stabiele Rust-toolchain met Cargo en ondersteuning voor editie 2024. Installeer op Windows de MSVC-buildtools, op Linux een C-linker met de gebruikelijke buildtools en op macOS de Xcode Command Line Tools. Installeer Git voor de voorbeeldtools en tests.

Voeg de componenten voor de ontwikkelcontroles toe.

```sh
rustup component add rustfmt clippy
```

De server zelf gebruikt de Rust-dependencies uit Cargo.lock. Aanvullende programma’s zijn nodig voor de tools die je aanroept.

| Gebruik | Vereiste |
| --- | --- |
| MCP via Inspector | Node.js 22 met npm |
| MCP via Claude Code | VS Code met de Claude Code-extensie en een geschikt account |
| Project zoeken | ripgrep (`rg`) op server-PATH |
| Persoonlijke tools | Python 3.10 of nieuwer op server-PATH |
| PDF-metadata | Poppler met `pdfinfo` |
| Afbeeldings- en video-informatie | FFmpeg met `ffprobe` |

Zie de [configuratiehandleiding](config/README.md) voor de vereisten per tool.

### Starten op Windows

Kies expliciet het Windows-profiel in PowerShell en bouw en start vanuit de projecthoofdmap.

```powershell
$env:COMMANDHUB_CONFIG = "config/tools.json"
cargo build --locked
.\target\debug\commandhub.exe
```

Open [de statusroute](http://127.0.0.1:3000/status). Verwacht het volgende antwoord.

```json
{"status":"ok","applicatie":"commandhub"}
```

De server luistert op 127.0.0.1:3000. Stop met Ctrl+C. Startup toont het aantal geladen tools en het luisteradres.

Voer met de server actief in een tweede PowerShell-terminal de geconfigureerde parameterloze tool uit.

```powershell
Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/tools/git_version" -ContentType "application/json" -Body '{}' | ConvertTo-Json
```

### Starten op Linux of macOS

Kies vanuit de projecthoofdmap het profiel voor je platform.

**Linux**

```sh
export COMMANDHUB_CONFIG=config/tools.linux.json
```

**macOS**

```sh
export COMMANDHUB_CONFIG=config/tools.macos.json
```

Bouw en start de server nadat je één profiel hebt gekozen.

```sh
cargo build --locked
./target/debug/commandhub
```

Gebruik op macOS desgewenst de native voorbeeldtool vanuit een tweede terminal.

```sh
curl -X POST http://127.0.0.1:3000/tools/echo_ready -H 'Content-Type: application/json' -d '{}'
```

De response bevat `success`, `stdout`, `stderr`, `exitCode` en `duration` in milliseconden. De route accepteert een JSON-object met de parameters van de gekozen tool; parameterloze tools verwachten een leeg object. De requestbody mag maximaal 16 KiB zijn. Zie [het API-contract](docs/api-en-configuratie.md) voor foutresponses.

## MCP met Claude Code

De repository bevat [.mcp.json](.mcp.json) met CommandHub als HTTP-server op http://127.0.0.1:3000/mcp. Start CommandHub met het profiel voor je platform. Open het project in VS Code en open `/mcp` in het chatpaneel van de Claude Code-extensie. Sta de projectserver toe wanneer de client daarom vraagt en gebruik **Reconnect**. Verwacht **Connected** en de geconfigureerde toollijst.

Vraag Claude vervolgens het volgende.

```text
Roep git_version van de CommandHub MCP-server aan met {} en toon het resultaat.
```

Open de toolaanroep. Verwacht een succesvolle call met de Git-versie in stdout. De volledige demo met verwachte uitkomsten staat in het [draaiboek](docs/demo.md). De eigenaar heeft die demo op 2026-11-11 als werkend bevestigd. [Claude Code-documentatie](https://code.claude.com/docs/en/vs-code#connect-to-external-tools-with-mcp).

## MCP handmatig testen

Installeer Node.js 22 met npm om de MCP Inspector CLI te gebruiken. Start vanuit de projecthoofdmap de server in de eerste terminal. Kies op Linux en macOS eerst het platformprofiel zoals hierboven beschreven.

```sh
cargo run --locked
```

Vraag in een tweede terminal de geconfigureerde tools en hun inputschema’s op. De eerste npx-aanroep downloadt zo nodig de vastgezette Inspector-versie.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/list
```

Voer daarna de parameterloze Git-tool uit. Deze staat in alle drie de meegeleverde configuraties. Git moet geïnstalleerd zijn.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name git_version
```

Verwacht een toolresultaat met isError gelijk aan false en uitvoervelden success, stdout, stderr, exitCode en duration in structuredContent en als JSON-tekst. stdout bevat de Git-versie.

De Windows- en Linux-profielen bevatten ook rust_details. Test daarmee een toegestane parameterwaarde en daarna een afgewezen waarde.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name rust_details --tool-arg detail=-vV
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name rust_details --tool-arg detail=--help
```

De eerste call toont uitgebreide Rust-versie-informatie. De tweede levert isError gelijk aan true met code invalid_parameters. Gebruik op macOS de gedeelde Git-voorbeelden of de native echo_ready-tool.

De CI-MCP-basistest voert met Inspector op Windows, Ubuntu en macOS ontdekking, git_version en een afgewezen git_diff_view-call uit. De Rust-tests gebruiken daarnaast een echte rmcp-client via TCP. Bij een toolfout zoals invalid_parameters sluit Inspector af met exitcode 5. [Inspector CLI-documentatie voor 2.11.0](https://github.com/modelcontextprotocol/inspector/blob/2.11.0/clients/cli/README.md).

### Projecttools

Start de server vanuit de projecthoofdmap. De Git-tools gebruiken de repository van die werkmap, `project_search` gebruikt `rg` en `project_format_check` gebruikt Cargo met rustfmt.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name repo_status
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name git_recent --tool-args-json '{"count":"5"}'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name git_diff_view --tool-arg view=--stat
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name project_search --tool-arg query=router_with_shutdown
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name project_format_check
```

`git_recent` verwacht `count` als string, dus gebruik JSON zoals `{"count":"5"}`. In PowerShell op Windows kun je voor deze aanroep de argumenten letterlijk aan `npx.cmd` doorgeven.

```powershell
npx.cmd --% --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name git_recent --tool-args-json "{\"count\":\"5\"}"
```

`project_search` geeft exitcode `1` zonder uitvoer als er geen treffers zijn. `project_format_check` toont de verschillen als rustfmt aanpassingen voorstelt.

### Bestanden, notities, media en systeem

Gebruik Python 3.10 of nieuwer. Zorg dat de terminal waarin de server draait op Windows `python` kan starten en op Linux en macOS `python3`. Controleer dat met `python --version` of `python3 --version`. Installeer voor PDF-metadata `pdfinfo` (Poppler) en voor media-informatie `ffprobe` (FFmpeg) en zet ze op het `PATH` van de server.

Maak `CommandHub-data` in je thuismap en plaats daarin de bestanden die je via MCP wilt opvragen. Maak voor de notitietools een submap `notes` met UTF-8-bestanden (`.md` of `.txt`). Kies eventueel een andere bestaande gegevensmap door vóór het starten van de server `COMMANDHUB_DATA_DIR` op een absoluut pad te zetten.

Gebruik in toolaanroepen **relatieve paden vanaf de gegevensmap**. `path=.` opent de hoofdmap. Vervang de voorbeeldbestandsnamen hieronder door bestanden die in jouw map bestaan. Aanhalingstekens rond het hele `key=value`-argument houden paden met spaties bij elkaar.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name folder_overview --tool-arg 'path=.'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name document_search --tool-arg 'query=order-42'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name pdf_info --tool-arg 'path=factuur.pdf'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name file_sha256 --tool-arg 'path=factuur.pdf'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name csv_preview --tool-arg 'path=Verkoop oktober.csv'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name notes_search --tool-arg 'query=planning'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name notes_recent
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name image_info --tool-arg 'path=foto.jpg'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name video_info --tool-arg 'path=opname.mp4'
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name disk_space
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --method tools/call --tool-name service_health --tool-arg service=commandhub
```

De persoonlijke tools schrijven JSON naar `structuredContent.stdout`. Bij een succesvolle aanroep staat `isError` op `false`. Voor `service_health` lees je de uitkomst in `reachable`, `healthy` en `status` binnen die JSON. Gebruik `service=local_web` voor de lokale webserver op poort 80.

De voorbeelden gebruiken expliciet de legacy-protocolmodus van Inspector. CommandHub heeft `2025-11-25` met de SDK-client getoetst. Zie de [Inspector CLI-handleiding voor 2.11.0](https://github.com/modelcontextprotocol/inspector/blob/2.11.0/clients/cli/README.md) en de [configuratiehandleiding](config/README.md) voor de vereisten en grenzen.

## Documentatie

| Bestand | Doel |
| --- | --- |
| [Plan van aanpak](docs/plan-van-aanpak.md) | Doel, scope, aanpak en evaluatie |
| [Requirements](docs/requirements.md) | Eisen met MoSCoW en koppeling naar features |
| [Architectuur](docs/architectuur.md) | Modules en gegevensstroom |
| [Security](docs/security.md) | Vertrouwensgrenzen en beveiligingskeuzes |
| [API en configuratie](docs/api-en-configuratie.md) | Actuele REST-, MCP- en configuratiecontracten |
| [Demo met Claude Code](docs/demo.md) | Draaiboek met prompts, commando’s en verwachte uitkomsten |
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

Cargo.lock hoort bij de applicatie en is opgenomen in de repository. Gebruik --locked voor reproduceerbare dependencyselectie.

## Bronnen

- [Educom CommandHub](https://e-learning.educom.nu/cases/commandhub/)
- [Educom werkwijze](https://e-learning.educom.nu/cases/case-workflow/)
- [Axum](https://docs.rs/axum/0.8/axum/)
- [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)


## Opleverstatus en beperkingen

De F06-platformcontrole van 2026-11-10 is geslaagd op Windows, Ubuntu en macOS met formatter, Clippy, tests, build en Inspector-basistest. De demo met Claude Code is bevestigd. De gezamenlijke README-doorloop, controles van de definitieve stand en opleverings-PR volgen nog. Zie het [testplan en de requirementdekking](docs/testplan.md).

CommandHub is ingericht voor lokaal gebruik met de rechten van het serveraccount. Beheer executables, serveromgeving en gegevensmap zorgvuldig. Padcanonicalisatie kent bestandssysteemraces. Onverwacht clientdisconnect heeft nog geen afzonderlijk annuleringsbeleid. De optionele tools vragen eigen dependencies en bestandsformaten. De volledige grenzen staan in [security en beperkingen](docs/security.md).
