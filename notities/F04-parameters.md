---
feature: F04
tags:
  - commandhub
  - feature-notitie
---

# F04 — Dynamische parameters en padbeleid

[Featureplan](../docs/features/F04-parameters.md)

## Wat bouw ik en waarom?

F04 maakt gecontroleerde dynamische invoer mogelijk zonder de client een executable te laten kiezen. De implementatie en lokale Windows-eindcontroles zijn afgerond. Status: Review, drieplatform-CI geslaagd; PR-review/merge is nog niet bevestigd. De actuele verificatie staat in de laatste werksessie hieronder.

## Hoe werkt de code?

- [Configuratie](../src/config.rs): ParameterDefinition heeft naast name, type en required ook allowed_values: Option<Vec<String>> en allowed_roots: Option<Vec<PathBuf>>. Ontbrekende optionele JSON-velden worden None; Rust-structliterals moeten de velden expliciet invullen.
- [Validatie](../src/validation.rs): validate_parameters weigert onbekende velden, ontbrekende verplichte parameters, niet-stringwaarden en NUL. build_arguments valideert eerst en vervangt vervolgens volledige placeholders door afzonderlijke Strings. Spaties worden niet gesplitst; vaste argumenten en volgorde blijven behouden. Een ontbrekende optionele placeholder wordt weggelaten, een lege string blijft een argument.
- Padcontrole gebeurt in build_arguments voor gebruikte placeholders met allowed_roots. validate_path vereist een bestaand absoluut pad, canonicaliseert pad en roots en vergelijkt componenten met Path::starts_with. Het gecanonicaliseerde pad wordt doorgegeven. Niet-UTF-8-paden worden afgewezen omdat de argumentvector momenteel Strings bevat.
- [Uitvoering](../src/execution.rs): execute_with_parameters delegeert naar execute_with_parameters_with_cancellation. Ook parameterloze tools gebruiken dit pad met een leeg invoerobject. De oude productie-ingangen en ParametersNotSupported zijn verwijderd; compatibiliteitshelpers bestaan alleen in de testmodule en delegeren naar het productiepad. De wrapper zoekt één keer de tool op en logt metadata. execute_with_parameters_inner bouwt argumenten vóór processtart en vertaalt fouten naar InvalidParameters. execute_tool_inner deelt timeout, outputbudget, capaciteit, annulering en shutdown. Argumentenslices lopen via start_process en create_wrapped_command naar Command::args; command komt uit de registry.
- [HTTP-controller](../src/controllers/execution_controller.rs): input_object leent een JSON-object; andere vormen geven 400 invalid_input. De handler roept execute_with_parameters aan. Validatiefouten geven 400 invalid_parameters met een vaste tekst. JSON/contenttype- en bodylimietfouten blijven invalid_json en request_too_large. De oude controle op uitsluitend een leeg object is verwijderd.
- [Foutresponse](../src/models/execution.rs): Debug is toegevoegd naast Serialize zodat tests Result::expect kunnen gebruiken. Dit verandert de JSON niet en voegt geen logging toe.
- Tests staan in src/tests/ en worden via cfg(test), path en mod tests gekoppeld; geen extra mod tests-wrapper in het testbestand.

Rust: & leent invoer zonder kopiëren; Option beschrijft aanwezigheid; Result en ? geven fouten door; map_err vertaalt fouttypen. Een slice &[String] leent een argumentenreeks. Closures bij any/find controleren elementen. De lokale argumentvector blijft bestaan tijdens de afgewachte uitvoering.

## Keuzes en grenzen

Alleen type string is ondersteund. allowed_values vergelijkt exact, inclusief hoofdletters en spaties. Een ingestelde lijst moet niet leeg zijn en mag geen NUL bevatten; een lege string in de lijst mag wel. Expliciet geconfigureerde waarden mogen optieachtige prefixes bevatten.

Zonder allowed_values worden waarden die met - beginnen afgewezen. Zonder allowed_values én zonder allowed_roots worden ook / en @ als eerste teken afgewezen. Dit beperkt bekende opties/argumentbestanden, maar de betekenis van argumenten blijft toolafhankelijk en vraagt beheerdersreview.

Maximaal één optionele placeholder mag voorkomen, uitsluitend achteraan. Vaste argumenten met - of / zijn bij zo’n tool verboden, inclusief -- en absolute Unix-paden. Dit is bewust conservatief; groepen van optionele optie en waarde zijn niet ontworpen. Gebruik daarvoor voorlopig een verplichte parameter.

Roots moeten absolute, bestaande directories zijn; lege lijsten en onoplosbare roots worden afgewezen. Clientpaden moeten bestaan en absoluut zijn. Symlinks worden gevolgd: het uiteindelijke doel moet binnen een root vallen. Ook de root zelf is toegestaan. Canonicalisatie is geen sandbox en voorkomt geen race door bestandssysteemwijzigingen na controle. Roots worden opnieuw opgelost tijdens argumentopbouw en zijn nog niet als onveranderlijke canonieke roots opgeslagen.

## Problemen en oplossingen

- Extra mod tests-wrapper veroorzaakte een verkeerde super-scope; de wrapper is uit het losse testbestand verwijderd.
- Nieuwe velden vereisten aanpassing van bestaande ParameterDefinition-structliterals.
- Bij de refactor moest ook een directe start_process-aanroep in execution_tests.rs argumenten meekrijgen.
- Het optionele-argumentfragment stond eerst in de parameterlus; het is verplaatst naar de argumentlus waar argument bestaat.
- Meerdere optionele placeholders achteraan bleken alsnog positioneel te kunnen verschuiven. De regel is aangescherpt tot één optionele placeholder.
- Result::expect in de objecttest vereiste Debug op ExecutionErrorResponse; derive(Debug) toegevoegd.

## Historische tussenstand — 2026-11-04, vóór eindcontrole

De eigenaar heeft de commando’s uitgevoerd en succes gemeld; de assistent heeft geen commando’s uitgevoerd. Er zijn geen volledige logs van de eindstand aangeleverd.

| Controle | Terugkoppeling en beperking |
| --- | --- |
| cargo check --locked tijdens kleine stappen | Geslaagd gemeld; tussentijds waren ongebruikte functies te verwachten |
| cargo test --locked validation::tests:: | Geslaagd gemeld: verplicht/optioneel, typen, onbekend, Unicode/NUL, argumenten, waardelijsten, prefixes en paden |
| cargo test --locked config::tests:: | Geslaagd gemeld: schema, waardelijsten, roots en optionele argumentvolgorde/combinaties |
| cargo test --locked execution::tests:: | Geslaagd gemeld: dynamische rustc-uitvoering, afwijzing vóór start, registry en shutdown |
| cargo test --locked controllers::execution_controller::tests:: | Geslaagd gemeld: objectvorm en vaste HTTP-foutmapping |
| cargo test --locked | Herhaald geslaagd gemeld; één tussenstand 107 tests, geen actueel eindtotaal bekend |
| cargo test --locked routes::tests:: | Laatste stap geslaagd gemeld: dynamische uitvoering, 400-fouten en 16 KiB/16 KiB + 1 |
| cargo fmt --check en cargo clippy --all-targets --locked -- -D warnings | Nog niet bevestigd voor F04-eindstand |
| F04-CI Windows/Linux/macOS | Nog niet aangeleverd; Unix-symlinktest lokaal niet uitgevoerd |

De ongeldige-invoertest gebruikt een niet-bestaande executable en verwacht InvalidParameters in plaats van ProcessStartFailed. Capaciteit blijft beschikbaar. Padtests controleren binnen/buiten root, .., relatieve en onoplosbare paden en data-extra naast data. De Unix-only test controleert symlinks naar binnen en buiten. Routertests gebruiken Axum oneshot en starten werkelijk rustc bij geldige invoer; ze zijn geen nieuwe TCP-tests.

## Werksessie — 2026-11-04

Eigenaar voert code zelf in na korte uitleg; tests mogen samen worden gegeven. Geen broncode door de assistent aangepast. Op verzoek aan het einde van de sessie is de documentatie bijgewerkt. Geen Git-acties uitgevoerd en geen featureafronding of merge geclaimd.

## Vervolg

Eigenaar probeert de tools uit docs/F04-handmatig-testen.md. Daarna GitHub Actions op Windows/Linux/macOS beoordelen, met name de Unix-symlinktest. PR-review en merge naar develop; daarna pas Klaar. De centrale checklist staat in het featureplan.

## Werksessie — 2026-11-04, afronding na hervatten

De eigenaar gaf expliciet toestemming om de resterende code en documentatie zelf af te ronden en lokale controles uit te voeren. Linux/macOS worden als laatste via Actions getest. Geen commit/push/merge/publicatie uitgevoerd.

Toegevoegd/bevestigd sinds de eerdere tussenstand:

- HTTP-padtests: geldig pad via git -C, buiten root, .. naar buiten, relatief en onoplosbaar pad; vaste 400-foutresponses.
- GET /tools toont enum en additionalProperties: false, met routetests.
- Dynamische loggingtest in een geïsoleerd proces: succes, niet-nul exit en validatiefout, zonder clientwaarden of output in logs.
- Iedere parameter moet minstens één volledige placeholder hebben; ongebruikte padparameters worden bij startup afgewezen. Configuratiefixtures aangepast en ontbrekende root getest.
- Clippy-correcties: goedkope vaste fout via ok_or, let chain voor allowed_values. Het dubbele productiepad is verwijderd. Bestaande timeout-, annulering-, shutdown-, drop- en procesboomtests lopen via dezelfde validatie als HTTP.
- De oude test rejects_tool_with_parameters is verwijderd: dynamische parameters zijn nu juist toegestaan. Ontbrekende/verkeerde parameters hebben al gerichte vervangende tests. Dit verklaart één test minder vóór de nieuwe eindtests.
- Nieuwe childhelper argument_echo_helper controleert werkelijk één dynamisch argument met spaties, quotes, Unicode en shelltekens. Dit voegt één ignored helper toe, geen overgeslagen acceptatietest.
- config/tools.json bevat rust_details, git_recent en git_diff_view. Alle drie zijn gevalideerd en werkelijk uitgevoerd in een test. Alleen vaste keuzelijsten worden geaccepteerd; git diff gebruikt --no-ext-diff en --no-textconv. Geen dynamische executable, Git-revisie of ongecontroleerde optie. Git-repository en omgeving zijn door de beheerder vertrouwd.
- Tijdelijke padfixtures maken een eigen directory via create_dir met testnaam en proces-ID. Opruiming verwijdert uitsluitend die testdirectory; symlinks worden niet naar hun doelen gevolgd door remove_dir_all. De Unix-test blijft voor CI. Een eerdere crash kan een fixture achterlaten; create_dir faalt dan veilig in plaats van een bestaande directory te hergebruiken.

### Uitgevoerde Windows-eindcontroles

| Controle | Resultaat |
| --- | --- |
| cargo fmt --check | Geslaagd |
| cargo clippy --all-targets --locked -- -D warnings | Geslaagd |
| cargo test --locked | 138 passed, 0 failed, 5 ignored, 0 filtered; suite 2,83 s |

Deze controles zijn door de assistent daadwerkelijk uitgevoerd. De laatste vormcorrectie in input_object is meegenomen in de definitieve controle vóór overdracht. De vijf ignored functies worden expliciet als proceshelpers gebruikt. Linux/macOS en de Unix-symlinktest zijn niet lokaal uitgevoerd. De bestaande workflow bevat formatter, Clippy, tests en build op alle drie platforms.

### Bewuste beperkingen

Bestaande paden zijn vereist; nieuw aan te maken bestanden zijn niet ondersteund. Geen afzonderlijke Windows-symlink/junctiontest. Niet-UTF-8-paden worden afgewezen. Roots worden per uitvoering opnieuw gecanonicaliseerd; beheerder moet roots, bovenliggende directories en repositoryconfiguratie beheren. Races tussen controle en gebruik zijn niet opgelost door canonicalisatie. Prefixfilters zijn geen algemene sandbox voor iedere tool; aangepaste tools vereisen beoordeling van hun argumentbetekenis. De voorbeeldtools gebruiken daarom expliciete keuzes. De limiet van één optioneel argument zonder vaste opties blijft bewust conservatief.

## Correctie na verplaatsen van voorbeeldtools

De eigenaar heeft de drie F04-tools naar config/tools.json verplaatst en het aparte bestand verwijderd. De include_str!-verwijzingen in de configuratie- en uitvoeringstests verwezen nog naar het verwijderde bestand, waardoor CI tijdens compilatie stopte. Beide tests lezen nu de standaardconfiguratie; de uitvoeringstest voert uitsluitend de drie genoemde platformonafhankelijke voorbeeldtools uit, niet het Windows-pingvoorbeeld. Documentatie en startinstructies volgen eveneens de standaardconfiguratie. Lokale verificatie volgt hieronder; Linux/macOS opnieuw via Actions.

Verificatie van de correctie op Windows: cargo fmt toegepast en cargo fmt --check geslaagd; cargo clippy --all-targets --locked -- -D warnings geslaagd; cargo test --locked: 138 passed, 0 failed, 5 ignored, suite 2,25 s. Geen commit of push uitgevoerd.

## CI-overdracht — 2026-11-04, commit 2d02834

De eigenaar leverde screenshots aan: Ubuntu geslaagd in 41 s en Windows in 1 m 30 s. macOS eindigde na 15 m 2 s zonder dat een hosted runner de job had opgepakt. Annotations: Internal server error; The job was not acquired by Runner of type hosted even after multiple attempts. Notices melden capaciteitstekort bij macOS arm64-runners. Er is geen macOS-test uitgevoerd en geen codefout aangetoond. Exact runnummer/URL en testtotalen zijn niet zichtbaar.

De assistent controleerde branch feature/F04-parameters en HEAD 2d02834 lokaal. De werkmap was schoon. Testverwijzingen gebruiken config/tools.json en de workflow bevat Ubuntu/Windows/macOS met formatter, Clippy, tests en build. Geen broncode gewijzigd of tests herhaald tijdens deze documentatieoverdracht. De groene Ubuntu-job levert F04-bewijs voor de Unix-symlinktest; macOS-verificatie ontbreekt nog.

Plan voor 2026-11-05: Re-run failed jobs, macOS-resultaat beoordelen. Bij groen bewijs en checklist actualiseren, PR-review en merge naar develop. Bij opnieuw geen runner: later herstarten; geen broncodecorrectie zonder concrete code-/testfout. Geen commit/push/merge namens de eigenaar uitgevoerd.

## CI-afronding — 2026-11-04

De aangeleverde screenshot van run #39 voor commit 1fb8d71 op feature/F04-parameters toont Success: macOS 57 s, Ubuntu 40 s en Windows 1 m 50 s; totale duur 1 m 54 s. De workflow voert formatter, Clippy, tests en build uit. Hiermee is ook de macOS-platformverificatie bevestigd; de eerdere runnercapaciteitsmelding is geen open blokkade meer. Exacte CI-testtotalen zijn niet zichtbaar. De eigenaar meldt dat alles werkt. Alleen PR-review/merge en branchafsluiting resten; status pas Klaar na bevestigde merge. Bij deze documentatie-update zijn geen broncode gewijzigd, tests herhaald of Git-mutaties uitgevoerd.
