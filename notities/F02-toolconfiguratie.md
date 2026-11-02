---
feature: F02
tags:
  - commandhub
  - feature-notitie
---

# F02 — Configuratie laden en tools ontdekken

[Featureplan](../docs/features/F02-toolconfiguratie.md)

## Wat bouw ik en waarom?

Ik laad bij startup een beheerdersconfiguratie met toolbeschrijvingen en maak die beschikbaar voor ontdekking via `GET /tools`. Nieuwe tools moeten door configuratie en een herstart toegevoegd kunnen worden, zonder de applicatie opnieuw te compileren. De actuele checklist staat in het [featureplan](../docs/features/F02-toolconfiguratie.md).

## Hoe werkt de code?

`ConfigFile`, `ToolDefinition` en `ParameterDefinition` beschrijven de JSON-structuur. Serde zet de JSON om naar deze Rust-types. `Option<u64>` maakt een timeout optioneel; de grenscontrole staat in `ConfigFile::validate`. De loader leest standaard `config/tools.json` (of het pad uit `COMMANDHUB_CONFIG`), parseert de inhoud en valideert die vóór startup verdergaat. Zie [src/config.rs](../src/config.rs) en de aanroep bij startup in [src/main.rs](../src/main.rs).

De gevalideerde configuratie wordt verplaatst naar de immutable [ToolRegistry](../src/registry.rs), gedeeld met Axum via `Arc`, en door `GET /tools` omgezet naar toolmetadata met een JSON Schema-inputschema. De route publiceert niet het command of de vaste argumenten. De routetest controleert de HTTP-status en de volledige responsevorm.

## Keuzes en alternatieven

- Start met `string`-parameters en een `required`-boolean; complexere typen zijn nog niet nodig voor de eerste scope.
- Vervang placeholders alleen wanneer ze een volledig argument innemen. Zo blijven executable en argumenten gescheiden en wordt geen shellcommandoregel opgebouwd.
- De beheerder bepaalt `command` en vaste argumenten. `command` mag een absoluut pad zijn of een naam die via de server-`PATH` wordt gevonden; clientinvoer kan geen executable instellen.
- Wijs onbekende configuratievelden af om configuratiefouten vroeg zichtbaar te maken.
- Een tool mag `timeout` weglaten; dan geldt de standaard van 10.000 ms (10 seconden). Een ingestelde timeout moet tussen 1 en 60.000 ms (60 seconden) liggen.
- De casus geeft geen volledig executablepadbeleid of uitvoeringslimieten. Die blijven expliciete ontwerpkeuzes; zie [API en configuratie](../docs/api-en-configuratie.md).

## Problemen en oplossingen

Noteer concrete oorzaak, oplossing en hoe de oplossing is gecontroleerd.

## Verificatie

De Cargo-controles zijn uitgevoerd: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` en `cargo test --locked` slagen. De test-run bevat 14 geslaagde tests. De gerichte laadfouttests controleren ontbrekende bestanden, kapotte JSON en ongeldige configuratie.

Handmatig is bevestigd dat een dubbele toolnaam startup verhindert met de diagnose `Ongeldige configuratie in bestand config/tools.json: Dubbele toolnaam: git_version`. Ook is bevestigd dat een aangepaste toolconfiguratie na herstart zichtbaar wordt via `GET /tools`, zonder rebuild. De response laat command en vaste argumenten weg. Deze controles zijn op Windows uitgevoerd; Linux en de GitHub Actions-workflow zijn hiermee niet geverifieerd.

## Werksessies

### 2026-10-22

- Doel: de casusvoorbeelden vergelijken met de open schema-vragen voor F02.
- Gewijzigd: het API-contract, het F02-plan, configuratievoorbeelden en deze notitie verduidelijken welke keuzes uit de casus volgen en welke nog vastgesteld moeten worden.
- Geleerd: de casus toont `string`-parameters, placeholders als argumenten en een timeoutvoorbeeld, maar bepaalt geen standaard- of maximumwaarden en geen executablepadbeleid.
- Gecontroleerd: huidige implementatie bekeken; alleen `/status` bestaat. Er zijn geen F02-tests of configuratietypen.
- Open vragen: executablepadbeleid; uitvoeringslimieten voor F03.

### 2026-10-22 — timeout- en configuratieafspraken

- Besluit: standaardtimeout 10 seconden; maximaal configureerbaar 60 seconden. Timeoutwaarden in JSON zijn milliseconden.
- Besluit: het configuratiebestand biedt dezelfde soort toolinstellingen als het casusvoorbeeld: `name`, `description`, `command`, `arguments`, `parameters` en optioneel `timeout`. Parameters bevatten `name`, `type` (`string`) en `required`.
- Gewijzigd: het configuratievoorbeeld laat zowel een tool met standaardtimeout als een tool met expliciete timeout zien.
- Nog niet gecontroleerd: JSON wordt nog niet door de applicatie gelezen; de documentatie beschrijft de afspraak, geen werkende functionaliteit.

### 2026-10-22 — executableselectie

- Besluit: de beheerder mag in `command` een absoluut pad of een executable-naam gebruiken die via de `PATH` van het serverproces wordt gevonden. Clients kunnen deze waarde niet kiezen of aanpassen.
- Open punt voor F03: toegestane executabletypen en het gedrag voor Windows `.bat`/`.cmd`.
- Nog niet gecontroleerd: dit is een documentatiekeuze; procesuitvoering is niet geïmplementeerd.

### 2026-10-22 — configuratiepad

- Besluit: standaard leest de server `config/tools.json`, relatief aan de huidige werkdirectory. De omgevingsvariabele `COMMANDHUB_CONFIG` kan een ander bestandspad opgeven.
- Nog niet gecontroleerd: deze locatiekeuze is nog niet in code geïmplementeerd.

### 2026-10-22 — configuratie laden en valideren

- Gebouwd: Serde-types voor configuratie, tools, parameters en het ondersteunde parametertype. `load()` leest de standaardlocatie of `COMMANDHUB_CONFIG`, deserialiseert JSON en valideert de configuratie; startup logt alleen het aantal tools.
- Gebouwd: immutable registry, gedeeld via Axum-state, en `GET /tools` met naam, beschrijving en JSON Schema-inputschema. De route laat executable en vaste argumenten weg.
- Gerapporteerde testresultaten: 11 slagen. Daaronder deserialisatie, onbekende/ontbrekende velden, tool- en parameternaamvalidatie, timeoutgrenzen, placeholders en de response van `GET /tools`.
- Gerapporteerde handmatige controle: server start zonder errors en `/tools` werkt; de response is in de browser bekeken.
- Nog open: gerichte tests voor configuratiebestand-laadfouten en verificatie dat een nieuw geconfigureerde tool na herstart zichtbaar wordt zonder rebuild. Volledige formatter-, Clippy- en Cargo-testresultaten zijn niet in deze sessie opnieuw uitgevoerd.

## Vervolg

F02 is gemerged en staat op Klaar; de merge is op 2026-10-26 bevestigd. De volgende implementatie is F03; zie het [featureplan](../docs/features/F03-procesuitvoering.md).

### 2026-10-23 — F02 afronden

- Bevestigd: ongeldige configuratie met dubbele toolnaam stopt startup en geeft een foutmelding met bestandspad en oorzaak.
- Bevestigd: een aangepaste toolconfiguratie wordt na herstart zichtbaar op `GET /tools`, zonder de executable opnieuw te bouwen.
- Gecontroleerd: formatter, Clippy en 14 tests slagen op Windows.
- Status: acceptatiecriteria aangetoond; feature staat ter review. Merge is nog niet uitgevoerd.

### 2026-10-26 — Merge bevestigd

- Bevestigd: F02 is als gemerged gemeld; documentatiestatus bijgewerkt naar Klaar.
- Vervolg: F03 is gestart; de uitvoeringsafspraken staan in het [F03-featureplan](../docs/features/F03-procesuitvoering.md).
