# Roadmap

Feature-ID’s zijn stabiel en de tabelvolgorde is de bouwvolgorde. F06 is actief op 2026-11-11. De featurebestanden bevatten de checklists en de [voortgang](voortgang.md) beschrijft de actuele opleverstand.

| ID | Resultaat | Afhankelijkheid | Milestone |
| --- | --- | --- | --- |
| F00 | MCP-proef en protocolkeuzes | Geen | Ontwerp bevestigd |
| F01 | Projectbasis lokaal gecontroleerd, checks ingericht | Geen | Fundament |
| F02 | Configuratie laden en tools ontdekken | F01, ontwerpinput F00 | Ontdekken |
| F03 | Vaste argumenten begrensd uitvoeren via HTTP | F02 | Uitvoeren |
| F07 | macOS-procescontrole, voorbeeldconfiguratie en CI | F03 | Platformuitbreiding |
| F04 | Dynamische parameters en padbeleid | F03 | Gecontroleerde invoer |
| F05 | MCP via HTTP verbonden met dezelfde kern | F00, F04 | MCP-keten |
| F06 | Windows/Linux/macOS-controles en reproduceerbare demo | F05 | Oplevering |

F07 is op verzoek vóór F04 ingevoegd. De macOS-ondersteuning is geïntegreerd en de platformcontroles zijn geslaagd op macOS, Ubuntu en Windows.

F00 bewaart de oorspronkelijke MCP-proef. De productie-integratie uit F05 is afgerond. In F06 zijn de platformprofielen, CI-basisketen en demo bevestigd. De README-doorloop, eindcontrole en opleverings-PR volgen nog.

## GitHub Project

Velden: status, feature-ID, MoSCoW, milestone, Start Date, End Date en weeklabel. Beschrijf per issue wat/waarom/hoe en link naar feature en notitie. Eén actieve implementatie tegelijk is het uitgangspunt.

## Branches

main = oplevering; develop = integratie; feature/Fxx-naam = samenhangend werk. Voor grotere features kunnen meerdere opeenvolgende issues/PR's worden gebruikt. Houd scope klein en merge alleen gecontroleerde resultaten.

Documentatie en tests horen bij de featurebranch. Maak geen afzonderlijke lange documentatiebranch die losloopt van de implementatie.
