---
id: F04
prioriteit: Must
status: Review
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
- [x] Optie-injectie per toegestane tool voorkomen.
- [ ] Padbeleid, canonicalisatie en symlinkgedrag implementeren.
- [x] Requestgrootte begrenzen en grensgevallen testen.

## Acceptatiecriteria

- [x] Ongeldige invoer start geen proces.
- [x] Client kan geen executable of ongeconfigureerde opties kiezen.
- [x] Paden buiten toegestane roots worden afgewezen.
- [x] Geldige parameters leveren de gedocumenteerde argumenten op.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F04-parameters.
- Pull request: nog niet aangemaakt.
- Lokale Windows-eindcontrole door de assistent: formatter en Clippy geslaagd; 138 tests geslaagd, 5 expliciete proceshelpers genegeerd.
- Padbeleid is lokaal geïmplementeerd en getest; de todo blijft open tot de Unix-symlinktest via CI is bevestigd.
- Optiebeleid beoordeeld voor de drie beheerde voorbeeldtools in config/tools.f04.json: alleen expliciete keuzelijsten, geen client-executable. Aangepaste tools vragen eigen beoordeling; prefixfilters zijn geen algemene toolisolatie.
- Acceptatiecriteria hierboven zijn lokaal aangetoond; definitieve platformacceptatie volgt via Actions op Windows/Linux/macOS. Status blijft Review tot CI en merge.
- [Handmatige voorbeelden](../F04-handmatig-testen.md).
- [Featurenotitie](../../notities/F04-parameters.md).

## Afronden

- [ ] Relevante controles geslaagd (lokaal groen; drieplatform-CI nog open).
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

