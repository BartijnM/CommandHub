---
id: F01
prioriteit: Must
status: Klaar
milestone: "Fundament"
---

# F01 — Projectbasis

## Doel

Projectbasis. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

Geen.

## Requirements

R13, R15, R19.

## Scope

Zie de taken en acceptatiecriteria hieronder. Dit featureplan beschrijft de eigen implementatiestap. Het actuele productcontract staat in [API en configuratie](../api-en-configuratie.md) en de opleverstand in [voortgang](../voortgang.md).

## Todo

- [x] Rust-project en minimale Axum-statusroute lokaal controleren.
- [x] Logging en Ctrl+C-afhandeling controleren.
- [x] Cargo.lock opnemen in de repository. De F06-CI bouwt vanaf schone checkout met --locked.
- [x] Formatter, Clippy en build controleren; CI voor Windows/Linux voorbereiden.

## Acceptatiecriteria

- [x] GET /status geeft 200 met status en applicatienaam.
- [x] cargo fmt --check en Clippy slagen.
- [x] Documentatie maakt duidelijk wat wel en niet geïmplementeerd is.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F01-projectbasis.
- Pull request gemerged, door eigenaar bevestigd op 2026-11-11.
- Tests/handmatige verificatie staan in de [F01-notitie](../../notities/F01-projectbasis.md). De latere F06-CI bevestigt de basis op Windows, Ubuntu en macOS.
- [Featurenotitie](../../notities/F01-projectbasis.md).

## Afronden

- [x] Relevante controles geslaagd.
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [x] PR gemerged, door eigenaar bevestigd op 2026-11-11.


## Oplevercontext op 2026-11-11

De projectbasis is in gebruik en de platform-CI controleert formatter, Clippy, tests en build. De eigenaar bevestigt op 2026-11-11 de historische merge van F01. Status Klaar.
