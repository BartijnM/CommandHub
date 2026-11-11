# Security en beperkingen

Bijgewerkt op 2026-11-11 voor de lokale oplevering van CommandHub. De maatregelen hieronder zijn vergeleken met configuratie, validatie, procesuitvoering en transportcode. Het beschikbare bewijs staat in het [testplan](testplan.md).

## Vertrouwensgrenzen

De beheerder bepaalt executables, vaste argumenten, scripts, serverwerkmap, omgeving, PATH en toegestane gegevenslocaties. Clientinvoer wordt vóór processtart gevalideerd. Procesoutput is onbetrouwbare inhoud. Een MCP-client en het taalmodel moeten toolresultaten als gegevens behandelen.

De listener gebruikt 127.0.0.1:3000. De lokale endpoints hebben de rechten van het serveraccount. Beheerders moeten uitsluitend tools en bestanden beschikbaar stellen die lokale clients mogen gebruiken.

## Configuratie en argumenten

- Serde wijst onbekende velden af. Startupvalidatie controleert unieke niet-lege tool- en parameternamen, een niet-leeg command, placeholders, roots, keuzelijsten en timeoutgrenzen.
- Het programma en de argumentvector worden afzonderlijk aan de proces-API doorgegeven. Een placeholder neemt één volledig argument in. De client kan waarden voor geconfigureerde parameters leveren.
- Scripts worden via een expliciet geconfigureerde interpreter gestart. Stdin is gesloten. Het proces erft omgeving en werkmap van de server.
- Alleen strings zijn ondersteund. Ontbrekende verplichte waarden, onbekende parameters, ongeldige typen en NUL worden afgewezen. allowed_values vergelijkt exact. Vrije strings hebben beperkingen voor -, / en @ als eerste teken volgens het [API-contract](api-en-configuratie.md).
- De betekenis van argumenten blijft afhankelijk van de executable. Beoordeel per tool opties, bestandsnamen, repositoryconfiguratie en mogelijke neveneffecten. De meegeleverde Git-diffconfiguratie schakelt externe diff- en textconv-programma’s uit.

## Begrenzing en procesbeheer

| Grens | Instelling |
| --- | --- |
| Timeout | Standaard 10.000 ms, configureerbaar van 1 tot 60.000 ms |
| Procesoutput | 1 MiB gecombineerde stdout/stderr per uitvoering |
| Requestbody | 16 KiB op de REST-POST-route en MCP-service |
| Gelijktijdige uitvoeringen | Vier gedeelde plekken voor REST en MCP, directe afwijzing bij volle capaciteit |

JobObject op Windows en ProcessGroup op Unix beheren de geconfigureerde processen. Timeout, outputoverschrijding, expliciete annulering en shutdown activeren de opruimroute. Future-drop gebruikt KillOnDrop en de Unix-groepsbewaker. PID-tests controleren de beschreven procesboomscenario’s op de ondersteunde platforms.

De ingestelde timeout bewaakt de asynchrone uitvoering. OS-processtart en opruiming kunnen aanvullende tijd kosten. Een opruimfout kan als execution_failed terugkomen. shutdown wacht op vrijgave van uitvoeringspermits. Processtatuschecks leveren afzonderlijk bewijs van beëindiging in de tests.

## Padbeleid

De Rust-kern vereist bij allowed_roots bestaande absolute paden en roots. Canonicalisatie lost symlinks op. Componentvergelijking voorkomt dat een vergelijkbare directoryprefix als toegestane root wordt gezien. Het opgeloste pad wordt aan het proces doorgegeven. Interne roots worden weggelaten uit de ontdekschema’s.

De Python-bestandstools hanteren relatieve paden vanaf een bestaande gegevensmap. Ze weigeren ankers, dubbele punten en ..-componenten en controleren het opgeloste doel tegen de mapgrens. Directoryscans slaan symlinks en Windows-reparsepoints over. Alleen-lezen acties kunnen wel vertrouwelijke inhoud aan de MCP-client teruggeven. Kies de beschikbare gegevens daarom bewust.

## MCP-transport

De MCP-service staat Host 127.0.0.1:3000 en localhost:3000 toe. Een aanwezige Origin moet http://127.0.0.1:3000 of http://localhost:3000 zijn. Requests zonder Origin worden toegelaten voor native clients. Deze controles zijn op de MCP-service ingesteld. De aanvullende REST-routes gebruiken de loopbacklistener.

rmcp verzorgt protocolverwerking en sessiebeheer. legacy_session_mode is ingeschakeld en protocol 2025-11-25 is met de SDK-client getoetst. Transporttests controleren Host/Origin, contentonderhandeling, sessies, protocolheaders en bodygrenzen. SDK-deserialisatiefouten geven in deze versie HTTP 415.

## Logging en foutresponses

Uitvoeringslogs bevatten registry-toolnaam, succes, exitcode of foutcategorie en duur. Voor een onbekende naam wordt <unknown> gebruikt. Geïsoleerde tests controleren dat argumentwaarden en procesoutput ontbreken in deze logregels.

De adapters geven vaste veilige foutcodes en berichten terug. Interne procesbeheerfouten worden execution_failed. Bij een afgerond proces worden stdout en stderr wel als bedoeld toolresultaat aan de client teruggegeven. Configuratiediagnostiek bij startup kan het gekozen configuratiepad en de foutoorzaak bevatten.

## Resterende beperkingen

| Onderwerp | Grens van de oplevering |
| --- | --- |
| Lokale toegang | Authenticatie, gebruikersautorisatie en TLS voor remote deployment vragen een afzonderlijk ontwerp. R18 staat gepland voor later. |
| Procesrechten | Tools draaien met serverrechten. Procesgroepen, Job Objects en argumentvalidatie leveren procesbeheer. De serveraccountrechten bepalen de toegang tot het OS. |
| Bestandssysteemraces | Bestanden, roots en symlinks kunnen wijzigen tussen controle en gebruik. Roots worden per uitvoering opnieuw opgelost. Beheer ook bovenliggende directories. |
| Nieuwe paden | allowed_roots ondersteunt bestaande paden. Nieuwe bestanden als doelpad en niet-UTF-8-argumentpaden worden afgewezen. |
| Optionele parameters | Eén optionele placeholder als laatste argument, met beperkingen op vaste opties. Alleen stringparameters zijn beschikbaar. |
| Disconnects | Expliciete MCP-annulering en server-shutdown zijn aangesloten. Onverwacht clientdisconnect heeft nog geen afzonderlijk vastgelegd annuleringsbeleid. |
| Windows-links | Een afzonderlijke Windows-symlink/junctiontest ontbreekt. Unix-symlinkscenario’s zijn opgenomen in de Rust-suite. |
| Resourcequota | CPU-, RAM- en schijfquota vragen OS-beheer. De kern begrenst uitvoeringstijd, uitvoerbytes, requestgrootte en gelijktijdigheid. |
| Andere procesbomen | PID-tests onderbouwen de opgenomen scenario’s. Executables die zich buiten hun procesgroep of Job Object verplaatsen vragen aanvullende beoordeling. |
| Persoonlijke tools | Succesvolle PDF- en media-analyse hangt af van geïnstalleerde programma’s en ondersteunde bestanden. ffprobe leest via stdin met beperkte protocollen en formaten. |
| Externe dependencies | Python, rg, Poppler en FFmpeg worden door de beheerder geïnstalleerd. De CI-basistest controleert ontdekking van de profielen en de Git-keten. |
| Clientgedrag | Claude kan invoer al vóór een toolcall afwijzen. Het draaiboek bevat een aanvullende Inspector-call voor servervalidatie. |

## Verificatie

De Rust-suite bevat parameter-, pad-, procesboom-, transport-, foutmapping- en loggingtests. F05-run #50 en de F06-platformcontrole van 2026-11-10 zijn geslaagd op Windows, Ubuntu en macOS. De eigenaar bevestigt op 2026-11-11 dat de demo met Claude Code werkt. De README-doorloop en eindcontrole van de opleveringsstand volgen nog. Zie het [testplan](testplan.md) en de [F06-notitie](../notities/F06-oplevering.md).
