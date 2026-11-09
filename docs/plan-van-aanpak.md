# Plan van aanpak

Status op 2026-11-09. Dynamische parameters en padbeleid zijn geïmplementeerd. F05 koppelt MCP aan dezelfde kern, met geslaagde lokale Windows-controles. MCP-padvalidatie is lokaal via de clientketen gecontroleerd. F05 staat op Review met platform-CI en PR-review/merge open.

## Doel

Ik bouw een generieke server waarmee een HTTP- of MCP-client vooraf geconfigureerde commandline-tools kan ontdekken en uitvoeren. Nieuwe tools moeten zonder opnieuw compileren kunnen worden toegevoegd.

## Technologie

Rust met Axum voor HTTP, Tokio voor async verwerking, Serde/serde_json voor JSON, tracing voor logging en de officiële Rust MCP SDK voor het protocol. De geïsoleerde F00-proef gebruikt rmcp 3.6.0 en Streamable HTTP; protocolwaarneming en clientversie staan in ADR-002. Definitieve integratiekeuzes worden in F05 getoetst.

Motivatie: ik heb al ervaring met Laravel en .NET en wil mij verdiepen in ownership, expliciete foutafhandeling, concurrency en procesbeheer. Zie ADR-001.

## Eerste oplevering

JSON-configuratie bij opstarten, registry, HTTP-toolroutes, gecontroleerde parameters, begrensde procesuitvoering, MCP via HTTP, logging, voorbeeldtools en demonstraties op Windows en Linux. Lokaal beheer is het uitgangspunt; publieke hosting is geen onderdeel van de eerste oplevering.

Geen database of frontend nodig. Hot-reload, YAML, jobqueue, WebSockets en distributed execution vallen buiten de eerste versie.

## Aanpak

1. Eisen en ontwerp nalopen en onbekende MCP-details met een kleine proef onderzoeken.
2. Projectbasis controleren.
3. Werkende features realiseren volgens docs/roadmap.md.
4. Security en tests in elke feature meenemen.
5. Demonstreerbare eerste versie opleveren en documentatie actualiseren.

## Evaluatie

Per feature toon ik acceptatiecriteria aan met tests en/of reproduceerbare controles. Ik meet uitvoeringsduur, beëindiging bij timeout, begrensde output en gedrag bij gelijktijdige requests. Succes betekent dat grenzen aantoonbaar worden gerespecteerd, niet alleen dat de normale route werkt.

## Planning

Concrete datums vul ik in GitHub Project Roadmap in, met Start Date, End Date en weeklabels. Afhankelijkheden staan in de featurebestanden. Ik plan eerst F00–F02 concreet; latere schattingen herzie ik op basis van ervaring.

## Oplevering en werkwijze

Code, README, ontwerp, configuratievoorbeelden, voorbeeldrequests en demonstratie. Per issue: wat, waarom, hoe en voortgang. Volgens de Educom-werkwijze commit en push ik rond 12:00 en 17:00, ook als het werk nog niet klaar is. Dit is mijn werkwijze; er worden geen automatische pushes ingesteld.

## Nog vast te stellen

- Deadline en beschikbare ontwikkeltijd.
- Definitieve MCP-integratie en transportbeveiliging voor F05; de F00-proef staat in ADR-002.
- F05-platformverificatie via GitHub Actions en PR-review/merge. REST en MCP delen het parameter- en padbeleid en zijn lokaal ook voor padvalidatie vergeleken.
- Platformvoorbeelden en volledige ketencontrole bij F06; F03 gebruikt al Unix-procesgroepen en Windows Job Objects.
- Naamgeving van documenten conform de opleidingsafspraken.

