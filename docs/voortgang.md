# Huidige voortgang

Bijgewerkt: 2026-11-03.

## Actief

De code tot en met F03 is nagekeken en de reviewpunten zijn verwerkt. F02 is gemerged volgens de eerder vastgelegde bevestiging. De eigenaar meldt F03 afgerond; de documenten stonden nog op Review. Zonder Git- of externe controle wordt geen nieuwe mergebevestiging afgeleid.

## Aanwezig

De hoofdapplicatie laadt en valideert configuratie, bouwt een immutable registry en biedt GET /status, GET /tools en parameterloze POST /tools/{toolname}. De kern bewaakt timeout, gecombineerde output, capaciteit, annulering en shutdown. De geïsoleerde F00-proef staat los van de hoofdapp; F04-parameters en F05-MCP-integratie ontbreken nog.

## Reviewwijzigingen

- Alle test- en helperfuncties hebben Engelse namen; de exacte childselecties zijn mee hernoemd.
- Configuratie wijst lege toolnamen, parameternamen en commands af, inclusief alleen witruimte.
- ToolRegistry::find zoekt de tool eenmaal op; logging en uitvoering lenen dezelfde definitie.
- De subprocessscenario’s delen configuratie-, PID-, start- en capaciteitshelpers. De routetests delen POST-opbouw en JSON-uitlezen.
- Windows-procescontrole gebruikt OpenProcess en niet-blokkerende WaitForSingleObject via een Windows-only dev-dependency; OwnedHandle sluit het handle. Er wordt geen tasklist meer gestart. De controle blijft maximaal 5 s wachten op procesbeëindiging en verbergt geen toegangs-/API-fouten.
- Documentatie is gecorrigeerd voor F02-placeholdercontrole, F03-functionaliteit, helpernamen, platformopruiming en async-eigendom. Oudere test- en CI-resultaten zijn historie.

## Verificatie

De review begon op Windows met 62 geslaagde tests en 4 proceshelpers. De latere parallelle run faalde op 10 tijdslimieten; serieel faalden nog 2 TCP-tests op tasklist. Eén losse tasklist-aanroep duurde 721 ms. De precieze oorzaak van die wisselende externe aanroepen is niet vastgesteld; de nieuwe processtatuscontrole verwijdert die afhankelijkheid.

Na de correcties slagen de gerichte processtatussuite (4 tests) en configuratiesuite (14 tests), en de volledige parallelle suite met **64 passed, 0 failed, 4 ignored** (3,24 s). Formatter en Clippy slagen. De vier ignored functies worden door andere tests expliciet als child gestart; het zijn geen overgeslagen acceptatiescenario’s. Zie de [F03-notitie](../notities/F03-procesuitvoering.md) voor de eindcontrole.

Historisch: CI-run #21 vond een ESRCH-race tijdens /proc-lezen. Drie regressietests en de correctie zijn daarna lokaal gecontroleerd. De eerder aangeleverde screenshot van run #22 toont Ubuntu en Windows Success. Dat bewijst de toenmalige stand, niet de reviewwijzigingen van vandaag. Linux is niet lokaal uitgevoerd; de later aangeleverde screenshot bevestigt wel de nieuwe CI voor commit 105e69b; de lokale omgeving heeft alleen een Windows-target en geen beschikbare Linux/WSL-testomgeving volgens de eerdere controle.

## Nog open en volgende stap

De CI-screenshot van commit 105e69b bevestigt de refactor op Ubuntu en Windows. Commit en push de documentatie-update, laat de laatste CI slagen en bevestig daarna review/merge voordat de administratieve status naar Klaar gaat. Daarna volgt F04. F00-review/merge en de eerdere F01-repository-/Project-openpunten zijn niet opnieuw gecontroleerd. Externe links en actuele GitHub-status zijn niet opnieuw geverifieerd. De assistent voert geen Git-commando’s, commits, pushes of merges uit.

Eindcontrole (2026-11-03): cargo fmt --check en cargo clippy --all-targets --locked -- -D warnings slagen. cargo test --locked: 64 passed, 0 failed, 4 ignored, 3,00 s; ook de eerdere parallelle run na correctie slaagde (3,24 s). De gewone cargo build --locked kon target/debug/commandhub.exe niet vervangen (toegang geweigerd); proces 15952 draaide vanaf dat pad. De bestaande server is niet gestopt. De alternatieve build cargo build --locked --offline --target-dir target/review-build is daarna geslaagd (1 m 57 s). De nieuwe executable staat in target/review-build/debug/commandhub.exe; de draaiende server gebruikt nog de eerdere binary.

## CI-bevestiging refactor — 2026-11-03

De aangeleverde screenshot toont een geslaagde push-run voor commit `105e69b` op `refactor/na-feature-3`:

- Runstatus: Success; totale duur 1 m 50 s.
- Checks (ubuntu-latest): Success, 42 s.
- Checks (windows-latest): Success, 1 m 47 s.
- De lokale workflow voert formatter, Clippy met warnings als errors, tests en build uit. Exacte CI-testtotalen zijn niet zichtbaar en worden niet uit de screenshot afgeleid.
- Twee Node.js 20-verouderingswaarschuwingen bij actions/checkout@v4 en een Ubuntu-labelmelding zijn zichtbaar; de jobs zijn geslaagd. Workflowonderhoud blijft een afzonderlijk punt.
- Hiermee is de eerder open Linux-/CI-controle van de refactor bevestigd. Review/merge is niet zichtbaar en blijft te bevestigen. Deze documentatie-update volgt na de getoonde codecommit; controleer na push de CI van de laatste branchstand.
- Alleen documentatie aangepast op basis van het aangeleverde bewijs; geen tests opnieuw uitgevoerd en geen Git-commando’s uitgevoerd.
