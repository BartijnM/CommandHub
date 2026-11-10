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

Dit bewijs geldt voor de basisketen op gehoste runners. De profielen zijn alle drie geladen en ontdekt. De afzonderlijke aanroepen voor Python, `rg`, `pdfinfo` en `ffprobe` vallen buiten deze CI-basistest. Een onafhankelijke README-doorloop en de volledige opleveringsdemo staan nog open.

De screenshot toont een geslaagde pushrun, maar het runnummer, de commit-SHA en de runlink zijn niet zichtbaar. De drie annotaties melden dat `actions/checkout@v4` op Node.js 24 draait vanwege de uitfasering van Node.js 20. De jobs zijn geslaagd. Ik neem de waarschuwing mee als onderhoudspunt voor CI.

## Stoppunt en vervolg

De platformprofielen en schone-checkouttest van de MCP-basisketen zijn afgevinkt. F06 staat op **In uitvoering**. De volgende sessie begin ik met het demo-draaiboek. Ik toon daarin configuratie, ontdekking, uitvoering, validatiefouten, logging en een timeout. Ik voeg ook een tool toe via JSON en start dezelfde gebouwde executable opnieuw om te demonstreren dat er geen rebuild nodig is. Daarna werk ik de requirementdekking, beperkingen en de opleverings-PR naar `main` uit.
