---
id: F06
prioriteit: Must
status: In uitvoering
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
- [x] README, technische documentatie en notities actualiseren op 2026-11-11. De gezamenlijke doorloop volgt.
- [x] Demo van configuratie, ontdekking, uitvoering, fouten, logging en timeout voorbereiden. De eigenaar bevestigt de werking op 2026-11-11.
- [x] Requirementdekking en resterende beperkingen nalopen en vastleggen in testplan en security.
- [ ] Opleverings-PR naar main voorbereiden.

## Acceptatiecriteria

- [ ] Andere ontwikkelaar kan vanaf README bouwen en starten.
- [x] Nieuwe tool toevoegen werkt zonder rebuild, bevestigd door de eigenaar via de demo.
- [ ] Demo en testresultaten onderbouwen de Must-eisen.
- [x] Open beperkingen staan expliciet in de opleverdocumentatie.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F06-oplevering.
- Pull request: nog niet aangemaakt.
- CI op 2026-11-10: de meest recente getoonde run is groen op Windows, Ubuntu en macOS. Elke runner gebruikt het eigen toolprofiel en draait de MCP-basistest. Afzonderlijke verificatie van optionele tools en de onafhankelijke README-doorloop blijven open.
- [Demo-draaiboek](../demo.md) met Claude Code en verwachte uitkomsten. De eigenaar bevestigt op 2026-11-11 dat de demo werkt.
- [Featurenotitie](../../notities/F06-oplevering.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [x] Documentatie en notitie bijgewerkt op 2026-11-11.
- [x] Voortgang bijgewerkt op 2026-11-11.
- [ ] PR gemerged; pas daarna status Klaar.

## Volgende stap

Loop README en handleidingen gezamenlijk door om het resterende bouw-/startcriterium en R17 te bevestigen. Daarna volgen de eindcontroles op de definitieve stand en de opleverings-PR naar main. Requirementdekking staat in het [testplan](../testplan.md) en beperkingen in [security](../security.md).
