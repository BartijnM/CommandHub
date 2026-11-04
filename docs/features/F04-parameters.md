---
id: F04
prioriteit: Must
status: In uitvoering
milestone: "Gecontroleerde invoer"
---

# F04 — Dynamische parameters en padbeleid

## Doel

Dynamische parameters en padbeleid. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F03.

## Requirements

R07, R08, R09, R10.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [x] Ondersteunde parametertypen en validatieregels vastleggen.
- [x] Vereiste, onbekende en ongeldige parameters controleren.
- [x] Placeholders omzetten naar afzonderlijke argumenten.
- [ ] Optie-injectie per toegestane tool voorkomen.
- [ ] Padbeleid, canonicalisatie en symlinkgedrag implementeren.
- [x] Requestgrootte begrenzen en grensgevallen testen.

## Acceptatiecriteria

- [ ] Ongeldige invoer start geen proces.
- [ ] Client kan geen executable of ongeconfigureerde opties kiezen.
- [ ] Paden buiten toegestane roots worden afgewezen.
- [ ] Geldige parameters leveren de gedocumenteerde argumenten op.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F04-parameters.
- Pull request: nog niet aangemaakt.
- Lokale controles door de eigenaar als geslaagd gemeld op Windows; geen actuele eindtelling. Zie de featurenotitie voor bewijs en beperkingen.
- Stand 2026-11-04: dynamische HTTP-keten aanwezig. Optiebeleidreview, HTTP-padtests, dynamische loggingtests, ontdekschema, eindcontroles en drieplatform-CI blijven open.
- De acceptatiecriteria blijven open tot de aanvullende verificatie en review zijn afgerond.
- [Featurenotitie](../../notities/F04-parameters.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

