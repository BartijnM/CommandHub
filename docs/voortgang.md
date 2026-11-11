# Huidige voortgang

Bijgewerkt op 2026-11-11.

## Huidige status

[F06 — Platformcontrole en demonstratie](features/F06-oplevering.md) staat op **Klaar**. Ik heb de merge naar develop op 2026-11-11 bevestigd en daarna develop via PR #17 naar main gemerged. CI-run #65 op e9790e6 is geslaagd op Windows, Ubuntu en macOS. De eerste oplevering is afgerond. De platformprofielen, schone-checkoutcontrole, demo en README-review zijn afgerond. Ik heb bevestigd dat het demo-draaiboek met Claude Code werkt.

## Afgerond voor F06

- Windows, Linux en macOS hebben elk een profiel met achttien tools. De standaardconfiguratie gebruikt python, de Unix-profielen python3.
- CI bouwt en test op drie platforms. De Inspector-basistest controleert startup, toollijst, git_version en een validatiefout met het gekozen profiel.
- De [demo](demo.md) toont ontdekking, projectonderzoek, foutafhandeling, logging, timeout met herstel en toevoegen van een tool zonder rebuild.
- De documentatiereview heeft README, configuratiehandleiding, API-contract, architectuur, security, testplan, requirements, planning, featureplannen en notities gecontroleerd en actuele afwijkingen verwerkt.
- Ik heb op 2026-11-11 de README en de historische featuremerges bevestigd.
- Het [testplan](testplan.md) koppelt alle requirements aan implementatie, bewijs en resterende verificatie. [Security](security.md) bevat de opleveringsbeperkingen.

## Verificatie

| Bewijs | Resultaat |
| --- | --- |
| Windows F05-eindcontrole op 2026-11-09 | Formatter en Clippy geslaagd, 156 tests geslaagd en 7 proceshelpers, 9,77 s |
| F05 CI-run #50 | Ubuntu, macOS en Windows geslaagd |
| F06 CI op 2026-11-10 | Windows 4 min 47 s, Ubuntu 2 min 5 s, macOS 3 min 14 s, alle jobs geslaagd |
| Claude Code-demo op 2026-11-11 | Door mij als werkend bevestigd |
| README en historische featuremerges op 2026-11-11 | Door mij goedgekeurd en bevestigd |
| F06-merge naar develop op 2026-11-11 | Door mij bevestigd |
| Oplevering op main op 2026-11-11 | PR #17 gemerged, commit e9790e6, CI-run #65 geslaagd op drie platforms |

De F06-CI omvat formatter, Clippy, tests, build en Inspector vanaf schone checkout. De laatste screenshot bevat geen exact runnummer, commit-SHA of CI-testtotalen. De lokale aantallen hierboven zijn historische Windows-resultaten.

## Oplevering afgerond

Alle feature- en opleveringsstappen zijn afgerond. Ik heb de merge en groene CI op main vastgelegd. De hieronder beschreven beperkingen en onderhoudspunten blijven beschikbaar voor eventueel vervolgwerk.

## Aandachtspunten

- Optionele tools vereisen eigen installatie van Python, rg, pdfinfo en ffprobe. De basisketen in CI controleert de beschikbare Git-tools. Een volledige optionele-toolmatrix is nog niet vastgelegd.
- CommandHub gebruikt lokale binding en de rechten van het serveraccount. Remote authenticatie en autorisatie zijn later werk.
- Padcanonicalisatie heeft bestandssysteemraces. Alleen bestaande absolute padparameters worden door de kern ondersteund. De Python-bestandstools hebben een afzonderlijk relatief padbeleid.
- Onverwacht clientdisconnect heeft nog geen expliciet annuleringsbeleid. Expliciete MCP-annulering en server-shutdown zijn aangesloten.
- De MCP-shutdowntestclient gebruikt NeverRetry. Windows-symlinks/junctions hebben geen afzonderlijke test.
- De Node-runtimewaarschuwing bij actions/checkout@v4 is als onderhoudspunt vastgelegd.

Details en oudere resultaten staan in de [F06-notitie](../notities/F06-oplevering.md), het [testplan](testplan.md) en de [eerdere featurenotities](../notities/README.md).
