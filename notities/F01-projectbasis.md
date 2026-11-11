---
feature: F01
tags:
  - commandhub
  - feature-notitie
---

# F01 — Projectbasis

[Featureplan](../docs/features/F01-projectbasis.md)

## Leeswijzer bij de oplevering

Deze notitie bewaart de ontwikkeling en resultaten per werksessie. Beschrijvingen van oude ingangen, configuraties en open punten horen bij hun gedateerde stand. Het actuele gedrag staat in het [API-contract](../docs/api-en-configuratie.md), de [architectuur](../docs/architectuur.md) en de [voortgang](../docs/voortgang.md).

## Wat bouw ik en waarom?

F01 controleert de bestaande minimale Axum-applicatie en legt de basis vast voor verdere ontwikkeling. Er is geen nieuwe Rust-functionaliteit toegevoegd. De actuele checklist staat in het featureplan.

## Hoe werkt de code?

`main` initialiseert `tracing`, bindt de Tokio-listener aan `127.0.0.1:3000` en start Axum met een router. De router koppelt `GET /status` aan de statuscontroller. Axum verzorgt de HTTP-afhandeling; Tokio levert de async runtime en Ctrl+C-signaalafhandeling. De graceful-shutdown-hook laat Axum stoppen wanneer Ctrl+C wordt ontvangen. Zie [main.rs](../src/main.rs) en [routes.rs](../src/routes.rs).

## Keuzes en alternatieven

De bestaande Axum/Tokio-basis is behouden. Het leerdoel en de technologiekeuze staan in [ADR-001](../docs/beslissingen/ADR-001-rust-en-axum.md).

## Problemen en oplossingen

Tijdens deze basiscontrole zijn geen aanvullende problemen vastgelegd.

## Verificatie

Uitgevoerd op Windows op 2026-10-21:

- `cargo fmt --check`: geslaagd.
- `cargo clippy --all-targets --locked -- -D warnings`: geslaagd.
- `cargo test --locked`: geslaagd; er zijn nog 0 tests.
- `cargo build --locked`: geslaagd.
- `GET http://127.0.0.1:3000/status`: HTTP 200 met `{"status":"ok","applicatie":"commandhub"}`.
- Startup logging was zichtbaar. Ctrl+C stopte de server; de terminal rapporteerde exitcode 1 doordat het draaiende proces met een interrupt werd afgebroken.
- CI-workflow voor Ubuntu en Windows toegevoegd in `.github/workflows/ci.yml`; uitvoering op GitHub is nog niet geverifieerd.
- `Cargo.lock` bestaat al. Committen, repository-/branchinrichting en GitHub Project blijven open projectstappen; hiervoor zijn geen Git-commando's uitgevoerd.

Linux is lokaal niet gecontroleerd. De CI-matrix moet dat na publicatie van de branch aantonen.

## Werksessies

### 2026-10-21

- Doel: F01 lokaal controleren en de Windows/Linux-CI voorbereiden.
- Gewijzigd: CI-workflow toegevoegd; featurechecklist en deze notitie bijgewerkt.
- Geleerd: `cargo test` slaagt, maar bewijst nog geen gedragstests omdat er 0 tests zijn.
- Gecontroleerd: formatter, Clippy, build en de HTTP-statusroute op Windows; details staan hierboven.
- Open: commit van `Cargo.lock` en de overige Git-/GitHub-inrichting; Linux-controle via CI.

## Vervolg

De basis wordt inmiddels in de volledige applicatie gebruikt. De actuele oplevering staat bij [F06](../docs/features/F06-oplevering.md).


## Opleverreview op 2026-11-11

De projectbasis wordt gebruikt door de huidige applicatie. Cargo.lock staat in de repository en de F06-CI bouwt vanaf schone checkout op drie platforms. Ik heb op 2026-11-11 de historische merge bevestigd van F01.

De actuele requirementdekking staat in het [testplan](../docs/testplan.md). De [F06-notitie](F06-oplevering.md) bevat de resterende opleveringsstappen. De oorspronkelijke resultaten hierboven behouden hun datum en scope.

## Mergebevestiging op 2026-11-11

Ik heb de historische merge bevestigd. De feature staat op Klaar. Eerdere open mergevermeldingen in de gedateerde werksessies beschrijven de toenmalige stand.
