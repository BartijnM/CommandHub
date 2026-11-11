# F04 handmatig testen

De configuratie [config/tools.json](../config/tools.json) bevat onder andere de drie F04-voorbeeldtools. Ze wijzigen geen repositorybestanden en gebruiken expliciete keuzelijsten. Git en rustc moeten via PATH beschikbaar zijn. Start vanuit de projectmap zodat Git de bedoelde repository gebruikt.

## Server starten op Windows

Stop een eventueel bestaande CommandHub-server en start met de standaardconfiguratie:

```powershell
Remove-Item Env:COMMANDHUB_CONFIG -ErrorAction SilentlyContinue
cargo run --locked
```

Voer de aanroepen in een tweede PowerShell-terminal uit:

```powershell
Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/tools/rust_details" -ContentType "application/json" -Body '{"detail":"-vV"}' | ConvertTo-Json
Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/tools/git_recent" -ContentType "application/json" -Body '{"count":"3"}' | ConvertTo-Json
Invoke-RestMethod -Method Post -Uri "http://127.0.0.1:3000/tools/git_diff_view" -ContentType "application/json" -Body '{"view":"--stat"}' | ConvertTo-Json
```

| Tool | Toegestane invoer | Verwacht resultaat |
| --- | --- | --- |
| rust_details | detail: --version of -vV | Korte of uitgebreide Rust-versie |
| git_recent | count: string 1, 3 of 5 | Maximaal dat aantal commits, één regel per commit |
| git_diff_view | view: --stat of --name-only | Wijzigingen in bijgehouden bestanden; lege stdout is geldig bij een schone werkmap |

GET /tools toont de waarden als enum en additionalProperties: false. De Git-tools lezen de repository in de serverwerkdirectory; ze accepteren geen clientpad. git_diff_view schakelt externe diff- en textconv-programma’s uit. git_recent eindigt met -- zodat er geen dynamische revisies of padopties worden toegevoegd. De serveromgeving en Git-configuratie blijven beheerdervertrouwen.

## Afwijzingen controleren

Herhaal git_recent met elk van deze bodies. Verwacht HTTP 400 met code invalid_parameters (PowerShell meldt een fout bij niet-succesvolle HTTP-status):

```jsonl
{"count":"2"}
{"count":3}
{}
{"count":"3","command":"other"}
```

Dit controleert achtereenvolgens de keuzelijst, het stringtype, een verplicht veld en een onbekend veld. Voorbeeldtools en positieve uitvoering zijn lokaal automatisch getest. Dit is een aanvullende REST-handleiding. De bevestigde F06-demo gebruikt Claude Code. Deze afzonderlijke REST-doorloop heeft een eigen resultaatregistratie.

## Padbeleid handmatig proberen

Voeg desgewenst onderstaande tool toe aan de tools-array van een eigen kopie van de configuratie. Pas de root aan als de projectlocatie verschilt. De root moet al bestaan en absoluut zijn; kies uitsluitend een beheerde repositorylocatie.

```json
{
  "name": "repository_root",
  "description": "Toon de repositoryroot binnen de toegestane projectmap",
  "command": "git",
  "arguments": ["-C", "{path}", "rev-parse", "--show-toplevel"],
  "parameters": [{
    "name": "path",
    "type": "string",
    "required": true,
    "allowed_roots": ["C:/xampp/htdocs/Educom/Cases/CommandHub"]
  }],
  "timeout": 10000
}
```

Herstart de server met die kopie. Body met path gelijk aan de absolute projectmap moet slagen. Een bestaand absoluut pad erbuiten moet 400 invalid_parameters geven; een relatief of niet-bestaand pad wordt eveneens afgewezen. Dezelfde Git-route is lokaal via HTTP-routertests gecontroleerd, inclusief een pad met src/.. dat binnen de root blijft. Symlinkdoelen moeten binnen de root vallen; De Unix-symlinktest is via de geslaagde F04-CI op Ubuntu en macOS gecontroleerd. Een afzonderlijke Windows-symlink/junctiontest blijft een beperking.

## Standaardconfiguratie herstellen

Stop de server. In de terminal waarin de configuratievariabele is ingesteld:

```powershell
Remove-Item Env:COMMANDHUB_CONFIG -ErrorAction SilentlyContinue
cargo run --locked
```

## Optioneel PowerShell-voorbeeld

Het script [list-directory.ps1](../scripts/list-directory.ps1) kan als geconfigureerde tool directories tonen. Onderstaand voorbeeld gebruikt de oorspronkelijke lokale projectlocatie. Pas zowel het vaste scriptpad als allowed_roots aan naar jouw bestaande absolute projectmap. Voeg het object toe aan het actieve profiel en herstart de server.

```json
    {
      "name": "list_directory",
      "description": "Toon bestanden en mappen binnen de CommandHub-projectmap",
      "command": "powershell.exe",
      "arguments": [
        "-NoProfile",
        "-NonInteractive",
        "-File",
        "C:/xampp/htdocs/Educom/Cases/CommandHub/scripts/list-directory.ps1",
        "{path}"
      ],
      "parameters": [
        {
          "name": "path",
          "type": "string",
          "required": true,
          "allowed_roots": ["C:/xampp/htdocs/Educom/Cases/CommandHub"]
        }
      ],
      "timeout": 10000
    }
```

Voor de actuele volledige demo en de resultaatregistratie zie het [Claude Code-draaiboek](demo.md).
