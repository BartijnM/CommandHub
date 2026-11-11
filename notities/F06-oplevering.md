---
feature: F06
tags:
  - commandhub
  - feature-notitie
---

# F06 — Platformcontrole en demonstratie

[Featureplan](../docs/features/F06-oplevering.md)

## Doel

Ik controleer of CommandHub vanuit een schone checkout op Windows, Ubuntu en macOS kan bouwen en via een echte MCP-client bruikbaar is. Daarna maak ik een demo waarmee iemand de oplevering kan herhalen.

## Werksessie op 2026-11-10

Ik heb `config/tools.linux.json` gemaakt op basis van de standaardconfiguratie. Op Windows starten de elf persoonlijke tools met `python`. Op Ubuntu en macOS gebruiken ze `python3`. De drie profielen bevatten elk achttien tools.

Ik heb de MCP-basistest stapsgewijs uitgebreid in de bestaande CI-matrix. Run #56 controleerde Ubuntu. Run #57 controleerde Ubuntu en macOS. De laatste getoonde run controleert ook Windows. `COMMANDHUB_CONFIG` kiest het profiel. De workflow kiest op Windows `commandhub.exe` en op Ubuntu en macOS `commandhub`. De drie jobs gebruiken hetzelfde Bash-script en Node.js 22 voor Inspector.

## Verificatie

De laatste CI-run is groen op alle drie de runners. De getoonde tijden zijn de looptijden van de volledige jobs.

| Runner | Profiel | CI | Jobduur |
| --- | --- | --- | --- |
| Windows (`windows-latest`) | `config/tools.json` | Geslaagd | 4 min 47 s |
| Ubuntu (`ubuntu-latest`) | `config/tools.linux.json` | Geslaagd | 2 min 5 s |
| macOS (`macos-latest`) | `config/tools.macos.json` | Geslaagd | 3 min 14 s |

Elke job begint met een schone checkout en draait formatter, Clippy, Rust-tests, build en de stap **MCP-basistest**. De MCP-stap start de gebouwde server met het platformprofiel, wacht op `/status`, vergelijkt de namen uit `tools/list` met de configuratie en controleert `git_version` via `tools/call`. Ook controleert de stap dat `git_diff_view` met `view=--patch` de toolfout `invalid_parameters` geeft. Inspector 2.11.0 gebruikt Streamable HTTP in legacy-modus. De stap ruimt het serverproces op bij afsluiten.

Dit bewijs geldt voor de basisketen op gehoste runners. De profielen zijn alle drie geladen en ontdekt. De afzonderlijke aanroepen voor Python, `rg`, `pdfinfo` en `ffprobe` vallen buiten deze CI-basistest. Ten tijde van deze platformcontrole stonden de onafhankelijke README-doorloop en volledige opleveringsdemo nog open. De demo is op 2026-11-11 bevestigd.

De screenshot toont een geslaagde pushrun, maar het runnummer, de commit-SHA en de runlink zijn niet zichtbaar. De drie annotaties melden dat `actions/checkout@v4` op Node.js 24 draait vanwege de uitfasering van Node.js 20. De jobs zijn geslaagd. Ik neem de waarschuwing mee als onderhoudspunt voor CI.

## Stoppunt na de platformcontrole

De platformprofielen en schone-checkouttest van de MCP-basisketen zijn afgevinkt. F06 staat op **In uitvoering**. Het demo-draaiboek staat in docs/demo.md. Deze doorloop is later op 2026-11-11 door de eigenaar als werkend bevestigd. Ik toon daarin configuratie, ontdekking, uitvoering, validatiefouten, logging en een timeout. Ik voeg ook een tool toe via JSON en start dezelfde gebouwde executable opnieuw om te demonstreren dat er geen rebuild nodig is. Daarna werk ik de requirementdekking, beperkingen en de opleverings-PR naar `main` uit.

## Demo voorbereiden op 2026-11-11

De eigenaar kiest de Claude Code-extensie als client en toont een geslaagde verbinding met de achttien oorspronkelijke tools. Het [draaiboek](../docs/demo.md) beschrijft voorbereiding per platform, exacte Claude-prompts, verwachte MCP-resultaten en logvelden, timeout en een configuratiewijziging met herstart van dezelfde executable. Voor timeout is een tijdelijke vaste Python-call gekozen die tien seconden wacht met een serverlimiet van 1000 ms. Een tweede tijdelijke tool gebruikt git --version om uitbreiding zonder rebuild te tonen. De lezer voegt deze objecten tijdens de demo aan het bestaande platformprofiel toe en verwijdert ze na afloop.

Claude kan de ongeldige enumwaarde zelf afwijzen. Het draaiboek bevat daarom een aanvullende Inspector-call om de serverresponse reproduceerbaar op te vragen. Verwachte uitkomsten zijn tegen configuratie, foutmapping en loggingcode gecontroleerd. Tijdens het schrijven van het draaiboek moesten de volledige Claude-demo en timeoutconfiguratie nog worden uitgevoerd. De eigenaar heeft de demo daarna als werkend bevestigd.

## Demo bevestigd en documentatie gecontroleerd op 2026-11-11

De eigenaar bevestigt dat de demo werkt. Die bevestiging is voldoende als handmatig bewijs voor ontdekking, projectonderzoek, validatiefout, logging, timeout/herstel en toevoegen van een tool zonder rebuild. Exacte meetwaarden, clientversie en de gebruikte route voor de ongeldige call zijn niet aangeleverd. Het draaiboek registreert de bevestiging zonder extra meetgegevens toe te voegen.

### Documentatiereview

Alle 33 Markdown-bestanden in de repository zijn gelezen. De primaire documenten zijn vergeleken met Cargo.toml, configuratieprofielen, de Rust-kern, REST/MCP-adapters, tests, CI, .mcp.json, het PowerShell-voorbeeld en het Python-toolscript.

| Bestanden | Uitkomst van de controle |
| --- | --- |
| README.md en config/README.md | Platformkeuze, dependencies, Claude Code, Inspector, tooluitvoer en installatiegrenzen bijgewerkt |
| docs/api-en-configuratie.md | Actuele routes, schema, argumentregels, padbeleid, foutcodes en MCP-transport vastgelegd |
| docs/architectuur.md | Bestaande modules, gedeelde state, procesbeheer, annulering en Python-tools beschreven |
| docs/security.md | Maatregelen en expliciete opleveringsbeperkingen samengebracht |
| docs/testplan.md en docs/requirements.md | R01–R21 gekoppeld aan code, tests, bewijs en resterende README-verificatie |
| docs/demo.md | Demo als door eigenaar bevestigd geregistreerd |
| docs/voortgang.md, docs/plan-van-aanpak.md en docs/roadmap.md | Huidige F06-stand en concrete vervolgstappen bijgewerkt |
| docs/features/F00 t/m F07 | Checklists en actuele context gecontroleerd, tegenstrijdige statusvelden gecorrigeerd waar afronding bevestigd is |
| docs/features/README.md en notities/README.md | Indeling en relatieve verwijzingen blijven toepasbaar |
| notities/F00 t/m F07 | Historie behouden, actuele oplevercontext toegevoegd |
| docs/F04-handmatig-testen.md | Aanvullende REST-stappen en het PowerShell-voorbeeld verduidelijkt, Unix-verificatie bijgewerkt |
| docs/beslissingen/ADR-001-rust-en-axum.md | Historische technologiekeuze blijft geldig |
| docs/beslissingen/ADR-002-mcp-proef.md | Latere F05-productiekeuzes toegevoegd bij de oorspronkelijke proef |
| AGENTS.md | Werk- en leerafspraken blijven toepasbaar |

De oude F03- en F07-featurechecklists bevestigen een merge. De tegenstrijdige Review-frontmatter is daarmee gelijkgetrokken. F05 staat als afgerond in de bestaande voortgang en is hierop afgestemd. Voor F00, F01 en F04 ontbreken expliciete historische mergegegevens in hun plannen. Die administratieve velden blijven herkenbaar. Ze worden afzonderlijk van de technische requirementdekking behandeld.

### Requirementreview en beperkingen

Alle Must-eisen hebben implementatie- en testdekking. R17 wacht op de gezamenlijke README-doorloop. R19 is door CI aangetoond. R18 en R20 staan voor later, R21 blijft buiten de eerste oplevering. De grenzen rond serverrechten, padcanonicalisatie, bestaande paden, stringparameters, optionele argumenten, clientdisconnects en externe toolafhankelijkheden staan in security.md.

### Volgende stap

Loop README en handleidingen gezamenlijk door vanaf de installatievereisten, profielkeuze, build, startup en eerste MCP-call. Verwerk eventuele verbeteringen. Daarna de definitieve Cargo-controles en platform-CI beoordelen en de opleverings-PR naar main voorbereiden. F06 blijft In uitvoering. De huidige documentatiereview heeft Markdown gecontroleerd en aangepast. De Cargo-controles zijn hierbij niet opnieuw uitgevoerd.

### Documentcontrole na de wijzigingen

Alle lokale Markdown-links en kopankers zijn gecontroleerd. De codeblokken zijn gesloten en de twaalf JSON-voorbeelden zijn als JSON geparseerd. De drie huidige configuratieprofielen bevatten elk achttien tools. De documentatiereview wijzigt 29 Markdown-bestanden. AGENTS.md, ADR-001 en de twee overzichtsbestanden blijven inhoudelijk geldig.

## README en historische merges bevestigd op 2026-11-11

De eigenaar bevestigt alle historische featuremerges tot en met F05, inclusief F07, en keurt de README goed met een verzoek om de CI-slottekst te actualiseren. F00, F01 en F04 staan nu eveneens op Klaar. De eerdere administratieve open mergevelden zijn hiermee afgehandeld.

De README-slottekst beschrijft de geslaagde F06-platformcontrole van 2026-11-10 en de bevestigde demo en documentatiereview. R17 en alle F06-acceptatiecriteria zijn bevestigd. De volgende stap is de eindcontrole van de definitieve stand en de opleverings-PR naar main. F06 blijft In uitvoering totdat deze afronding is bevestigd. De Cargo-controles zijn bij deze documentatie-update niet herhaald.
