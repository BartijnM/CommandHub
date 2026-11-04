# Configuratievoorbeelden

Deze documentatie beschrijft het JSON-formaat, uitgebreid tijdens F04 (Review). De applicatie leest en valideert dit configuratiebestand bij startup.

Bij startup leest de server standaard `config/tools.json`, relatief aan de huidige werkdirectory. De beheerder kan een andere bestandslocatie instellen met de omgevingsvariabele `COMMANDHUB_CONFIG`.

Een tool heeft een naam, beschrijving, executable (`command`), lijst losse argumenten en parameters. Een placeholder verwijst naar een parameter en staat als volledig argument. `timeout` is optioneel, in milliseconden, met een standaard van 10.000 ms en een maximum van 60.000 ms.

`command` kan een absoluut pad zijn of een executable-naam, zoals `git`, die via de `PATH` van het serverproces wordt gevonden. De configuratiebeheerder bepaalt deze waarde; een client kan die niet aanleveren of wijzigen.

```json
{
  "tools": [
    {
      "name": "list_files",
      "description": "List files in a directory",
      "command": "ls",
      "arguments": ["-la", "{path}"],
      "parameters": [
        {
          "name": "path",
          "type": "string",
          "required": true
        }
      ]
    },
    {
      "name": "git_status",
      "description": "Show the status of a Git repository",
      "command": "git",
      "arguments": ["-C", "{repository}", "status"],
      "parameters": [
        {
          "name": "repository",
          "type": "string",
          "required": true
        }
      ],
      "timeout": 5000
    }
  ]
}
```

`timeout` weglaten betekent 10 seconden. Een ingestelde timeout moet groter zijn dan 0 en maximaal 60 seconden zijn. De applicatie leest het voorbeeldformaat strikt in en wijst onbekende velden en ongeldige configuraties af. De voorbeelden tonen configuratie; procesuitvoering en bijbehorende beveiligingsgrenzen vallen buiten F02. De genoemde `ls`-opdracht is niet op elk platform beschikbaar; kies bij uitvoering een platformgeschikte executable.

De client kan `command` of vaste argumenten niet wijzigen. De huidige F04-code ondersteunt dynamische parametervervanging; parameterloze tools blijven een lege parameterslijst gebruiken.

F03 staat native executables en expliciet geconfigureerde interpreters toe. Voor een script configureert de beheerder de interpreter als `command` en het script als vast argument. De server start niet automatisch een shell en dispatcht niet impliciet naar Windows `.bat`- of `.cmd`-bestanden. Het childproces erft de omgeving en werkdirectory van de server.

De F03-grenzen zijn maximaal 1 MiB gecombineerde stdout/stderr, 16 KiB requestbody en vier gelijktijdige uitvoeringen. Een volle capaciteit wordt direct afgewezen. De executor, process-wrap-platformwrappers, procesboomtests, HTTP-requestlimiet en POST-route zijn geïmplementeerd. POST /tools/{toolname} accepteert een JSON-object met gevalideerde parameters; een parameterloze tool verwacht een leeg object. Lege toolnamen, parameternamen en commands (ook uitsluitend witruimte) worden bij startup afgewezen.

## macOS (F07)

Gebruik [tools.macos.json](tools.macos.json) op macOS. Kies het bestand vanuit de projectmap met `export COMMANDHUB_CONFIG=config/tools.macos.json` en start daarna `cargo run --locked`. De standaard tools.json blijft het bestaande Windows-voorbeeld; er is geen automatische OS-selectie.

De macOS-configuratie bevat git_version, echo_ready (/bin/echo met een vast argument), hostname, whoami en ping_localhost. macOS-ping gebruikt losse argumenten -c, 4 en 127.0.0.1; de Windows-configuratie gebruikt -n. De toegevoegde CI-test controleert configuratievalidatie en voert op macOS echo_ready uit. De macOS-CI-job is geslaagd; de andere voorbeeldtools zijn niet afzonderlijk geverifieerd.

## F04-velden en beperkingen

Parameterdefinities ondersteunen optioneel allowed_values (lijst strings) en allowed_roots (lijst absolute bestaande directorypaden). Zie [het API-contract](../docs/api-en-configuratie.md) voor de exacte validatieregels. De bovenstaande padvoorbeelden stammen uit F02 en bevatten nog geen roots; voeg voor werkelijk padbeleid expliciet allowed_roots met een bestaande beheerderslocatie toe. Zonder roots is de parameter gewone tekst. De meegeleverde tools.json en tools.macos.json bevatten nog geen nieuwe dynamische F04-voorbeeldtool.

Een ontbrekende optionele placeholder wordt weggelaten. Maximaal één optionele placeholder, alleen als laatste argument en zonder vaste argumenten met prefix - of /, is toegestaan. De beheerder beoordeelt per executable de argumentbetekenis; prefixcontroles vormen geen volledige toolisolatie.

## Geteste dynamische voorbeelden

[tools.f04.json](tools.f04.json) bevat rust_details (detail: --version of -vV), git_recent (count als string 1/3/5) en git_diff_view (view: --stat of --name-only). Dit aparte bestand bevat uitsluitend deze drie tools; de standaardconfiguratie wordt niet aangepast. Kies het via COMMANDHUB_CONFIG en herstart. Alle drie zijn lokaal werkelijk uitgevoerd. Zie [de testhandleiding](../docs/F04-handmatig-testen.md) voor PowerShell-aanroepen en een optioneel padvoorbeeld.

Iedere parameter moet minstens één volledige placeholder hebben. Het ontdekschema toont de waardelijsten en wijst extra velden af. F04 staat op Review: lokale formatter-, Clippy- en testcontroles zijn geslaagd; CI op drie platforms en merge volgen nog.
