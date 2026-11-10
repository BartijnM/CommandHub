# Toolconfiguratie

CommandHub valideert bij het starten het JSON-bestand uit `COMMANDHUB_CONFIG`. De standaard is `config/tools.json`, relatief aan de serverwerkmap. Start vanuit de projecthoofdmap. Kies op macOS `export COMMANDHUB_CONFIG=config/tools.macos.json` vóór `cargo run --locked`. Herstart de server na wijzigingen aan de configuratie of de serveromgeving.

## Configuratieformaat

Een tool heeft een unieke naam, een beschrijving, een executable (`command`), losse argumenten en een lijst met parameters. De executable is een absoluut pad of een naam op het `PATH` van de server. Plaats een placeholder als volledig argument. Een parameterloze tool heeft een lege parameterslijst.

```json
{
  "tools": [
    {
      "name": "service_health",
      "description": "Controleer de HTTP-status van een vooraf ingestelde lokale service.",
      "command": "python",
      "arguments": ["-I", "tools/personal_tools.py", "service_health", "{service}"],
      "parameters": [
        {
          "name": "service",
          "type": "string",
          "required": true,
          "allowed_values": ["commandhub", "local_web"]
        }
      ]
    }
  ]
}
```

`timeout` is optioneel en staat in milliseconden. De standaard is 10.000 en het toegestane bereik is 1–60.000. De executor begrenst gecombineerde stdout en stderr op 1 MiB en deelt vier uitvoeringsplekken tussen REST en MCP. De requestbody is begrensd op 16 KiB.

`allowed_values` begrenst een string tot een keuzelijst. Met `allowed_roots` valideert de kern een absoluut clientpad tegen bestaande absolute directorypaden. Een optionele placeholder mag maximaal één keer voorkomen, achteraan en zonder vaste argumenten die met `-` of `/` beginnen. Iedere parameter moet minimaal één volledige placeholder hebben. Zie het [API-contract](../docs/api-en-configuratie.md).

## Meegeleverde tools

| Groep | Tools | Benodigd op de server |
| --- | --- | --- |
| Repository | `repo_status`, `git_recent`, `git_diff_view`, `git_version` | Git en een repository als serverwerkmap voor de inspectietools |
| Project | `project_search`, `project_format_check` | Respectievelijk ripgrep (`rg`) en Cargo met rustfmt |
| Bestanden | `folder_overview`, `document_search`, `file_sha256`, `csv_preview` | Python en een gegevensmap |
| PDF | `pdf_info` | Python, de gegevensmap en `pdfinfo` van Poppler |
| Media | `image_info`, `video_info` | Python, de gegevensmap en `ffprobe` van FFmpeg |
| Notities | `notes_search`, `notes_recent` | Python en een submap `notes` in de gegevensmap |
| Systeem | `disk_space`, `service_health` | Python |
| Platformcontrole | `rust_details` in de standaardconfiguratie, `echo_ready` op macOS | Respectievelijk rustc en `/bin/echo` |

Beide configuraties bevatten achttien tools. `git_recent` accepteert `count` als string (`"1"`, `"3"` of `"5"`). `git_diff_view` accepteert `view=--stat` of `view=--name-only`. De [README](../README.md#mcp-handmatig-testen) bevat aanroepen via MCP Inspector.

## Gegevensmap en persoonlijke tools

De configuraties starten [personal_tools.py](../tools/personal_tools.py) met een vaste toolnaam en losse argumenten. De standaardconfiguratie gebruikt `python`, de macOS-configuratie `python3`. Gebruik Python 3.10 of nieuwer en zet de benodigde programma’s op het `PATH` van de server.

De gegevensmap is `CommandHub-data` in de thuismap van het account dat CommandHub draait. Met `COMMANDHUB_DATA_DIR` kan ik vooraf een andere bestaande absolute map kiezen. Bestandstools accepteren relatieve paden vanaf die map. Het script controleert het opgeloste pad op de mapgrens. Bij recursief doorzoeken slaat het symbolische links en Windows-junctions over. Ik plaats alleen bestanden in de gegevensmap die via MCP opgevraagd mogen worden.

- `folder_overview` geeft maximaal 25 bestanden in de gekozen map. `notes_recent` geeft maximaal 25 tekst- en Markdown-notities uit de submap `notes` en onderliggende mappen.
- Zoeken is hoofdletterongevoelig en beperkt tot 25 treffers. Tekstbestanden groter dan 1 MiB worden overgeslagen. Documenten mogen onder meer `.txt`, `.md`, `.log`, `.json` of `.csv` zijn. Notities gebruiken `.txt` of `.md`.
- `csv_preview` toont de kopregel en maximaal tien gegevensrijen. De uitvoer is begrensd op 40 velden per rij en 160 tekens per veld. Het veld `truncated` geeft aan of de uitvoer is ingekort.
- `service_health` gebruikt `commandhub` voor `127.0.0.1:3000/status` en `local_web` voor `127.0.0.1:80/`. De client kiest uit deze twee waarden.
- Persoonlijke tools geven JSON op stdout. MCP levert die tekst in `structuredContent.stdout`. Een onbereikbare service kan een succesvolle toolaanroep opleveren met `reachable: false` en `healthy: false`.
