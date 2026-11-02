# Configuratievoorbeelden

Deze documentatie beschrijft het JSON-formaat voor F02. De applicatie leest en valideert dit configuratiebestand bij startup.

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

De client kan `command` of vaste argumenten niet wijzigen. F03 voert alleen tools zonder dynamische parameters uit; daarvoor bevat `parameters` een lege lijst en worden geen placeholders gebruikt. Parametervervanging volgt in F04.

F03 staat native executables en expliciet geconfigureerde interpreters toe. Voor een script configureert de beheerder de interpreter als `command` en het script als vast argument. De server start niet automatisch een shell en dispatcht niet impliciet naar Windows `.bat`- of `.cmd`-bestanden. Het childproces erft de omgeving en werkdirectory van de server.

De F03-grenzen zijn maximaal 1 MiB gecombineerde stdout/stderr, 16 KiB requestbody en vier gelijktijdige uitvoeringen. Een volle capaciteit wordt direct afgewezen. De executor en process-wrap-platformwrappers zijn geïmplementeerd; HTTP-requestlimitering, procesboomtests met subprocessen en de uitvoeringsroute zijn nog niet gereed.
