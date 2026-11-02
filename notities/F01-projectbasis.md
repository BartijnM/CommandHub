---
feature: F01
tags:
  - commandhub
  - feature-notitie
---

# F01 — Projectbasis

[Featureplan](../docs/features/F01-projectbasis.md)

## Wat bouw ik en waarom?

F01 controleert de bestaande minimale Axum-applicatie en legt de basis vast voor verdere ontwikkeling. Er is geen nieuwe Rust-functionaliteit toegevoegd. De actuele checklist staat in het featureplan.

## Hoe werkt de code?

`main` initialiseert `tracing`, bindt de Tokio-listener aan `127.0.0.1:3000` en start Axum met een router. De router koppelt `GET /status` aan de statuscontroller. Axum verzorgt de HTTP-afhandeling; Tokio levert de async runtime en Ctrl+C-signaalafhandeling. De graceful-shutdown-hook laat Axum stoppen wanneer Ctrl+C wordt ontvangen. Zie [main.rs](../src/main.rs) en [routes.rs](../src/routes.rs).

## Keuzes en alternatieven

Beschrijf de reden van keuzes en verwijs bij blijvende architectuurbesluiten naar docs/beslissingen/.

## Problemen en oplossingen

Noteer concrete oorzaak, oplossing en hoe de oplossing is gecontroleerd.

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

Link naar het featureplan en de eerstvolgende concrete stap. Houd geen tweede afvinklijst bij.

