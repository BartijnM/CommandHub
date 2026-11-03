---
id: F03
prioriteit: Must
status: Review
milestone: "Uitvoeren"
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

## Doel

Geconfigureerde tool met vaste argumenten uitvoeren. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F02.

## Requirements

R04, R05, R06, R07, R12, R14.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Werkaanpak

Bouw F03 iteratief: begin met een kleine werkende verticale slice die een geconfigureerde parameterloze tool start, stdout en stderr gelijktijdig begrensd uitleest, op het proces wacht en een resultaat oplevert. Breid die slice daarna gecontroleerd uit met timeout, outputoverschrijding, annulering en procesopruiming, en sluit pas daarna de HTTP-route aan. Behoud de vastgestelde veiligheidsgrenzen in elke stap. Werk met korte codefragmenten met een exacte bestandslocatie en begrijpelijke uitleg; voer per stap een gerichte controle uit voordat de volgende stap begint. De algemene leerwerkwijze staat in [AGENTS.md](../../AGENTS.md#leerbegeleiding).

## Vastgestelde afspraken

- F03 voert alleen tools zonder dynamische parameters uit. F04 voegt parametercontrole en placeholdervervanging toe.
- De beheerder configureert de executable en vaste argumenten. Native executables zijn toegestaan; scripts alleen via een expliciet geconfigureerde interpreter met vaste argumenten. Geen automatische shell-, `.bat`- of `.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet wijzigen.
- De standaardtimeout is 10.000 ms en het maximum 60.000 ms.
- stdout en stderr worden samen begrensd op 1 MiB. Bij overschrijding wordt de procesboom beëindigd en volgt een expliciete fout.
- Requestbody maximaal 16 KiB; maximaal vier gelijktijdige uitvoeringen. Bij volle capaciteit direct afwijzen.
- `process-wrap` 10.0.1 is toegevoegd met de Tokio-feature. De executor gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Gerichte Windows-tests controleren dat subprocessen verdwijnen na timeout, outputoverschrijding en annulering, ook bij een eerder afgerond direct child. Linux slaagt na de ESRCH-correctie in CI-run #22; run #21 blijft als historisch foutbewijs vastgelegd.
- Expliciete annulering per uitvoering loopt via `oneshot::Receiver<()>`. Alleen een ontvangen bericht annuleert; het droppen van de zender zonder bericht niet. Shutdown gebruikt een gedeeld `watch`-signaal en wacht op capaciteitsvrijgave. HTTP vertaalt annulering naar 503; MCP en clientdisconnectbeleid blijven open.
- HTTP-mapping: 400 invoerfout, 404 onbekende tool, 413 te grote requestbody, 503 volle capaciteit, 504 timeout, 502 outputlimiet. Een proces met niet-nul exitcode geeft HTTP 200 met `success: false`.

## Todo

- [x] Gedeelde kernresultaten en fouttypen definiëren.
- [x] Command-opbouw met geconfigureerde executable, losse vaste argumenten, gesloten stdin en gepipete stdout/stderr voorbereiden.
- [x] Basisexecutor voor parameterloze tools maken; executable en vaste argumenten komen uit de beheerdersconfiguratie en de serveromgeving/werkdirectory worden geërfd.
- [x] Semaphore voor maximaal vier gelijktijdige uitvoeringen en een helper voor de gecombineerde outputlimiet toevoegen.
- [x] Timeout, outputlimiet en concurrentielimiet implementeren; subprocessopruiming in gerichte Windows-tests controleren.
- [x] `process-wrap` integreren voor Unix-procesgroepen en Windows Job Objects.
- [x] `stop_child` alleen laten beëindigen bij ontbrekende exitstatus; regressietest voor een al afgerond proces uitvoeren op Windows.
- [x] Testbinary als child een subprocess laten starten en het PID vastleggen; procescontrole delen via `process_checks::assert_stopped`.
- [x] Subprocessopruiming en capaciteitsvrijgave bij timeout en outputoverschrijding op Windows testen.
- [x] Windows-scenario testen waarin het directe child eerder dan het subprocess eindigt.
- [x] Expliciete annulering met `oneshot` en `tokio::select!` implementeren; signaal, gesloten kanaal en subprocessopruiming met capaciteitsvrijgave op Windows testen.
- [x] Server-shutdown signaleren, nieuwe uitvoeringen weigeren en capaciteitsvrijgave afwachten; Windows-subprocesstest, wachtgedrag en Ctrl+C zonder actieve tools controleren.
- [x] Future-drop en Tokio-taakafbreking op Windows afzonderlijk testen; expliciete annulering/shutdown als normale stoproute en `KillOnDrop` als vangnet vastleggen. Linux valt onder de platformcontrole hieronder.
- [x] Linux-processtatuscontroles via `/proc` voorbereiden en de zeven opruimscenario's ook daarop aansluiten; zombieprocessen als beëindigd onderscheiden.
- [x] Unix-groepsopruiming na een afgeronde leider en een groepsbewaker bij future-drop voorbereiden op basis van de lokale wrapperimplementatie.
- [x] De `/proc`-race uit run #21 corrigeren: ENOENT/NotFound en ESRCH als beëindigd herkennen; overige lees-/formaatfouten behouden en deterministisch testen.
- [x] Linux-code na ESRCH-correctie opnieuw via Ubuntu-CI controleren; run #22 toont Ubuntu en Windows Success.
- [x] Windows-procescontrole delen en begrenzen: sinds de review native proceshandles, maximaal 5 s herhaald controleren met intervallen van 50 ms. De eerdere tasklist-implementatie is vervangen; geen harde OS-processtartgarantie.
- [x] POST /tools/{toolname} aansluiten met gedeelde state, uitsluitend `{}`, 16 KiB bodylimiet, resultaatmodel en vaste foutresponses; Windows-routetests en echte POST controleren.
- [x] Gerichte tests voor succesvolle/niet-succesvolle exit, timeout, outputlimiet en capaciteitsvrijgave toevoegen.
- [x] Reproduceerbare subprocess-helper met PID-bestand, optionele grote output en vroegtijdig afgerond child toevoegen; gecontroleerde annulering laten wachten op een leesbaar PID.
- [x] HTTP-tests voor succes, niet-nul exit, 400/404/413/500/502/503/504, bodygrens, parameterafwijzing en shutdownafwijzing uitvoeren op Windows.
- [x] Testmodules scheiden van productiecode onder `src/tests/` met behoud van logische modulepaden; volledige suite opnieuw controleren.
- [x] Uitvoeringslogging met registry-toolnaam, uitkomst/foutcategorie en duur toevoegen; handmatig en met geïsoleerde tests controleren dat clientwaarden, argumentwaarden en procesoutput ontbreken.
- [x] Vijf echte HTTP-verzoeken testen: vier actieve procesbomen, vijfde direct 503, daarna vier succesvolle responses en aantoonbaar capaciteitsherstel.
- [x] Hetzelfde serverafsluitpad als Ctrl+C via integratie testen met actief TCP-request, 503-response, beëindigde child/subprocess en gesloten luisterpoort. Signaalbron geïnjecteerd; geen OS-Ctrl+C verstuurd.
- [x] Windows-formatter-/lintmeldingen oplossen; formatter, Clippy, volledige tests en build op 2026-11-02 uitvoeren.

## Acceptatiecriteria

- [x] HTTP laat executable en argumenten uitsluitend uit de registry komen; alleen `{}` en parameterloze tooldefinities worden toegelaten. Succes, invoerafwijzing en parameterafwijzing zijn getest.
- [x] Stdout/stderr worden gelijktijdig en begrensd gelezen.
- [x] Timeout, te grote output en volle capaciteit hebben voorspelbaar gedrag in de geteste Windows-scenario's, inclusief subprocessen en capaciteitsvrijgave.
- [x] Procesopruiming en regressiecontrole op beide platforms geverifieerd via CI-run #22; platformbeperkingen blijven vastgelegd.

## Bewijs en koppelingen

- GitHub issue: nog vast te leggen.
- Branch: feature/F03-procesuitvoering.
- Pull request/review en merge: nog te bevestigen; de aangeleverde screenshot toont een push-run, geen mergebewijs.
- Historische verificatie (2026-11-02): run #21 Ubuntu faalt in `stopt_subproces_bij_annulering` op ESRCH tijdens `/proc`-lezen; Windows groen. Na de lokale correctie slagen op Windows formatter, Clippy, build en 62 tests (4 proceshelpers genegeerd). Drie nieuwe regressietests bewaken verdwenen processen, statussen en het behouden van echte fouten. Daarna bevestigt de aangeleverde screenshot van run #22 Success op beide platforms (Ubuntu 45 s, Windows 2 m 51 s; totale run 2 m 55 s). Exacte CI-testtotalen zijn niet zichtbaar.
- [Featurenotitie](../../notities/F03-procesuitvoering.md).

## Afronden

- [x] Relevante Windows-controles geslaagd.
- [x] Ubuntu- en Windows-CI na de racecorrectie geslaagd in run #22.
- [x] Documentatie en notitie bijgewerkt tot de gecontroleerde stand van 2026-11-02.
- [x] Voortgang bijgewerkt tot de gecontroleerde stand van 2026-11-02.
- [ ] PR gemerged; pas daarna status Klaar.

## Reviewcorrecties — 2026-11-03

De review verwerkt lege configuratiewaarden, enkele toollookup, Engelse testnamen en gedeelde testhelpers. Windows-proceschecks gebruiken nu handles in plaats van tasklist; geen verhoogde timeout. De Windows-suite slaagt met 64 tests en 4 proceshelpers. De eerdere CI-run #22 is historisch bewijs; Linux-controle van deze reviewstand staat open. De eigenaar meldt F03 afgerond, maar review/merge is niet afzonderlijk gecontroleerd. De featurestatus blijft daarom Review. Details en eindcontrole staan in de featurenotitie.
