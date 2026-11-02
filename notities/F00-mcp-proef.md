---
feature: F00
tags:
  - commandhub
  - feature-notitie
---

# F00 — MCP-proef en protocolkeuzes

[Featureplan](../docs/features/F00-mcp-proef.md) · [Beslisdocument](../docs/beslissingen/ADR-002-mcp-proef.md)

## Doel en resultaat

Deze aparte proef controleert of de officiële Rust-SDK een kleine MCP-server via HTTP kan aanbieden, zonder de CommandHub-server al met MCP te integreren. Een MCP-client kon de vaste tool `hello` ontdekken en aanroepen. De proef gaf als resultaat `De MCP-testtool werkt.`.

## Hoe de proef werkt

In [main.rs](../prototypes/f00-mcp-probe/src/main.rs) beschrijft `ProbeServer` de tool met `#[tool]`. `#[tool_router(server_handler)]` laat de SDK de MCP-afhandeling voor die tool genereren.

`StreamableHttpService` koppelt de handler aan het MCP-transport. De closure `|| Ok(ProbeServer)` maakt een serverinstantie wanneer de SDK die nodig heeft. `Arc<LocalSessionManager>` deelt het lokale sessiebeheer tussen requests. Axum routeert de MCP-service op `/mcp`; de TCP-listener bindt aan `127.0.0.1:3001`.

`Arc` is een gedeelde pointer: meerdere delen van de server kunnen dezelfde sessiebeheerder gebruiken zonder die telkens te verplaatsen. De proef gebruikt een lokale sessiebeheerder; dit is geen opslag die sessies na een herstart bewaart.

## Geverifieerde keuzes en bevindingen

- SDK: `rmcp` 3.6.0 met features `server`, `schemars` en `transport-streamable-http-server`; zie [Cargo.toml](../prototypes/f00-mcp-probe/Cargo.toml).
- Transport: Streamable HTTP op `http://127.0.0.1:3001/mcp`.
- MCP-client: MCP Inspector CLI 2.11.0. De eerste proefaanroepen gebruikten geen expliciete versiepin; de herhaalbare controle op 2026-10-20 gebruikte wel expliciet 2.11.0.
- Protocol: de `initialize`-response meldde `2025-11-25`.
- Toolontdekking: `tools/list` gaf `hello` met een leeg object als inputschema.
- Toolaanroep: `tools/call` gaf de vaste tekst terug met `isError: false`.
- Serverinformatie uit `initialize`: `rmcp`, versie `3.6.0`.

Zie [ADR-002](../docs/beslissingen/ADR-002-mcp-proef.md) voor de afbakening, transportbeveiliging en het verschil met REST.

## Verificatie

Op Windows is de proefserver gestart met `cargo run`. Met MCP Inspector CLI zijn de volgende controles uitgevoerd:

1. `--method initialize` — server meldde protocol `2025-11-25` en toolcapaciteit.
2. `--method tools/list` — client ontdekte `hello`.
3. `--method tools/call --tool-name hello` — client ontving `De MCP-testtool werkt.` en `isError: false`.
4. Met Inspector 2.11.0 is `tools/list` herhaald; de client ontdekte opnieuw `hello`.
5. Met `--method tools/call --tool-name bestaat_niet` gaf de client `{"error":{"code":"tool_not_found","message":"Tool 'bestaat_niet' not found on server."}}` terug. Dit bevestigt de MCP-fout voor een onbekende toolnaam; er is geen HTTP-statuscode gemeten.
6. Met Inspector 2.11.0 is `hello` aangeroepen met `--tool-arg extra=value`. De server gaf alsnog de normale tekst terug met `isError: false`. De handler heeft geen parameters en dit resultaat laat zien dat deze proefaanroep het extra veld niet afwees. Het bewijst niet dat de SDK in het algemeen JSON Schema valideert of dat toekomstige toolinvoer veilig wordt gecontroleerd.

Ook is `cargo check --locked` uitgevoerd. De eerste controle leverde alleen twee waarschuwingen op: een ongebruikte import en de nog niet aangemaakte serverstruct. Nadat de HTTP-service was aangesloten, wordt de serverinstantie gebruikt. De latere geslaagde MCP-clientaanroepen bevestigen dat de proefserver op dat moment draaide.

Deze resultaten gelden voor de geïsoleerde proef, niet voor de hoofdapplicatie. De versie is vastgelegd voor de herhaalde controles; de oorspronkelijke onversieerde aanroepen zijn niet aan een exacte Inspector-versie toe te schrijven. De foutwaarnemingen zijn beperkt tot onbekende toolnaam en extra argument; dit is geen volledige MCP-foutmatrix. De proef toont specifiek dat we niet mogen aannemen dat een extra argument bij een parameterloze handler automatisch wordt afgewezen.

De hoofdcrate en de geïsoleerde proef zijn op Windows beide gecontroleerd met `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` en `cargo test --locked`; alle commando's slaagden. `cargo test` rapporteerde in beide crates 0 tests. Dit bewijst dat de crates voor deze configuratie bouwen, maar niet dat er automatische functionele tests of Linux-controles zijn.

## Open punten

- F00 onderzocht foutgedrag voor een onbekende tool en een extra veld. Een volledige foutmatrix en extra protocolnegatieve tests vallen buiten deze minimale proef en moeten bij F05 worden uitgebreid waar relevant.
- De beveiligingsafbakening is alleen voor lokaal gebruik gedocumenteerd: loopback-binding en SDK-standaard Host-controle; Origin-validatie staat standaard uit. Er is geen Origin-aanvalstest uitgevoerd en dit is geen remote-security-audit.
- F00 staat klaar voor review. Jij voert de Git-sync uit; de PR moet nog worden gemaakt/beoordeeld/gemerged voordat de feature de status Klaar krijgt.

## Werksessie

### 2026-10-20

- Doel: onderzoeken of de officiële Rust-SDK met een echte MCP-client via HTTP werkt.
- Gewijzigd: aparte proef onder `prototypes/f00-mcp-probe`; geen wijziging aan de CommandHub-applicatiecode.
- Geleerd: de SDK registreert de tool via macros en biedt hem aan via Streamable HTTP; protocolonderhandeling en toolinteractie zijn door een echte client bevestigd.
- Gecontroleerd: `cargo check --locked`; met MCP Inspector CLI zijn `initialize`, `tools/list`, `tools/call`, `tool_not_found` en een `hello`-aanroep met extra argument geprobeerd. Voor de herhaalde tests is versie 2.11.0 expliciet gebruikt.
- Open vragen: zie [Open punten](#open-punten).

### 2026-10-20 — Afrondingscontrole

- Doel: F00 gereedmaken voor review.
- Gewijzigd: documentatie bijgewerkt met extra-argumentgedrag en eindcontroleresultaten.
- Geleerd: de extra parameter werd geaccepteerd; vertrouw voor latere invoervalidatie op expliciete validatie in de kern, niet op deze proefhandler.
- Gecontroleerd: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` en `cargo test --locked` vanuit zowel repositoryroot als proefmap; alle geslaagd, beide testbuilds hadden 0 tests.
- Open vragen: PR-review en merge; bredere protocol/securitytests horen bij latere integratie.

## Vervolg

F00 is inhoudelijk klaar voor review. Bekijk de wijzigingen en voer zelf de Git-sync uit. Na beoordeling en merge van de PR kan de status van F00 naar Klaar; zie het [featureplan](../docs/features/F00-mcp-proef.md).
