# Plan van aanpak

Bijgewerkt op 2026-11-11. F06 rondt de lokale oplevering af. De configuratiegestuurde kern, REST, MCP, drieplatform-CI en demo zijn aanwezig. De gezamenlijke README-doorloop en definitieve opleveringscontrole volgen.

## Doel

Ik bouw een generieke server waarmee een HTTP- of MCP-client vooraf geconfigureerde commandline-tools ontdekt en uitvoert. Nieuwe tools worden via JSON en herstart toegevoegd zonder opnieuw compileren.

## Technologie en leerdoel

CommandHub gebruikt Rust editie 2024, Axum, Tokio, Serde/serde_json, tracing, process-wrap en de officiële Rust MCP SDK rmcp 3.6.0. Streamable HTTP is op /mcp aangesloten. De getoetste protocolonderhandeling is 2025-11-25. De oorspronkelijke keuzes en proef staan in [ADR-001](beslissingen/ADR-001-rust-en-axum.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md).

Ik heb ervaring met Laravel en .NET en verdiep mij met dit project in ownership, expliciete foutafhandeling, concurrency en procesbeheer. Kleine werkende stappen met uitleg en gerichte controles vormen de leerwerkwijze. De afspraken staan in [AGENTS.md](../AGENTS.md).

## Eerste oplevering

De scope omvat JSON-configuratie bij startup, registry, REST-ontdekking en uitvoering, gecontroleerde stringparameters, padbeleid, begrensde procesuitvoering, MCP via HTTP, logging, platformprofielen en een reproduceerbare demo met Claude Code. Windows, Linux en macOS worden ondersteund.

De toepassing bindt lokaal. Remote authenticatie, YAML, hot-reload, dashboard, database, jobqueue, WebSockets en distributed execution hebben de prioriteit en planning uit [requirements](requirements.md).

## Aanpak en evaluatie

1. Onderzoek de MCP-keten in een geïsoleerde proef.
2. Controleer de projectbasis en bouw configuratie en ontdekking.
3. Voeg begrensde procesuitvoering, platformcontrole en parameters toe.
4. Sluit MCP op dezelfde kern aan en vergelijk het gedrag met REST.
5. Controleer de keten op drie platforms, demonstreer het gebruik en actualiseer de opleverdocumentatie.

Per feature onderbouw ik acceptatiecriteria met gerichte tests, volledige Cargo-controles, CI en reproduceerbare handmatige controles. Daarbij horen fout- en grensgevallen, uitvoeringsduur, procesbeëindiging en herstel na afwijzing. Het [testplan](testplan.md) bevat de requirementdekking.

## Stand van de oplevering

De platformprofielen en schone-checkout-MCP-basistest zijn op 2026-11-10 groen gemeld op Windows, Ubuntu en macOS. De eigenaar bevestigt op 2026-11-11 de volledige [demo](demo.md). De documentatie en requirementmatrix zijn bijgewerkt. De volgende stap is de gezamenlijke README-doorloop. Daarna volgen eindcontroles en de opleverings-PR naar main.

## Planning en Git-werkwijze

Feature-ID’s en afhankelijkheden staan in de [roadmap](roadmap.md). GitHub Issues en Project bevatten planning, mijlpalen en eventuele datums. Per feature houd ik één [notitie](../notities/README.md) bij met keuzes, problemen en verificatie.

Featurewerk gaat via review naar develop. De oplevering gaat via een PR naar main. De eigenaar voert Git-acties uit. Code, documentatie, configuratievoorbeelden en testbewijs horen bij dezelfde oplevering.

## Opleveronderdelen

- Broncode en Cargo.lock.
- README met vereisten, platformkeuze, starten, REST en MCP-clients.
- Configuratieprofielen en handleiding voor tools en gegevensmap.
- API-contract, architectuur, security en expliciete beperkingen.
- Requirementmatrix, CI-resultaten en demo-draaiboek.

Actuele open punten staan in [voortgang](voortgang.md) en het [F06-featureplan](features/F06-oplevering.md).
