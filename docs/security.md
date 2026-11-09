# Securityontwerp

Historische stand vóór F04 (2026-11-03): configuratievalidatie, parameterloze HTTP-uitvoering, uitvoeringslogging en shutdown zijn aanwezig. De laatste geverifieerde Windows-suite vóór F07 slaagde met 64 tests en 4 proceshelpers. De review voegt afwijzing van lege namen/commands en een Windows-controle via proceshandles toe. Run #22 bevestigde eerder Ubuntu en Windows na de ESRCH-correctie; de reviewwijzigingen zijn daarna op Ubuntu en Windows geslaagd in CI voor commit 105e69b.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke niet-lege namen, niet-leeg command, bekende placeholders en geldige timeoutgrenzen; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden; geen automatische shell-dispatch. Scripts zijn alleen toegestaan via een expliciet geconfigureerde interpreter met vaste scriptargumenten. Stdin sluiten. Het childproces erft de omgeving en werkdirectory van de server; de client kan deze niet wijzigen.
- F03: timeout van maximaal 60 seconden (standaard 10 seconden), maximaal 1 MiB gecombineerde stdout/stderr en maximaal vier gelijktijdige uitvoeringen per gedeelde service-instantie zijn in de kern aanwezig. De POST-route heeft een bodylimiet van 16 KiB vóór JSON-deserialisatie; de exacte grens en overschrijding zijn getest. Bij volle capaciteit direct afwijzen. `process-wrap` verzorgt platformbeheer; gerichte Windows-tests en echte TCP-tests controleren het verdwijnen van child/subprocess na shutdown en vrijgave. Linux-hercontrole na ESRCH-correctie is geslaagd in run #22.
- F03: een expliciet `oneshot`-bericht annuleert één uitvoering via de opruimroute; sluiten zonder bericht niet. Server-shutdown gebruikt een gedeelde `watch`-status, weigert nieuwe uitvoeringen en wacht op capaciteitsvrijgave. Ctrl+C is aangesloten. Future-drop/abort is een apart Windows-getest vangnet via `KillOnDrop`, zonder normaal teruggegeven opruimresultaat. Unix gebruikt daarnaast een groepsbewaker bij future-drop en groepsbeëindiging na afloop van de leider; run #20 controleerde deze tak; hercontrole na de racecorrectie slaagt in run #22. Expliciet clientdisconnectbeleid blijft open.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- HTTP (F04): accepteer een JSON-object en JSON-contenttype; de gedeelde validatie wijst onbekende en ongeldige parameters af. Andere JSON-vormen worden door de controller afgewezen. Executable en argumenten komen uit de registry. Vaste JSON-foutresponses geven geen interne OS-details terug; startfout en foutmapping zijn via HTTP getest.
- Alle: geen geheimen, volledige parameters, executablepaden of stdout/stderr loggen. De loggingwrapper gebruikt uitsluitend registry-toolnaam (anders `<unknown>`), uitkomst, exitcode/foutcategorie en duur. De gecontroleerde argumentwaarden en procesoutput ontbreken in de geïsoleerde logtests.

## Afspraken vóór uitvoering

- Native executables en expliciet geconfigureerde interpreters zijn toegestaan; geen automatische `.bat`/`.cmd`- of shell-dispatch.
- Executablepaden mogen absoluut zijn of via de geërfde server-`PATH` worden gevonden.
- De timeout-, output-, request- en concurrentiegrenzen staan vast in het API-contract.
- `process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Windows-procescontroles gebruiken OpenProcess en een niet-blokkerende WaitForSingleObject-controle; OwnedHandle sluit het handle automatisch. De async test wacht maximaal 5 s met herhaalde controles; toegangs- en API-fouten blijven testfouten. Dit is geen sandbox of garantie voor alle procesbomen of blokkerende OS-processtart.
- Linux-processtatus via `/proc` is voorbereid; een zombie geldt als gestopt, niet als reeds gereaped. De eerste Ubuntu-CI slaagde; na de ESRCH-fout uit run #21 is de helper gecorrigeerd en is hercontrole geslaagd in run #22. Dit toont de geconfigureerde scenario's aan, geen sandbox of garantie voor elke mogelijke procesboom.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.
De processtatushelper beschouwt uitsluitend NotFound/ENOENT en ESRCH als een verdwenen proces. Toegangsproblemen of ongeldige /proc-data worden niet als geslaagde beëindiging verborgen. Dit onderscheid is met regressietests gecontroleerd.

## macOS-uitbreiding — CI bevestigd

F07 hergebruikt Unix-procesgroepen en de bestaande uitvoeringsgrenzen. De macOS-testhelper gebruikt proc_pidinfo en beschouwt alleen ESRCH of een zombie als beëindigd; echte fouten blijven fouten. De FFI-buffer wordt alleen na een volledig resultaat uitgelezen. De aangeleverde screenshot bevestigt geslaagde jobs op macOS, Ubuntu en Windows. Dit verifieert de geconfigureerde testscenario’s; exacte CI-testtotalen zijn niet zichtbaar. Dit is geen sandbox of uitbreiding naar publieke hosting.

## Actuele F04-stand — 2026-11-04

Dynamische HTTP-uitvoering valideert vóór processtart. NUL, ongeldige typen en onbekende/ontbrekende parameters worden afgewezen. Waardelijsten en prefixbeperkingen beperken optie-invoer; dit is geen algemene bescherming voor iedere executable. De meegeleverde F04-tools zijn beoordeeld op hun argumentbetekenis en gebruiken expliciete keuzelijsten. Nieuwe beheerdersconfiguratie vraagt dezelfde beoordeling. Optionele argumenten zijn beperkt tot één laatste placeholder zonder vaste argumenten die met - of / beginnen.

Padbeleid vereist bestaande absolute clientpaden en roots. Canonicalisatie controleert symlinkdoelen met componentvergelijking en levert het werkelijk doorgegeven argument. Dit is geen sandbox; rootwijzigingen en races tussen controle en gebruik blijven mogelijk. Iedere parameter moet bij startup aan een placeholder gekoppeld zijn; padcontrole gebeurt bij argumentopbouw vóór processtart. De Unix-symlinktest is meegenomen in de geslaagde Ubuntu- en macOS-CI voor F04; Windows-symlinks zijn niet apart getest. HTTP-padtests en gerichte dynamische loggingtests zijn op Windows geslaagd. Formatter, Clippy en 138 tests slagen lokaal; CI is geslaagd op Windows, Ubuntu en macOS voor commit 1fb8d71. Zie de F04-notitie voor bewijs en vervolg.

## CI-afronding — 2026-11-04

De aangeleverde screenshot van run #39 voor commit 1fb8d71 op feature/F04-parameters toont Success: macOS 57 s, Ubuntu 40 s en Windows 1 m 50 s; totale duur 1 m 54 s. De workflow voert formatter, Clippy, tests en build uit. Hiermee is ook de macOS-platformverificatie bevestigd; de eerdere runnercapaciteitsmelding is geen open blokkade meer. Exacte CI-testtotalen zijn niet zichtbaar. De werking is lokaal bevestigd. Alleen PR-review/merge en branchafsluiting resten; status pas Klaar na bevestigde merge. Bij deze documentatie-update zijn geen broncode gewijzigd, tests herhaald of Git-mutaties uitgevoerd.


## F05-tussenstand op 2026-11-09

De lokale MCP-service beperkt Host en aanwezige Origin tot de ingestelde loopbackadressen op poort 3000. Ontbrekende Origin is toegestaan voor native clients. Tests controleren de allowlists, sessieinitialisatie, onbekende sessie, protocolversieheader, contentonderhandeling en de bodygrens van 16 KiB.

Protocolannulering is getest met een gestart proces en PID-bewijs. Productie-shutdown is getest met een MCP-procesboom, beide processtatussen, capaciteitsvrijgave en een gesloten luisterpoort. Een gemengde belastingstest controleert dat REST en MCP vier gedeelde uitvoeringsplekken gebruiken en na vrijgave herstellen. Interne OS-details worden in de MCP-foutmapping afgeschermd.

De shutdowntestclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit. Bij de standaard herverbindingsinstellingen bleef de clientaanroep wachten na serverafsluiting. Alleen de gedocumenteerde testclientinstelling is gewijzigd.

Open blijven MCP-padvalidatie via de clientketen, platform-CI en expliciet beleid voor onverwacht clientdisconnect. Beheerder beheert executable, serveromgeving en padroots. De eerder vastgelegde grenzen rond canonicalisatie gelden ook voor MCP.
