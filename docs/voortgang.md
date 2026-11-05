# Huidige voortgang

Bijgewerkt: 2026-11-05, einde werksessie.

## Actief

[F05 — MCP-integratie](features/F05-mcp-integratie.md) is in uitvoering. Ik bouw en controleer de feature in kleine stappen. De vorige overdracht vermeldde F04 op Review met geslaagde lokale controles en drieplatform-CI; merge is in deze werksessie niet afzonderlijk bevestigd.

## Aanwezig

/mcp biedt Streamable HTTP via rmcp 3.6.0 naast REST. Beide adapters delen registry, inputschemaopbouw, validatie en ExecutionService. MCP vertaalt procesresultaten en kernfouten naar gestructureerde en tekstuele toolresultaten. Host/Origin-controles en een bodylimiet van 16 KiB zijn ingesteld. Legacy-sessiemodus is ingeschakeld.

Een CancellationToken verbindt de MCP-service met het bestaande serverafsluitpad. De MCP-requestcontext is via select! en oneshot aangesloten op execute_with_parameters_with_cancellation. Deze requestannulering compileert, maar de protocolannuleringsketen is nog niet getest.

## Verificatie

Op Windows slagen de eerder uitgevoerde 18 REST-routetests en 9 MCP-mapping-/transporttests. Daarna slagen de twee bestaande HTTP-integratiescenario's na de shutdownwijziging en een nieuwe test voor het sluiten van een actieve MCP-stream na tokenannulering.

Een echte SDK-client ontdekt en voert via TCP een geconfigureerde testtool uit. Metadata/schema, veilige afwijzing van extra parameters, onbekende tool en een geldige vervolgaanroep zijn gecontroleerd. De ketentest gebruikt een testbinary als tool en een eigen Axum-server op een vrije loopbackpoort. Hij bewijst nog niet het productieafsluitpad met actieve MCP-tools.

Laatste controle: cargo check --tests --locked geslaagd na toevoegen mcp_wait_helper. Deze helper is alleen gecompileerd. Geen volledige suite, formatter, Clippy of platform-CI voor de huidige eindstand bevestigd. Details en historische testtotalen staan in de [F05-notitie](../notities/F05-mcp-integratie.md).

## Grenzen

Protocolannulering, procesopruiming/capaciteitsherstel via MCP, dynamische MCP-parameters, gedeelde REST/MCP-belasting en productie-shutdown met een actieve MCP-call vragen nog bewijs. De onderhandelde protocolversie van de SDK-ketentest is nog niet expliciet vastgelegd. Geen remote deployment, authenticatie of sandbox toegevoegd.

## Exact stoppunt

src/tests/mcp_tests.rs bevat nu mcp_wait_helper: een ignored proceshelper die zijn PID schrijft naar het opgegeven commandhub-mcp-pid-file= pad en maximaal 30 seconden wacht. De bijbehorende annuleringstest is nog niet geschreven.

## Volgende stap

Schrijf de protocolannuleringstest: start de helper via een MCP-toolcall, wacht begrensd op een geldig PID-bestand, stuur een expliciet annuleringsbericht via de SDK-client en controleer procesbeëindiging met process_checks::assert_stopped. Controleer vooraf het SDK-gedrag bij annulering. Daarna resterende keten-/grenstests, volledige eindcontroles en CI. Geen Git-status, commit, push of merge geverifieerd tijdens deze documentatie-update.
