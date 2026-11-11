---
id: F04
prioriteit: Must
status: Klaar
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

Zie de taken en acceptatiecriteria hieronder. Dit featureplan beschrijft de eigen implementatiestap. Het actuele productcontract staat in [API en configuratie](../api-en-configuratie.md) en de opleverstand in [voortgang](../voortgang.md).

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
- Pull request gemerged, door eigenaar bevestigd op 2026-11-11.
- Lokale Windows-eindcontrole: formatter en Clippy geslaagd; 138 tests geslaagd, 5 expliciete proceshelpers genegeerd.
- Padbeleid is lokaal getest; de Unix-symlinktest is meegenomen in de geslaagde Ubuntu-CI van commit 2d02834. macOS is inmiddels eveneens geslaagd in run #39.
- Optiebeleid beoordeeld voor de drie beheerde voorbeeldtools in config/tools.json: alleen expliciete keuzelijsten, geen client-executable. Aangepaste tools vragen eigen beoordeling; prefixfilters zijn geen algemene toolisolatie.
- Acceptatiecriteria hierboven zijn lokaal aangetoond; alle drie platforms zijn via Actions geslaagd voor 1fb8d71 (run #39). De eigenaar bevestigt op 2026-11-11 de merge. Status Klaar.
- [Handmatige voorbeelden](../F04-handmatig-testen.md).
- [Featurenotitie](../../notities/F04-parameters.md).

## Afronden

- [x] Relevante controles geslaagd (lokaal en CI op Windows, Ubuntu en macOS).
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [x] PR gemerged, door eigenaar bevestigd op 2026-11-11.


## Historische CI-tussenstand

Screenshot voor 2d02834: Ubuntu 41 s en Windows 1 m 30 s geslaagd. macOS kreeg geen runner en eindigde na 15 m 2 s; geen testfout aangetoond. Deze runnerfout is vervolgens opgelost door de geslaagde run #39 hieronder. Exacte run-URL en testtotalen zijn niet aangeleverd.

## CI-afronding — 2026-11-04

De aangeleverde screenshot van run #39 voor commit 1fb8d71 op feature/F04-parameters toont Success: macOS 57 s, Ubuntu 40 s en Windows 1 m 50 s; totale duur 1 m 54 s. De workflow voert formatter, Clippy, tests en build uit. Hiermee is ook de macOS-platformverificatie bevestigd; de eerdere runnercapaciteitsmelding is geen open blokkade meer. Exacte CI-testtotalen zijn niet zichtbaar. De werking is lokaal bevestigd. De later op 2026-11-11 bevestigde merge rondt F04 af. Bij deze documentatie-update zijn geen broncode gewijzigd, tests herhaald of Git-mutaties uitgevoerd.


## Oplevercontext op 2026-11-11

De parameter- en padfunctionaliteit maakt deel uit van de huidige REST/MCP-keten en wordt in F06 opnieuw door de platform-CI gecontroleerd. Het [testplan](../testplan.md) bevat het huidige bewijs. De historische merge is op 2026-11-11 door de eigenaar bevestigd.
