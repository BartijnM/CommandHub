# API- en configuratiecontract

Bijgewerkt op 2026-11-11. REST en MCP gebruiken dezelfde registry, inputschema’s, parametervalidatie en procesuitvoering.

## REST-routes

| Methode en route | Resultaat |
| --- | --- |
| GET /status | HTTP 200 met status en applicatienaam |
| GET /tools | Object met tools-array met name, description en inputSchema |
| POST /tools/{toolname} | Tool uitvoeren met een JSON-object als parameters |

De productielistener gebruikt 127.0.0.1:3000. GET /tools publiceert de geconfigureerde metadata en het inputschema. Executable, vaste argumenten en interne padroots worden weggelaten.

Een POST gebruikt Content-Type application/json. Parameterloze tools ontvangen een leeg object. Een succesvolle uitvoering heeft bijvoorbeeld deze response.

~~~json
{
  "success": true,
  "stdout": "git version ...\n",
  "stderr": "",
  "exitCode": 0,
  "duration": 12
}
~~~

duration is een getal in milliseconden. exitCode kan null zijn wanneer het OS geen numerieke exitcode levert. Een afgerond proces met niet-nul exitcode geeft HTTP 200 met success gelijk aan false. Procesoutput wordt als tekst met UTF-8-vervanging teruggegeven.

### REST-fouten

| Situatie | Status | Code |
| --- | --- | --- |
| JSON-/contenttype-extractiefout | 400 | invalid_json |
| Geldige JSON met een andere vorm dan een object | 400 | invalid_input |
| Ontbrekende, onbekende of ongeldige parameter of afgewezen pad | 400 | invalid_parameters |
| Onbekende tool | 404 | unknown_tool |
| Body groter dan 16 KiB | 413 | request_too_large |
| Vier uitvoeringsplekken bezet | 503 | capacity_exceeded |
| Annulering of uitvoering na begonnen shutdown | 503 | execution_cancelled |
| Timeout | 504 | execution_timeout |
| Gecombineerde stdout/stderr boven 1 MiB | 502 | output_limit_exceeded |
| Interne start-, lees-, wacht- of beëindigingsfout | 500 | execution_failed |

Foutresponses bevatten code en message.

~~~json
{
  "code": "invalid_parameters",
  "message": "Parameterwaarde is niet toegestaan"
}
~~~

De afhandelvolgorde is body/extractie, objectvorm, toolselectie, parametervalidatie en uitvoering. Een geldig object voor een onbekende tool geeft unknown_tool. Precies 16 KiB is toegestaan. De uitvoeringsfoutresponse schermt interne OS-foutdetails af.

## JSON-configuratie

De server leest bij startup config/tools.json of het pad uit COMMANDHUB_CONFIG. Relatieve paden worden opgelost vanaf de serverwerkmap. Na aanpassen van JSON of serveromgeving is een herstart nodig. De bestaande executable kan dezelfde configuratie opnieuw laden.

~~~json
{
  "tools": [
    {
      "name": "git_recent",
      "description": "Toon een begrensd aantal recente commits",
      "command": "git",
      "arguments": ["--no-pager", "log", "-n", "{count}", "--oneline", "--no-decorate", "--"],
      "parameters": [
        {
          "name": "count",
          "type": "string",
          "required": true,
          "allowed_values": ["1", "3", "5"]
        }
      ],
      "timeout": 10000
    }
  ]
}
~~~

| Veld | Contract |
| --- | --- |
| tools | Verplichte array met tooldefinities |
| name | Verplichte unieke niet-lege toolnaam |
| description | Verplichte string voor clientontdekking |
| command | Verplichte niet-lege executable-naam op server-PATH of executablepad |
| arguments | Verplichte array met afzonderlijke strings en volledige placeholders |
| parameters | Verplichte array, leeg voor parameterloze tools |
| timeout | Optioneel, standaard 10.000 ms, toegestaan bereik 1–60.000 ms |

Toolnamen, commands en parameternamen met uitsluitend witruimte worden afgewezen. De validatie gebruikt trim voor de controle en behoudt de oorspronkelijke waarde. Onbekende velden, ontbrekende verplichte velden, dubbele namen en ongeldige placeholders verhinderen startup. Startupdiagnostiek noemt het configuratiebestand en de oorzaak.

De beheerder bepaalt executable en vaste argumenten. Scripts gebruiken een expliciet geconfigureerde interpreter. Childprocessen erven de serveromgeving en werkdirectory. Stdin is gesloten. De client levert uitsluitend parameters uit het schema.

### Parameters en argumenten

Een parameter heeft name, type en required. Alleen type string is ondersteund. allowed_values en allowed_roots zijn optionele velden met exact deze snake_case-namen. Ontbrekende optionele beleidsvelden betekenen dat die beperking niet is ingesteld.

- Iedere parameter moet aan minstens één volledige placeholder zoals {count} gekoppeld zijn. Een placeholder beslaat één argument. Herhaling van dezelfde verplichte placeholder is toegestaan.
- Verplichte ontbrekende parameters, onbekende invoervelden, niet-stringwaarden en NUL worden afgewezen.
- allowed_values is een niet-lege lijst strings zonder NUL. Vergelijking is exact. Een lege string mag als expliciet toegestane waarde voorkomen.
- Zonder allowed_values is een waarde met - als eerste teken verboden. Zonder allowed_values én allowed_roots worden ook / en @ als eerste teken afgewezen.
- Een lege string blijft één argument. Spaties, quotes, Unicode en shelltekens worden behouden binnen dat argument.
- Een ontbrekende optionele placeholder wordt weggelaten. Er mag maximaal één optionele placeholder voorkomen, uitsluitend als laatste argument. Dezelfde tool mag dan geen vaste argumenten hebben die met - of / beginnen, inclusief --.

### Absolute padparameters in de kern

allowed_roots is een niet-lege lijst bestaande absolute directorypaden. Bij gebruik moet de clientwaarde een bestaand absoluut pad zijn. De kern canonicaliseert pad en roots en vergelijkt directorycomponenten. Het canonieke pad wordt aan het proces doorgegeven. De root zelf is toegestaan.

Relatieve paden, niet-bestaande doelen, doelen buiten roots en paden die niet als UTF-8-argument kunnen worden doorgegeven leveren invalid_parameters. Symlinkdoelen moeten binnen een toegestane root liggen. De [securitydocumentatie](security.md) beschrijft de grenzen rond bestandssysteemwijzigingen en races.

### Persoonlijke bestandstools

De meegeleverde Python-tools gebruiken een eigen relatief padbeleid binnen CommandHub-data of COMMANDHUB_DATA_DIR. Deze stringparameters krijgen hun mapcontrole in tools/personal_tools.py. Hun JSON-uitvoer staat als tekst in stdout. Afhankelijkheden, filters en grenzen staan in de [configuratiehandleiding](../config/README.md).

## Uitvoeringsgrenzen en logging

REST en MCP delen vier gelijktijdige uitvoeringsplekken. Bij volle capaciteit volgt direct capacity_exceeded. Beide adapters begrenzen requests op 16 KiB. De executor begrenst de gecombineerde ruwe stdout/stderr-bytes op 1 MiB per uitvoering.

Timeout, outputoverschrijding, expliciete annulering en shutdown activeren procesopruiming. Een fout bij opruimen kan als execution_failed terugkomen. De daadwerkelijke tijd tot afronding omvat ook processtart en opruiming. De server logt per uitvoering toolnaam, uitkomst, exitcode of foutcategorie en duration_ms.

## MCP via Streamable HTTP

/mcp gebruikt rmcp 3.6.0 met tools-capability, Streamable HTTP en legacy_session_mode. De SDK-ketentest rapporteert protocolversie 2025-11-25. De SDK verzorgt initialisatie, JSON-RPC en sessiebeheer.

| MCP-methode | Gedrag |
| --- | --- |
| tools/list | Volledige geconfigureerde lijst met name, description en inputSchema |
| tools/call | Geconfigureerde toolnaam en parameters aan de gedeelde kern doorgeven |

inputSchema gebruikt object, properties, required, enum voor allowed_values en additionalProperties met waarde false. Ontbrekende arguments worden een leeg object.

Een afgeronde uitvoering levert success, stdout, stderr, exitCode en duration als structuredContent en als JSON-tekst in content. isError volgt de omgekeerde success-waarde. Validatie-, capaciteits-, timeout-, output- en annuleringsfouten zijn toolresultaten met isError true en veilige code/message. Een onbekende tool levert de protocolfout METHOD_NOT_FOUND. Interne procesbeheerfouten worden execution_failed.

### MCP-transportinstellingen

| Instelling | Waarde |
| --- | --- |
| Listener | 127.0.0.1:3000 |
| Toegestane Hosts | 127.0.0.1:3000 en localhost:3000 |
| Toegestane aanwezige Origins | http://127.0.0.1:3000 en http://localhost:3000 |
| Ontbrekende Origin | Toegestaan voor native clients |
| Bodylimiet | 16 KiB |
| Sessiebeheer | LocalSessionManager, legacy_session_mode ingeschakeld |

### Getoetste MCP-transportfouten

| Geval | HTTP-status in rmcp 3.6.0 |
| --- | --- |
| Ongeldige JSON of ongeldige JSON-RPC-body | 415 |
| Content-Type text/plain | 415 |
| Accept uitsluitend application/json | 406 |
| Onbekende MCP-Protocol-Version-header | 400 |
| Onbekende MCP-sessie | 404 |
| Niet-toegestane Host of aanwezige Origin | 403 |
| Body groter dan 16 KiB | 413 |

Deze statussen horen bij transportafhandeling. Toolfouten worden via het MCP-resultaatmodel teruggegeven.

### Annulering en shutdown

De MCP-adapter vertaalt context.ct naar een expliciet oneshot-bericht aan de kern en wacht de uitvoeringsafhandeling af. server::serve signaleert uitvoeringsshutdown, sluit MCP-streams en wacht op capaciteitsvrijgave. Een actieve clientcall kan tijdens serverafsluiting een annuleringsresultaat of transportfout ontvangen. De shutdowntestclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit. Een afzonderlijk beleid voor onverwacht clientdisconnect blijft open.

## Voorbeelden en bewijs

De [README](../README.md) beschrijft starten en Inspector-aanroepen. Het [demo-draaiboek](demo.md) gebruikt Claude Code. [F04 handmatig testen](F04-handmatig-testen.md) bevat aanvullende REST- en padvoorbeelden. De requirementdekking en vastgelegde controles staan in het [testplan](testplan.md). Historische resultaten blijven bewaard in de [featurenotities](../notities/README.md).
