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
- [x] Padbeleid, canonicalisatie en symlinkgedrag implementeren.
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
- Padbeleid is lokaal getest; de Unix-symlinktest is meegenomen in de geslaagde Ubuntu-CI van commit 2d02834. macOS-platformverificatie blijft open.
- Optiebeleid beoordeeld voor de drie beheerde voorbeeldtools in config/tools.json: alleen expliciete keuzelijsten, geen client-executable. Aangepaste tools vragen eigen beoordeling; prefixfilters zijn geen algemene toolisolatie.
- Acceptatiecriteria hierboven zijn lokaal aangetoond; Windows en Ubuntu zijn via Actions geslaagd voor 2d02834; alleen macOS ontbreekt door runnercapaciteit. Status blijft Review tot CI en merge.
- [Handmatige voorbeelden](../F04-handmatig-testen.md).
- [Featurenotitie](../../notities/F04-parameters.md).

## Afronden

- [ ] Relevante controles geslaagd (lokaal en Ubuntu/Windows groen; macOS-runner nog open).
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.


## Laatste CI-stand en vervolg

Screenshot voor 2d02834: Ubuntu 41 s en Windows 1 m 30 s geslaagd. macOS kreeg geen runner en eindigde na 15 m 2 s; geen testfout aangetoond. Op 2026-11-05 de mislukte job opnieuw uitvoeren, daarna bewijs bijwerken en PR-review/merge afronden. Exacte run-URL en testtotalen zijn niet aangeleverd. Zie [hervatprompt](../F04-hervatprompt.md).
