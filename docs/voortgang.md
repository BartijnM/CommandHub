# Huidige voortgang

Bijgewerkt op 2026-11-11.

## Actief

[F06 — Platformcontrole en demonstratie](features/F06-oplevering.md) staat op **In uitvoering**. De platformprofielen, schone-checkoutcontrole en demo zijn afgerond. De eigenaar bevestigt dat het demo-draaiboek met Claude Code werkt.

## Afgerond voor F06

- Windows, Linux en macOS hebben elk een profiel met achttien tools. De standaardconfiguratie gebruikt python, de Unix-profielen python3.
- CI bouwt en test op drie platforms. De Inspector-basistest controleert startup, toollijst, git_version en een validatiefout met het gekozen profiel.
- De [demo](demo.md) toont ontdekking, projectonderzoek, foutafhandeling, logging, timeout met herstel en toevoegen van een tool zonder rebuild.
- De documentatiereview heeft README, configuratiehandleiding, API-contract, architectuur, security, testplan, requirements, planning, featureplannen en notities gecontroleerd en actuele afwijkingen verwerkt.
- Het [testplan](testplan.md) koppelt alle requirements aan implementatie, bewijs en resterende verificatie. [Security](security.md) bevat de opleveringsbeperkingen.

## Verificatie

| Bewijs | Resultaat |
| --- | --- |
| Windows F05-eindcontrole op 2026-11-09 | Formatter en Clippy geslaagd, 156 tests geslaagd en 7 proceshelpers, 9,77 s |
| F05 CI-run #50 | Ubuntu, macOS en Windows geslaagd |
| F06 CI op 2026-11-10 | Windows 4 min 47 s, Ubuntu 2 min 5 s, macOS 3 min 14 s, alle jobs geslaagd |
| Claude Code-demo op 2026-11-11 | Door eigenaar als werkend bevestigd |

De F06-CI omvat formatter, Clippy, tests, build en Inspector vanaf schone checkout. De laatste screenshot bevat geen exact runnummer, commit-SHA of CI-testtotalen. De lokale aantallen hierboven zijn historische Windows-resultaten.

## Volgende stap

Loop de README en handleidingen samen met de eigenaar door, beginnend bij de vereisten, profielkeuze, build, startup en een MCP-call. Hiermee wordt het resterende acceptatiecriterium voor zelfstandig bouwen en starten gecontroleerd.

Daarna volgen de eindcontroles op de definitieve stand en het voorbereiden van de opleverings-PR naar main. De eigenaar voert commits, push en merge uit. F06 krijgt status Klaar na bevestigde review en merge.

## Aandachtspunten

- Optionele tools vereisen eigen installatie van Python, rg, pdfinfo en ffprobe. De basisketen in CI controleert de beschikbare Git-tools. Een volledige optionele-toolmatrix is nog niet vastgelegd.
- CommandHub gebruikt lokale binding en de rechten van het serveraccount. Remote authenticatie en autorisatie zijn later werk.
- Padcanonicalisatie heeft bestandssysteemraces. Alleen bestaande absolute padparameters worden door de kern ondersteund. De Python-bestandstools hebben een afzonderlijk relatief padbeleid.
- Onverwacht clientdisconnect heeft nog geen expliciet annuleringsbeleid. Expliciete MCP-annulering en server-shutdown zijn aangesloten.
- De MCP-shutdowntestclient gebruikt NeverRetry. Windows-symlinks/junctions hebben geen afzonderlijke test.
- De Node-runtimewaarschuwing bij actions/checkout@v4 is als onderhoudspunt vastgelegd.

Details en oudere resultaten staan in de [F06-notitie](../notities/F06-oplevering.md), het [testplan](testplan.md) en de [eerdere featurenotities](../notities/README.md).
