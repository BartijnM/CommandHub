---
feature: F07
tags:
  - commandhub
  - feature-notitie
---

# F07 — macOS-ondersteuning

[Featureplan](../docs/features/F07-macos-ondersteuning.md)

## Wat en waarom?

Ik voeg macOS-ondersteuning toe vóór F04. De feature krijgt het nieuwe ID F07; F04–F06 worden niet hernummerd. De hoofdapp had al een Unix-uitvoeringspad, maar de testhelper weigerde andere platforms dan Windows en Linux. De aangeleverde screenshot bevestigt de implementatie inmiddels via geslaagde jobs op macOS, Ubuntu en Windows. Status: Review; review/merge is nog niet bevestigd.

## Ontwerp en Rust-uitleg

`cfg(unix)` in `src/execution.rs` omvat ook macOS. Procesgroepen, de groepsbewaker bij future-drop en de bestaande timeout-/annuleringslogica worden hergebruikt; de uitvoeringskern krijgt geen transportcode of nieuwe platformlaag.

`src/tests/process_checks.rs` gebruikt op macOS `libc::proc_pidinfo` met `PROC_PIDTBSDINFO`. De retourstructuur bevat `pbi_status`: statussen 1–4 zijn actief, SZOMB (5) is gestopt maar mogelijk nog niet gereaped. ESRCH (3) betekent dat het PID verdwenen is. Andere OS-fouten en onbekende statussen blijven fouten. De pure classificatie is ook op Windows en Linux testbaar. Een macOS-specifieke test controleert de eigen PID, ongeldige PID's en een gestart/gestopt `/bin/sleep`-proces. Bestaande subprocess- en TCP-tests gebruiken dezelfde `assert_stopped`-helper.

`#[cfg(target_os = "macos")]` laat Rust alleen de relevante OS-code compileren. `libc` is uitsluitend een macOS-dev-dependency; deze dependency was al transitief opgenomen in Cargo.lock (0.2.191). De lockfile koppelt die bestaande versie nu ook rechtstreeks aan de tests; er zijn geen dependencyversies gewijzigd.

De C-API schrijft naar een buffer. `MaybeUninit<libc::proc_bsdinfo>` reserveert daarvoor correct uitgelijnd geheugen. De `unsafe`-aanroep krijgt exact de buffergrootte. Pas als de API die volledige grootte teruggeeft, wordt `assume_init()` gebruikt; een gedeeltelijk gevuld resultaat wordt afgewezen. Deze controle gebruikt geen shell of extern statusprogramma.

`config/tools.macos.json` bevat uitsluitend vaste argumenten. `ping` gebruikt daar `-c 4`; de bestaande standaardconfiguratie bevat een Windows-pingvoorbeeld met `-n 4`. De macOS-configuratie wordt expliciet gekozen met `COMMANDHUB_CONFIG`; er is geen stilzwijgende platformdetectie. Een nieuwe test valideert het bestand op alle platforms; de macOS-job voert `/bin/echo` uit via dezelfde registry en executor als andere tools. De overige voorbeeldtools zijn niet afzonderlijk getest.

## Bronnen

- [Rust libc: Apple-types, constantes en proc_pidinfo](https://github.com/rust-lang/libc/blob/main/src/unix/bsd/apple/mod.rs).
- [Apple: procesinformatie en retourwaarden](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/proc_info.c).
- [Apple: libproc-wrapper](https://github.com/apple-oss-distributions/xnu/blob/main/libsyscall/wrappers/libproc/libproc.c).

## Werksessie — 2026-11-03

- Opdracht: macOS toevoegen, daarna documentatie bijwerken; tests via GitHub Actions. Geen Git-commando's uitvoeren.
- Toegevoegd: macOS-statuscontrole, classificatietests, native levenscyclustest, voorbeeldconfiguratie en uitvoeringstest.
- CI: `macos-latest` toegevoegd naast Ubuntu en Windows. Dezelfde formatter, Clippy (`-D warnings`), tests en build gelden voor alle drie; `fail-fast: false` en de jobtimeout blijven behouden.
- Gecontroleerd: `cargo fmt --check` geslaagd na formatteren. Cargo metadata met `--locked --offline --no-deps` accepteert het manifest en toont de macOS-only dev-dependency. Dit compileert of test de macOS-code niet.
- Niet uitgevoerd: Clippy, tests en build; op verzoek gebeurt de verificatie via Actions. Lokaal is geen macOS-omgeving beschikbaar.
- Nog geen testtotalen of geslaagde macOS-acceptatie geclaimd. De eerdere groene CI van refactorcommit 105e69b is geen bewijs voor F07.
- Geen Git-commando's, commits, pushes of merges uitgevoerd.

## Volgende stap

Commit en push de documentatie-update en controleer de CI van de laatste branchstand. Daarna PR-review en merge naar develop; vervolgens F04. De actuele checklist staat uitsluitend in het featureplan.

## CI-bevestiging — 2026-11-03

De werking van F07 is bevestigd met een GitHub Actions-screenshot met drie groene checks:

| Job | Resultaat | Zichtbare duur |
| --- | --- | --- |
| Checks (macos-latest) | Geslaagd | 56 s |
| Checks (ubuntu-latest) | Geslaagd | 44 s |
| Checks (windows-latest) | Geslaagd | 1 m 54 s |

De huidige workflow voert op elk platform cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo build --locked uit. Daarmee zijn de platformacceptatiecriteria onderbouwd, inclusief de macOS-status-/uitvoeringstests en bestaande procesboomscenario’s. Exacte testtotalen, commit-ID, runnummer en totale runduur zijn niet zichtbaar in deze screenshot en worden niet ingevuld.

Er staan drie waarschuwingen en twee notices bij de run. Zichtbaar zijn Node.js 20-verouderingsmeldingen bij actions/checkout@v4; de checks zijn groen. De notices zijn niet volledig zichtbaar en worden niet nader geïnterpreteerd. Action-onderhoud blijft een afzonderlijk punt.

Alleen documentatie aangepast op basis van dit bewijs; geen tests opnieuw uitgevoerd en geen Git-commando’s uitgevoerd. Status naar Review. PR-review/merge is niet zichtbaar en blijft open. Deze documentatie-update volgt op de getoonde run; controleer na push de CI van de laatste branchstand.

