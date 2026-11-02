# Securityontwerp

Status: configuratievalidatie, parameterloze HTTP-uitvoering, geselecteerde uitvoeringslogging en server-shutdown zijn geïmplementeerd. Op 2026-11-02 slagen Windows-formatter, Clippy, build en 59 tests, inclusief echte TCP-concurrency en actieve procesboomopruiming. Linux-controles zijn voorbereid maar nog niet uitgevoerd.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke namen, bekende placeholders, geldige limieten; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden; geen automatische shell-dispatch. Scripts zijn alleen toegestaan via een expliciet geconfigureerde interpreter met vaste scriptargumenten. Stdin sluiten. Het childproces erft de omgeving en werkdirectory van de server; de client kan deze niet wijzigen.
- F03: timeout van maximaal 60 seconden (standaard 10 seconden), maximaal 1 MiB gecombineerde stdout/stderr en maximaal vier gelijktijdige uitvoeringen per gedeelde service-instantie zijn in de kern aanwezig. De POST-route heeft een bodylimiet van 16 KiB vóór JSON-deserialisatie; de exacte grens en overschrijding zijn getest. Bij volle capaciteit direct afwijzen. `process-wrap` verzorgt platformbeheer; gerichte Windows-tests en echte TCP-tests controleren het verdwijnen van child/subprocess na shutdown en vrijgave. Linux-verificatie blijft open; deze wordt als laatste uitgevoerd.
- F03: een expliciet `oneshot`-bericht annuleert één uitvoering via de opruimroute; sluiten zonder bericht niet. Server-shutdown gebruikt een gedeelde `watch`-status, weigert nieuwe uitvoeringen en wacht op capaciteitsvrijgave. Ctrl+C is aangesloten. Future-drop/abort is een apart Windows-getest vangnet via `KillOnDrop`, zonder normaal teruggegeven opruimresultaat. Unix gebruikt daarnaast een groepsbewaker bij future-drop en groepsbeëindiging na afloop van de leider; deze tak is nog niet op Linux uitgevoerd. Expliciet clientdisconnectbeleid blijft open.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- HTTP: accepteer alleen `{}` en JSON-contenttype; parameters, onbekende velden en andere JSON-typen worden afgewezen. Executable en argumenten komen uit de registry. Vaste JSON-foutresponses geven geen interne OS-details terug; startfout en foutmapping zijn via HTTP getest.
- Alle: geen geheimen, volledige parameters, executablepaden of stdout/stderr loggen. De loggingwrapper gebruikt uitsluitend registry-toolnaam (anders `<unknown>`), uitkomst, exitcode/foutcategorie en duur. De gecontroleerde argumentwaarden en procesoutput ontbreken in de geïsoleerde logtests.

## Afspraken vóór uitvoering

- Native executables en expliciet geconfigureerde interpreters zijn toegestaan; geen automatische `.bat`/`.cmd`- of shell-dispatch.
- Executablepaden mogen absoluut zijn of via de geërfde server-`PATH` worden gevonden.
- De timeout-, output-, request- en concurrentiegrenzen staan vast in het API-contract.
- `process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Windows-procescontroles zijn async en begrensd: 3 s per `tasklist`, maximaal 5 s herhaald controleren. Dit is geen sandbox of garantie voor alle procesbomen of blokkerende OS-processtart.
- Linux-processtatus via `/proc` is voorbereid; een zombie geldt als gestopt, niet als reeds gereaped. Geen Linux-garantie totdat Ubuntu-CI de controles uitvoert.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.
