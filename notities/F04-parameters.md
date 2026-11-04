---
feature: F04
tags:
  - commandhub
  - feature-notitie
---

# F04 — Dynamische parameters en padbeleid

[Featureplan](../docs/features/F04-parameters.md)

## Wat bouw ik en waarom?

F04 maakt gecontroleerde dynamische invoer mogelijk zonder de client een executable te laten kiezen. De eerste keten werkt volgens lokale terugkoppeling van de eigenaar via de gedeelde service en HTTP-router. De feature is nog in uitvoering; eindcontroles en aanvullende veiligheidsverificatie staan open.

## Hoe werkt de code?

- [Configuratie](../src/config.rs): ParameterDefinition heeft naast name, type en required ook allowed_values: Option<Vec<String>> en allowed_roots: Option<Vec<PathBuf>>. Ontbrekende optionele JSON-velden worden None; Rust-structliterals moeten de velden expliciet invullen.
- [Validatie](../src/validation.rs): validate_parameters weigert onbekende velden, ontbrekende verplichte parameters, niet-stringwaarden en NUL. build_arguments valideert eerst en vervangt vervolgens volledige placeholders door afzonderlijke Strings. Spaties worden niet gesplitst; vaste argumenten en volgorde blijven behouden. Een ontbrekende optionele placeholder wordt weggelaten, een lege string blijft een argument.
- Padcontrole gebeurt in build_arguments voor gebruikte placeholders met allowed_roots. validate_path vereist een bestaand absoluut pad, canonicaliseert pad en roots en vergelijkt componenten met Path::starts_with. Het gecanonicaliseerde pad wordt doorgegeven. Niet-UTF-8-paden worden afgewezen omdat de argumentvector momenteel Strings bevat.
- [Uitvoering](../src/execution.rs): execute_with_parameters gebruikt execute_logged met Some(input); de parameterloze methode gebruikt None. De wrapper zoekt één keer de tool op en logt dezelfde metadata. execute_with_parameters_inner bouwt argumenten vóór processtart en vertaalt fouten naar InvalidParameters. execute_tool_inner deelt timeout, outputbudget, capaciteit, annulering en shutdown. Argumentenslices lopen via start_process en create_wrapped_command naar Command::args; command komt uit de registry.
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

## Verificatie — 2026-11-04, Windows

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

Begin met HTTP-padtests. Daarna gerichte loggingtests voor dynamische invoer en ontdekking: GET /tools mist nog enum voor allowed_values en additionalProperties: false. Review toolbeleid, ongebruikte padparameters, rootwijzigingen en tijdelijke testdirectory-opruiming; controleer ontbrekende root en aanvullende platformgrenzen. Voer daarna eindcontroles en drieplatform-CI uit. Volledige suite na de laatste routetests opnieuw uitvoeren. Zie het featureplan voor de centrale checklist.
