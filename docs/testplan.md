# Testplan en requirementdekking

Bijgewerkt op 2026-11-11 voor F06. Deze matrix koppelt iedere requirement aan implementatie en aanwezig bewijs. De code en testdefinities zijn bij de documentatiereview gelezen. De testresultaten hieronder komen uit de eerder vastgelegde controles en de bevestiging van de eigenaar.

## Must-eisen

| Eis | Implementatie en bewijs | Stand |
| --- | --- | --- |
| R01 Configuratie laden en valideren | config::load en ConfigFile::validate. config_tests controleert JSON, onbekende/ontbrekende velden, dubbele namen, timeouts, placeholders, roots en laadfouten. De CI-MCP-basistest start met ieder platformprofiel. | Aangetoond voor de beschreven configuratiecontroles |
| R02 Toevoegen zonder rebuild | Immutable registry wordt bij startup opnieuw geladen. Handmatig bevestigd in F02 en opnieuw door de eigenaar met demo-stap 6. | Door eigenaar bevestigd |
| R03 Ontdekken en opzoeken | ToolRegistry::find, gedeelde input_schema, GET /tools en MCP tools/list. tools_route_returns_tool_metadata_and_input_schema en SDK-ketentest. CI vergelijkt toolnamen met JSON. | Automatische en handmatige dekking |
| R04 Executable met losse argumenten | Command::new met args. starts_with_platform_wrapper_and_waits_for_success en dynamic_argument_preserves_spaces_quotes_and_shell_characters. Demo gebruikt Git en rg. | Automatische en handmatige dekking |
| R05 Uitvoeringsresultaten | ExecutionResult en adaptermapping. converts_core_result_to_http_json, returns_result_for_unsuccessful_exit_code en process_results_preserve_fields_and_indicate_failure. | Automatische en handmatige dekking |
| R06 Timeout, annulering en opruimen | stops_subprocess_on_timeout, stops_subprocess_on_cancellation, stops_subprocess_on_shutdown, drop/abort-scenario’s en protocol_cancellation_stops_process_and_preserves_capacity. Actieve REST/MCP-shutdowntests controleren child en subprocess. Demo toont timeout en herstel. | Rust-suite op drie platforms en bevestigde demo |
| R07 Uitvoer, requestgrootte en capaciteit | limits_stdout_and_stderr_with_shared_budget, rejects_output_above_limit, REST/MCP-tests voor exact 16 KiB en overschrijding. five_real_http_requests_enforce_limit_and_restore_capacity en rest_and_mcp_share_execution_capacity. | Automatische grensgevallen en capaciteitstests |
| R08 Parametercontrole | validation_tests voor verplicht/optioneel, typen, onbekende velden, NUL, enum en prefixes. invalid_parameters_are_rejected_before_process_start. Demo en CI tonen invalid_parameters. | Automatische en handmatige dekking |
| R09 Dynamische argumenten | build_arguments behoudt afzonderlijke strings. Argument-echohelper controleert spaties, quotes, Unicode en shelltekens. Configuratietests controleren volledige placeholders en optionele argumentregels. | Automatische dekking en gecontroleerde voorbeeldtools |
| R10 Padbeleid | validate_path canonicaliseert en vergelijkt componenten. validation_tests controleert roots, vergelijkbare prefixes, traversal, ontbrekende/relatieve paden en Unix-symlinks. HTTP-padtests en MCP-ketentest vergelijken uitvoering en fouten. | Aangetoond voor bestaande absolute paden. Windows-links blijven een testbeperking |
| R11 Echte MCP-client via HTTP | mcp_client_discovers_and_executes_configured_tool, protocol 2025-11-25. F06-CI gebruikt Inspector op drie platforms. Demo gebruikt Claude Code. | Automatische en handmatige clientdekking |
| R12 HTTP-routes | Handmatige statuscontrole uit F01 en routes_tests voor ontdekking, uitvoering en foutmapping. TCP-tests voor capaciteit en actieve shutdown. | Automatische en handmatige dekking |
| R13 Logging en fouten | Geïsoleerde logs_results_without_arguments_or_process_output en logs_dynamic_execution_without_sensitive_values. Controller- en MCP-mappingtests controleren veilige fouten. Demo toont logs. | Automatische en handmatige dekking |
| R14 Windows, Linux en macOS | process_checks gebruikt OS-specifieke API’s. De CI-matrix draait Rust-tests, build en MCP-basistest met drie platformprofielen. | Basisketen op drie platforms aangetoond. Optionele tools hebben eigen afhankelijkheden |
| R15 Gedeelde kern | AppState deelt registry en ExecutionService. De kern gebruikt eigen fouttypen en serde_json. REST/MCP-consistentietest en gemengde capaciteitstest gebruiken dezelfde state. | Codecontrole en ketentests |
| R16 Lokale binding en transportcontroles | main bindt aan 127.0.0.1:3000. transport_* controleert Host, Origin, contentonderhandeling, protocolheaders, sessies en bodygrenzen. | Codecontrole en automatische transporttests |
| R17 Documentatie en demo | README, configuratiehandleiding, API, architectuur, security en demo zijn bijgewerkt. De eigenaar bevestigt de demo. De gezamenlijke README-doorloop volgt hierna. | Deels afgerond, README-doorloop nog open |

De testnamen verwijzen naar [src/tests](../src/tests). Details van de eisen staan in [requirements](requirements.md). R17 wordt afgerond nadat de geplande README-doorloop is bevestigd.

## Overige requirements

| Eis | Prioriteit | Stand |
| --- | --- | --- |
| R18 Remote API-key-authenticatie | Should | Later, met afzonderlijk ontwerp voor authenticatie, autorisatie en transportbeveiliging |
| R19 CI op drie platforms | Should | Aanwezig en als geslaagd vastgelegd, inclusief Inspector-basistest |
| R20 YAML en hot-reload | Could | Later, huidige configuratie is JSON bij startup |
| R21 Dashboard, database, jobqueue, WebSockets en distributed execution | Won’t | Buiten de eerste oplevering |

## Testindeling

Testcode staat in negen Rust-bestanden onder src/tests. cfg(test) en path koppelen de bestanden als submodules aan de oorspronkelijke modules. Daardoor blijven private toegang en super-imports beschikbaar.

| Bestand | Onderwerp |
| --- | --- |
| [config_tests.rs](../src/tests/config_tests.rs) | Configuratie, laden, beleidsvelden en profielvalidatie |
| [validation_tests.rs](../src/tests/validation_tests.rs) | Parameter- en padcontrole, argumentopbouw |
| [execution_tests.rs](../src/tests/execution_tests.rs) | Executor, output, timeout, procesbomen, logging en voorbeeldtools |
| [execution_controller_tests.rs](../src/tests/execution_controller_tests.rs) | Objectvorm en veilige HTTP-foutmapping |
| [execution_model_tests.rs](../src/tests/execution_model_tests.rs) | JSON-resultaatmodel |
| [routes_tests.rs](../src/tests/routes_tests.rs) | REST-router, schema’s, bodygrenzen, dynamische invoer en paden |
| [http_execution_tests.rs](../src/tests/http_execution_tests.rs) | Echte TCP-calls, gedeelde capaciteit en actieve REST/MCP-shutdown |
| [mcp_tests.rs](../src/tests/mcp_tests.rs) | MCP-resultaten, SDK-client, annulering, invoerconsistentie en transport |
| [process_checks.rs](../src/tests/process_checks.rs) | Platformstatus en begrensde PID-controle |

De zeven ignored functies zijn subprocesshelpers die expliciet met --ignored en --exact worden gestart. Het gaat om subprocess_timeout_helper, writes_output_above_limit, argument_echo_helper, http_tool_helper, subprocess_leaf, mcp_echo_helper en mcp_wait_helper.

## Vastgelegde resultaten

### Lokale F05-eindcontrole op Windows, 2026-11-09

| Controle | Resultaat |
| --- | --- |
| cargo fmt --check | Geslaagd |
| cargo clippy --all-targets --locked -- -D warnings | Geslaagd |
| cargo test --locked | 156 passed, 0 failed, 7 ignored, 0 filtered out, 9,77 s |
| SDK-ketentest inclusief padvalidatie | 1 passed, 162 filtered out, 0,61 s, protocol 2025-11-25 |
| Transporttests | 10 passed, 0,07 s |

Deze aantallen horen bij de vastgelegde Windows-run. F05-CI-run #50 was geslaagd op Ubuntu, macOS en Windows.

### F06-platformcontrole, 2026-11-10

| Runner | Profiel | Resultaat | Volledige jobduur |
| --- | --- | --- | --- |
| Windows | config/tools.json | Geslaagd | 4 min 47 s |
| Ubuntu | config/tools.linux.json | Geslaagd | 2 min 5 s |
| macOS | config/tools.macos.json | Geslaagd | 3 min 14 s |

Elke job begint met een schone checkout en draait formatter, Clippy met -D warnings, Rust-tests en build. De MCP-basistest start de gebouwde executable met het platformprofiel, wacht op /status, vergelijkt tools/list met de configuratie, voert git_version uit en controleert invalid_parameters bij git_diff_view met view=--patch. Inspector 2.11.0 gebruikt HTTP met legacy-protocolmodus. De stap ruimt het serverproces op bij afsluiten.

De screenshot van de laatste F06-run vermeldt geen exact runnummer, commit-SHA of testtotalen. De genoemde jobduren komen uit de vastgelegde terugkoppeling. De warning over de Node-runtime van actions/checkout@v4 blijft een CI-onderhoudspunt.

### Demo met Claude Code, 2026-11-11

De eigenaar bevestigt dat het [draaiboek](demo.md) werkt. Daarmee zijn ontdekking, projectonderzoek, validatiefouten, logging, timeout met herstel en het toevoegen van een tool via JSON en herstart bevestigd. De bevestiging is als handmatig resultaat vastgelegd. Exacte duurmetingen, clientversie en gebruikte route voor de ongeldige call zijn daarbij niet gespecificeerd.

## Grenzen van het bewijs

- De CI-basistest ontdekt de volledige profielen en voert de Git-basisketen uit. Afzonderlijke succesaanroepen van alle Python-, rg-, Poppler- en FFmpeg-tools op iedere runner zijn nog niet vastgelegd.
- De persoonlijke Python-tools zijn eerder deels gecontroleerd met tijdelijke gegevens en een gebundelde Python-runtime. De mediatests bevestigden de fout bij ontbrekende ffprobe. De gezondheidscheck gebruikte gesimuleerde HTTP-antwoorden in die testomgeving.
- Procesboomtests gebruiken gecontroleerde helpers en PID-bestanden. De actieve shutdowntests injecteren een signaal in hetzelfde serverpad als Ctrl+C. De MCP-testclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit.
- De Unix-symlinktest draait op Linux/macOS. Een afzonderlijke Windows-symlink/junctiontest ontbreekt.
- De onderbouwing van R15 bevat ook een codecontrole. Modulescheiding is een ontwerpkenmerk dat naast gedragstests wordt beoordeeld.

De productgrenzen staan in [security](security.md). Historische resultaten, fouten en correcties staan in de bijbehorende [notities](../notities/README.md).

## Laatste oplevercontroles

Loop de README gezamenlijk vanaf de vereisten door en noteer eventuele verbeteringen. Controleer daarna de definitieve branchstand met de volgende commando’s en de drieplatform-CI.

~~~sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
~~~

Werk het bewijs en de F06-checklist bij voordat de opleverings-PR naar main wordt afgerond.
