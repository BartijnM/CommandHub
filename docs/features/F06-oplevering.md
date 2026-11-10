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

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [x] Voorbeeldconfiguraties voor Windows, Linux en macOS afronden; bouw voort op F07.
- [x] Schone installatie en volledige keten op alle drie platforms testen. De CI-basistest bouwt vanaf een schone checkout en controleert startup, ontdekking, uitvoering en validatiefout via Inspector.
- [ ] README, technische documentatie en notities actualiseren.
- [ ] Demo van configuratie, ontdekking, uitvoering, fouten, logging en timeout voorbereiden.
- [ ] Requirementdekking en resterende beperkingen nalopen.
- [ ] Opleverings-PR naar main voorbereiden.

## Acceptatiecriteria

- [ ] Andere ontwikkelaar kan vanaf README bouwen en starten.
- [ ] Nieuwe tool toevoegen werkt zonder rebuild.
- [ ] Demo en testresultaten onderbouwen de Must-eisen.
- [ ] Open beperkingen staan expliciet in de opleverdocumentatie.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F06-oplevering.
- Pull request: nog niet aangemaakt.
- CI op 2026-11-10: de meest recente getoonde run is groen op Windows, Ubuntu en macOS. Elke runner gebruikt het eigen toolprofiel en draait de MCP-basistest. Afzonderlijke verificatie van optionele tools en de onafhankelijke README-doorloop blijven open.
- [Featurenotitie](../../notities/F06-oplevering.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.
