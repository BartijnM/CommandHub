# Demo met Claude Code en CommandHub

Dit draaiboek hoort bij F06 en demonstreert configuratie, ontdekking, uitvoering, validatiefouten, logging, timeout en uitbreiding zonder rebuild. Trek ongeveer 10–15 minuten uit, plus voorbereiding. Gebruik de Claude Code-extensie in VS Code als MCP-client.

## Voorbereiding

- Open de repository in VS Code en voer alle terminalcommando’s vanuit de projecthoofdmap uit.
- Installeer Rust met Cargo en een geschikte linker volgens de [README](../README.md#installeren-en-gebruiken).
- Zorg dat Git, ripgrep (`rg`) en Python beschikbaar zijn in de terminal die de server start. De timeouttool gebruikt Python op Windows en Python 3 op Linux/macOS.
- Installeer de Claude Code-extensie en meld je aan met een geschikt account. Registreer CommandHub als HTTP-MCP-server op `http://127.0.0.1:3000/mcp`. Een bestaande verbinding kan worden hergebruikt.
- Houd de serverterminal en het Claude Code-paneel naast elkaar zichtbaar. Open toolaanroepen tijdens de demo om invoer en resultaten te tonen.

Controleer de afhankelijkheden op Windows.

```powershell
git --version
rg --version
python --version
```

Gebruik op Linux en macOS voor de laatste controle `python3 --version`. Verwacht steeds versie-informatie. Los een ontbrekend commando op voordat je verdergaat.

### Kies één platformprofiel

| Platform | Te bewerken configuratie | Python-command |
| --- | --- | --- |
| Windows | `config/tools.json` | `python` |
| Linux | `config/tools.linux.json` | `python3` |
| macOS | `config/tools.macos.json` | `python3` |

Voer de bijbehorende instelling uit in de serverterminal.

**Windows, PowerShell**

```powershell
$env:COMMANDHUB_CONFIG = "config/tools.json"
$env:RUST_LOG = "info"
```

**Linux, Bash**

```sh
export COMMANDHUB_CONFIG=config/tools.linux.json
export RUST_LOG=info
```

**macOS, Bash of Zsh**

```sh
export COMMANDHUB_CONFIG=config/tools.macos.json
export RUST_LOG=info
```

### Voeg de timeouttool toe

Stop een eventueel draaiende CommandHub-server met Ctrl+C. Voeg het onderstaande object achteraan de bestaande `tools`-array toe in het gekozen platformprofiel. Zet een komma tussen het voorgaande object en dit object. Behoud de bestaande tools. Gebruik op Linux en macOS `"command": "python3"`.

```json
{
  "name": "demo_timeout",
  "description": "Demonstratie van een proces dat de ingestelde timeout overschrijdt",
  "command": "python",
  "arguments": ["-I", "-c", "import time; time.sleep(10)"],
  "parameters": [],
  "timeout": 1000
}
```

Het vaste Python-programma wacht tien seconden. CommandHub begrenst deze tool tot 1000 milliseconden. Executable en argumenten worden afzonderlijk geconfigureerd.

Bouw nu één keer.

```sh
cargo build --locked
```

Verwacht een geslaagde build. Bewaar deze executable voor alle volgende starts in het draaiboek.

## 1. Starten en tools ontdekken

Start op Windows de gebouwde executable.

```powershell
.\target\debug\commandhub.exe
```

Gebruik op Linux en macOS het volgende commando.

```sh
./target/debug/commandhub
```

Verwacht de logmeldingen `Configuratie geladen` met `aantal_tools` en `CommandHub gestart` met adres `127.0.0.1:3000`. Open in de browser [de statusroute](http://127.0.0.1:3000/status). Verwacht het volgende antwoord.

```json
{"status":"ok","applicatie":"commandhub"}
```

Open `/mcp` in het chatpaneel van de Claude Code-extensie, kies CommandHub en gebruik **Reconnect**. Controleer dat de server **Connected** toont. Klap de tools open. Verwacht onder andere `repo_status`, `git_recent`, `project_search`, `git_diff_view`, `git_version` en de toegevoegde `demo_timeout`.

Toon daarna in de editor één bestaande tooldefinitie. Leg de keten uit als Claude Code → MCP via HTTP → CommandHub → geconfigureerd proces → toolresultaat.

## 2. Projectonderzoek via Claude

Plak deze opdracht in het Claude Code-paneel.

```text
Gebruik de MCP-tools van CommandHub voor dit projectonderzoek.
Roep repo_status aan met {}.
Roep git_recent aan met {"count":"3"}.
Roep project_search aan met {"query":"router_with_shutdown"}.
Gebruik voor deze drie controles uitsluitend deze MCP-tools.
Vat de ontvangen resultaten samen en vermeld de gebruikte tools.
```

Geef bij een toestemmingsvraag toestemming voor de genoemde calls. Open de zichtbare toolaanroepen en controleer de argumenten. `count` wordt als string doorgegeven.

**Verwachte uitkomst**

- `repo_status` geeft de huidige branch en eventuele gewijzigde of nieuwe bestanden terug. De branchnaam en bestanden volgen de werkelijke Git-status.
- `git_recent` geeft maximaal drie commits met korte hash en onderwerp terug.
- `project_search` geeft treffers met bestandsnamen en regelnummers terug, waaronder de routercode in `src`.
- Geslaagde calls hebben `isError: false`. `structuredContent` bevat `success: true`, `stdout`, `stderr`, `exitCode: 0` en een numerieke `duration` in milliseconden. Dezelfde resultaatgegevens staan ook als JSON-tekst in `content`.
- Claude vat de ontvangen informatie samen. De formulering en de volgorde van de calls kunnen per sessie verschillen.

## 3. Ongeldige invoer afwijzen

Plak deze opdracht.

```text
Ik demonstreer de servervalidatie van CommandHub.
Probeer git_diff_view via MCP aan te roepen met {"view":"--patch"}.
Toon de ontvangen toolfout. Stop na deze demonstratie.
```

**Verwachte uitkomst bij een uitgevoerde call**

`--patch` valt buiten de geconfigureerde waarden `--stat` en `--name-only`. Het toolresultaat heeft `isError: true` en de volgende `structuredContent`.

```json
{
  "code": "invalid_parameters",
  "message": "Parameterwaarde is niet toegestaan"
}
```

Claude kan de invoer al op basis van het gepubliceerde schema weigeren. Leg in dat geval de clientafwijzing vast. Om ook de serverresponse te tonen, kun je in een tweede terminal de onderstaande Inspector-call uitvoeren. Hiervoor zijn Node.js 22 met npm nodig.

```sh
npx --yes @modelcontextprotocol/inspector@2.11.0 --cli http://127.0.0.1:3000/mcp --transport http --protocol-era legacy --format json --method tools/call --tool-name git_diff_view --tool-arg view=--patch
```

Verwacht de fout hierboven onder `result`. Inspector sluit voor deze toolfout af met exitcode `5`. Ga daarna verder in Claude Code.

## 4. Logging bekijken

Schakel naar de serverterminal en zoek de regels van de uitgevoerde calls.

**Verwachte uitkomst**

- Geslaagde calls tonen `Tooluitvoering afgerond` met `tool`, `success`, `exit_code` en `duration_ms`.
- De uitgevoerde ongeldige call toont `Tooluitvoering afgewezen of mislukt` met `tool=git_diff_view`, `category=invalid_parameters` en `duration_ms`.
- De uitvoeringslogregels bevatten uitsluitend de genoemde metadata. Controleer dat volledige argumentwaarden en procesoutput daarin ontbreken.

## 5. Timeout en herstel tonen

Plak deze opdracht.

```text
Roep demo_timeout van CommandHub precies één keer aan met {}.
Wacht op het resultaat en toon de ontvangen foutcode.
Roep daarna git_version aan met {} om een vervolgaanroep te controleren.
```

**Verwachte uitkomst**

De eerste call levert na ongeveer één seconde plus processtart en opruiming `isError: true` met de volgende `structuredContent`.

```json
{
  "code": "execution_timeout",
  "message": "Uitvoering verlopen"
}
```

De serverterminal toont voor `demo_timeout` de categorie `execution_timeout`. Daarna slaagt `git_version` met `isError: false`, `success: true`, `exitCode: 0` en stdout die met `git version ` begint. Dit toont herstel na timeout. De automatische procesbeheertests leveren aanvullend bewijs voor procesopruiming.

Als de eerste call `execution_failed` oplevert, controleer dan of het gekozen Python-command in de serverterminal werkt. Na wijziging van de configuratie herstart je dezelfde executable en verbind je Claude Code opnieuw.

## 6. Een nieuwe tool toevoegen zonder rebuild

Controleer eerst in de Claude-toollijst dat `demo_git_version` nog ontbreekt. Verwijder bij een herhaalde demo een eerdere definitie, herstart de server en gebruik Reconnect voordat je deze stap begint.

Stop de server met Ctrl+C. Voeg het volgende object toe aan dezelfde `tools`-array waarin je de timeouttool hebt gezet. Zet opnieuw een komma tussen de objecten.

```json
{
  "name": "demo_git_version",
  "description": "Toon de Git-versie via een tijdens de demo toegevoegde tool",
  "command": "git",
  "arguments": ["--version"],
  "parameters": []
}
```

Sla de JSON op en start direct dezelfde executable met het startcommando uit stap 1. Gebruik vervolgens **Reconnect** in het MCP-paneel van Claude Code.

**Verwachte uitkomst**

- De startupmelding toont één tool meer dan bij de vorige start.
- `demo_git_version` verschijnt in de MCP-toollijst.

Plak vervolgens deze opdracht.

```text
Roep demo_git_version van CommandHub aan met {} en toon het resultaat.
```

Verwacht `isError: false`, `success: true`, `exitCode: 0` en de Git-versie in stdout. De configuratiewijziging en herstart zijn voldoende om de nieuwe tool te ontdekken en uit te voeren.

## 7. Afronden en uitkomsten vastleggen

Stop de server met Ctrl+C. Verwijder na de demonstratie de twee toegevoegde objecten `demo_timeout` en `demo_git_version` uit het gekozen profiel. Controleer de komma’s. Bij een volgende start wordt het oorspronkelijke profiel geladen.

Onderstaande tabel bevat de door de eigenaar bevestigde doorloop. Bij herhaling kun je de werkelijke uitkomsten aanvullen met toolresponses en relevante logcategorieën.

| Onderdeel | Verwacht resultaat | Werkelijke uitkomst |
| --- | --- | --- |
| Startup en ontdekking | Status ok, Connected, timeouttool zichtbaar | Bevestigd door eigenaar op 2026-11-11 |
| Projectonderzoek | Drie MCP-calls met succesvolle resultaten | Bevestigd door eigenaar op 2026-11-11 |
| Validatiefout | invalid_parameters via servercall, of vastgelegde clientafwijzing met aanvullende Inspector-call | Bevestigd door eigenaar op 2026-11-11 |
| Logging | Genoemde metadata bij succes en fouten | Bevestigd door eigenaar op 2026-11-11 |
| Timeout en herstel | execution_timeout, daarna succesvolle git_version | Bevestigd door eigenaar op 2026-11-11 |
| Uitbreiding zonder rebuild | Nieuwe tool zichtbaar en uitvoerbaar na herstart | Bevestigd door eigenaar op 2026-11-11 |

Noteer ook datum, besturingssysteem, commit, gebruikt profiel en Claude Code-versie bij de resultaten. De bestaande screenshot bevestigt de verbinding met Claude Code en de oorspronkelijke toollijst. Op 2026-11-11 bevestigt de eigenaar dat de demo werkt. De tabel legt die bevestiging vast. Exacte meetwaarden, clientversie en de gebruikte route voor de ongeldige call zijn daarbij niet gespecificeerd.

## Koppeling aan requirements

| Requirements | Demonstratiestap |
| --- | --- |
| R01–R03 | Configuratie laden, tools ontdekken en een tool via JSON toevoegen |
| R04–R05, R08–R09, R11 | Projectonderzoek met zichtbare MCP-argumenten en procesresultaten |
| R06–R07 | Begrensde uitvoering met timeout en succesvolle vervolgaanroep |
| R13 | Veilige foutcategorieën en uitvoeringslogging |
| R17 | Reproduceerbaar draaiboek met verwachte en vast te leggen werkelijke uitkomsten |

Het [testplan](testplan.md) koppelt de demo en automatische tests aan alle Must-eisen. De gezamenlijke README-doorloop blijft onderdeel van de afronding van R17. De platformresultaten en open opleverpunten staan in [voortgang](voortgang.md). De instellingen van de client staan beschreven in de [officiële Claude Code-documentatie](https://code.claude.com/docs/en/mcp) en de [VS Code-handleiding](https://code.claude.com/docs/en/vs-code).
