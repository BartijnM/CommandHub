# API- en configuratiecontract

Status: `GET /status`, `GET /tools` en `POST /tools/{toolname}` zijn geïmplementeerd. F03-uitvoering voor parameterloze tools is op Windows automatisch en via een echte HTTP-aanroep gecontroleerd. Dynamische parameters volgen in F04.

## HTTP

| Route | Doel |
| --- | --- |
| GET /status | Lokale beschikbaarheid |
| GET /tools | Toolbeschrijvingen en parameters opvragen |
| POST /tools/{toolname} | Tool uitvoeren met een JSON-object als invoer |

Een optionele GET /tools/{toolname} kan later worden toegevoegd. Niet nodig om de eerste scope groter te maken.

Het uitvoeringsresultaat heeft deze vorm:

```json
{
  "success": true,
  "stdout": "...",
  "stderr": "",
  "exitCode": 0,
  "duration": 12
}
```

duration is milliseconden. exitCode kan null zijn wanneer het OS geen numerieke code levert. Een niet-nul exitcode is een afgeronde uitvoering met success=false; een uitvoeringsfout is een afzonderlijke foutcategorie.

Voor F03 geldt deze foutmapping:

| Situatie | HTTP-status | Gedrag |
| --- | --- | --- |
| Ongeldige JSON of ongeldige invoer | 400 | Geen proces starten |
| Geconfigureerde tool met parameters of placeholders | 400 | Niet starten; `parameters_not_supported` |
| Onbekende tool | 404 | Geen proces starten |
| Requestbody groter dan 16 KiB | 413 | Request afwijzen vóór JSON-deserialisatie |
| Capaciteit van vier uitvoeringen bereikt | 503 | Direct afwijzen, niet in een wachtrij plaatsen |
| Annulering of uitvoering na begonnen shutdown | 503 | `execution_cancelled`; bij actieve annulering eerst opruiming proberen |
| Interne processtart-, lees-, wacht- of beëindigingsfout | 500 | `execution_failed`, zonder interne foutdetails |
| Timeout | 504 | Procesboom beëindigen |
| Gecombineerde stdout/stderr groter dan 1 MiB | 502 | Procesboom beëindigen; geen afgekapt succesresultaat teruggeven |
| Proces gestart met niet-nul exitcode | 200 | Normaal uitvoeringsresultaat met `success: false` en de exitcode |

F03 accepteert uitsluitend een leeg JSON-object `{}` met `Content-Type: application/json`. Onbekende velden en andere JSON-typen worden met 400 en `invalid_input` afgewezen. JSON-extractiefouten, waaronder ontbrekend JSON-contenttype, geven 400 en `invalid_json`. Een body boven 16 KiB geeft 413 en `request_too_large`, vóór JSON-deserialisatie; precies 16 KiB is toegestaan. Toolparameters en placeholders volgen in F04.

Foutresponses hebben het vaste formaat:

```json
{
  "code": "unknown_tool",
  "message": "Onbekende tool"
}
```

Overige codes zijn `capacity_exceeded` (503), `execution_timeout` (504) en `output_limit_exceeded` (502). De response bevat geen interne OS-fouttekst. Een onbekende tool geeft pas 404 na geldige body/invoercontrole. Dit volgt uit de volgorde extractor → invoercontrole → toolselectie.

Handmatig gecontroleerd op Windows op 2026-10-30: `POST /tools/git_version` met `{}` gaf `success: true`, stdout met Git-versie 2.55.0.windows.3, lege stderr, exitcode 0 en duration 61 ms. Een herhaalde uitvoering kan een andere duur opleveren.

## Configuratie

Het eerste formaat sluit aan op het configuratievoorbeeld uit de opdracht: tools met `name`, `description`, `command`, `arguments`, `parameters` en optioneel `timeout`. `parameters` ondersteunen aanvankelijk `name`, `type` (`string`) en `required`. Zie [configuratievoorbeelden](../config/README.md).

### Eerste schema-keuzes

De casus geeft richting en voorbeelden, maar legt niet alle randgevallen of limietwaarden vast. Voor de eerste versie gelden deze keuzes:

- De server leest standaard `config/tools.json`, relatief aan de huidige werkdirectory. De beheerder kan dit pad overschrijven met de omgevingsvariabele `COMMANDHUB_CONFIG`.
- Ondersteun als parametertype alleen `string` en een `required`-boolean. Breid dit pas uit als een feature dat nodig maakt.
- Een placeholder heeft de vorm `{naam}` en mag alleen een volledig argument innemen, bijvoorbeeld `"{path}"`. Plak geen vaste tekst en placeholder aan elkaar; zo blijft elk procesargument een afzonderlijk item.
- `command` en de vaste argumenten komen uitsluitend uit beheerdersconfiguratie. `command` mag een absoluut pad of een executable-naam zijn die via de `PATH` van het serverproces wordt gevonden. Een HTTP- of MCP-client kan geen executable kiezen of de `PATH` aanpassen.
- Wijs lege toolnamen, parameternamen en commands af, ook als ze alleen witruimte bevatten. Waarden worden gecontroleerd met trim(), maar niet automatisch aangepast.
- Wijs onbekende configuratievelden af, zodat typefouten niet ongemerkt worden genegeerd. Wijs ook ontbrekende verplichte velden, ongeldige waarden, dubbele toolnamen en placeholders die niet bij een parameter horen af.
- `timeout` is optioneel per tool en wordt uitgedrukt in milliseconden. Zonder deze instelling geldt een standaardtimeout van 10.000 ms (10 seconden). Een ingestelde timeout moet groter zijn dan 0 en mag maximaal 60.000 ms (60 seconden) zijn; grotere waarden worden afgewezen.
- De configuratie blijft na startup onveranderlijk; een gewijzigde configuratie wordt actief na een herstart, niet via hot-reload.

De beheerder stelt `command` en de vaste argumenten in; de server laat de client dit niet overschrijven. De beheerder kan dus een absoluut pad of een naam gebruiken die via de server-`PATH` wordt gevonden. F03 start native executables. Een script mag alleen worden uitgevoerd wanneer de beheerder de interpreter expliciet als `command` configureert en het script als vast argument opgeeft. De server start niet automatisch een shell en dispatcht niet impliciet naar Windows `.bat`- of `.cmd`-bestanden.

### Uitvoeringsgrenzen voor F03

F03 hanteert maximaal 1 MiB gecombineerde stdout en stderr, maximaal 16 KiB requestbody en maximaal vier gelijktijdige uitvoeringen. Een volle capaciteit wordt direct afgewezen. Bij overschrijding van de outputlimiet wordt de procesboom beëindigd en als expliciete fout gemeld; afgekorte uitvoer mag niet op een normale succesvolle uitvoering lijken. De standaardtimeout blijft 10.000 ms; de configureerbare bovengrens blijft 60.000 ms.

Het childproces erft de omgeving en werkdirectory van het serverproces. De client kan deze niet instellen. Beheer van de serveromgeving is daarom onderdeel van het vertrouwensbeleid; omgevingswaarden worden niet gelogd.

Timeout, outputoverschrijding en expliciete annulering worden in de F03-kern afgehandeld. `process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Windows-tests controleren subprocess-PID's na timeout, outputoverschrijding, annulering, shutdown, future-drop en taakafbreking, ook bij een eerder afgerond direct child. Run #20 controleerde Linux-status en Unix-groepsopruiming succesvol, maar run #21 vond een ESRCH-race in de testhelper. De correctie is daarna via run #22 op Ubuntu en Windows gecontroleerd; het HTTP-contract is ongewijzigd.

De annuleerbare kernmethode ontvangt een `oneshot::Receiver<()>`. Een expliciet bericht leidt na de opruimroute tot `Cancelled`; een opruimfout kan die fout vervangen. Sluiten zonder bericht annuleert niet. Shutdown gebruikt daarnaast een `watch`-signaal, weigert nieuwe uitvoeringen en wacht op capaciteitsvrijgave. Ctrl+C is hierop aangesloten. De HTTP-mapping voor `Cancelled` is 503. Dit levert geen publiek annuleringsendpoint op; expliciet clientdisconnectbeleid en MCP-mapping blijven open.

## MCP

De voorbeeld-REST-routes zijn geen volledige MCP-implementatie. Gebruik de officiële SDK en kies in F00 een passende protocolversie en HTTP-transport. Laat zowel REST als MCP dezelfde registry en uitvoeringskern gebruiken.

F00 legt client, protocolversie, initialisatie/onderhandeling, toollisting, toolcalls, schema's en toepasselijke beveiligingsregels vast. De MCP-resultaatstructuur volgt het protocol en is niet simpelweg dezelfde body als REST.
## Historische aanvullende verificatie — vóór ESRCH-correctie

Op 2026-11-02 slagen 59 tests en alle Windows-Cargo-eindcontroles. Echte TCP-verzoeken bevestigen vier actieve uitvoeringen, afwijzing van het vijfde verzoek met 503 en herstel na afloop. Actieve server-shutdown via hetzelfde afsluitpad als Ctrl+C geeft 503 aan het lopende request, sluit de server en ruimt child en subprocess op. De integratietest injecteert het signaal; hij verstuurt geen OS-Ctrl+C. Logging bevat alleen geselecteerde metadata, geen argumentwaarden of procesoutput. Linux is lokaal niet uitgevoerd, maar daarna via de geslaagde Ubuntu-CI geverifieerd (run #20, aangeleverde screenshot).

## Reviewstand — 2026-11-03

Het HTTP-contract is ongewijzigd. Configuratie wijst nu ook lege namen en commands af. De huidige Windows-suite slaagt met 64 tests en 4 proceshelpers; de Windows-statushelper gebruikt handles. CI van deze refactor is op Ubuntu en Windows geslaagd voor commit 105e69b; oudere resultaten hierboven zijn historie. Zie de F03-notitie voor de eindcontrole.

## macOS-configuratie (F07)

Het HTTP-contract blijft gelijk. Kies op macOS config/tools.macos.json via COMMANDHUB_CONFIG; de standaardbestandskeuze verandert niet. Platformgeschikte vaste argumenten blijven door de beheerder bepaald. macOS-procescontrole en een macOS-CI-job zijn toegevoegd, maar nog niet uitgevoerd. Zie [F07](features/F07-macos-ondersteuning.md).
