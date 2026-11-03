# Roadmap

ID's zijn stabiel; de tabelvolgorde is de bouwvolgorde. Alle toekomstige features hebben status Backlog. Datums worden in GitHub gepland.

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

F07 is op verzoek vóór F04 ingevoegd; bestaande feature-ID’s veranderen niet. De eerste macOS-CI staat nog open.

F00 is een korte technische proef; schrijf de definitieve executor niet alvast. F01 heeft al een minimale codebasis in dit pakket, maar lokale verificatie en repository-inrichting blijven nodig.

## GitHub Project

Velden: status, feature-ID, MoSCoW, milestone, Start Date, End Date en weeklabel. Beschrijf per issue wat/waarom/hoe en link naar feature en notitie. Eén actieve implementatie tegelijk is het uitgangspunt.

## Branches

main = oplevering; develop = integratie; feature/Fxx-naam = samenhangend werk. Voor grotere features kunnen meerdere opeenvolgende issues/PR's worden gebruikt. Houd scope klein en merge alleen gecontroleerde resultaten.

Documentatie en tests horen bij de featurebranch. Maak geen afzonderlijke lange documentatiebranch die losloopt van de implementatie.

