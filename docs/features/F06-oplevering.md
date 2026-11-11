---
id: F06
prioriteit: Must
status: Klaar
milestone: "Oplevering"
---

# F06 — Platformcontrole en demonstratie

## Doel

Platformcontrole en demonstratie. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F05.

## Requirements

R14, R17, R19.

## Scope

Zie de taken en acceptatiecriteria hieronder. Dit featureplan beschrijft de eigen implementatiestap. Het actuele productcontract staat in [API en configuratie](../api-en-configuratie.md) en de opleverstand in [voortgang](../voortgang.md).

## Todo

- [x] Voorbeeldconfiguraties voor Windows, Linux en macOS afronden; bouw voort op F07.
- [x] Schone installatie en volledige keten op alle drie platforms testen. De CI-basistest bouwt vanaf een schone checkout en controleert startup, ontdekking, uitvoering en validatiefout via Inspector.
- [x] README, technische documentatie en notities actualiseren op 2026-11-11. Ik heb de README op 2026-11-11 beoordeeld en goedgekeurd.
- [x] Demo van configuratie, ontdekking, uitvoering, fouten, logging en timeout voorbereiden. Ik heb de werking op 2026-11-11 bevestigd.
- [x] Requirementdekking en resterende beperkingen nalopen en vastleggen in testplan en security.

## Acceptatiecriteria

- [x] Andere ontwikkelaar kan vanaf README bouwen en starten. Ik heb de README op 2026-11-11 beoordeeld en goedgekeurd.
- [x] Nieuwe tool toevoegen werkt zonder rebuild, bevestigd door mij via de demo.
- [x] Demo, testresultaten en de goedgekeurde README onderbouwen de Must-eisen.
- [x] Open beperkingen staan expliciet in de opleverdocumentatie.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F06-oplevering.
- Feature gemerged naar develop, door mij bevestigd op 2026-11-11.
- CI op 2026-11-10: de meest recente getoonde run is groen op Windows, Ubuntu en macOS. Elke runner gebruikt het eigen toolprofiel en draait de MCP-basistest. Afzonderlijke verificatie van alle optionele tools per platform blijft een gedocumenteerde beperking. De README is op 2026-11-11 door mij goedgekeurd.
- [Demo-draaiboek](../demo.md) met Claude Code en verwachte uitkomsten. Ik heb op 2026-11-11 bevestigd dat de demo werkt.
- [Featurenotitie](../../notities/F06-oplevering.md).

## Afronden

- [x] Relevante controles geslaagd volgens de vastgelegde drieplatform-CI van 2026-11-10.
- [x] Documentatie en notitie bijgewerkt op 2026-11-11.
- [x] Voortgang bijgewerkt op 2026-11-11.
- [x] Feature gemerged naar develop, door mij bevestigd op 2026-11-11. Status Klaar.

## Oplevering naar main

Ik heb develop via PR #17 naar main gemerged op 2026-11-11. Mergecommit e9790e6 heeft een geslaagde CI-run #65.

- [x] Opleverings-PR #17 van develop naar main voorbereid.
- [x] CI-run #65 op main geslaagd op Windows, Ubuntu en macOS.
- [x] Opleverings-PR #17 naar main gemerged, commit e9790e6.

Ik heb de eerste oplevering afgerond. Requirementdekking staat in het [testplan](../testplan.md) en beperkingen in [security](../security.md).
