# Requirements en MoSCoW

Requirements voor de eerste lokale oplevering, bijgewerkt op 2026-11-11. De prioriteit bepaalt de scope en de roadmap bepaalt de bouwvolgorde. Must = noodzakelijk; Should = belangrijk maar uitstelbaar; Could = optioneel; Won't = buiten deze oplevering.

| ID | Eis | Prioriteit | Feature |
| --- | --- | --- | --- |
| R01 | Configuratie bij opstarten laden en volledig valideren | Must | F02 |
| R02 | Nieuwe tools toevoegen zonder opnieuw compileren; herstart toegestaan | Must | F02 |
| R03 | Tools ontdekken en op naam opzoeken | Must | F02 |
| R04 | Geconfigureerde executable met vaste losse argumenten uitvoeren | Must | F03 |
| R05 | Stdout, stderr, exitcode, succes en duur teruggeven | Must | F03 |
| R06 | Timeout, annulering en opruimen van processen regelen | Must | F03 |
| R07 | Output, requestgrootte en gelijktijdige uitvoeringen begrenzen | Must | F03/F04 |
| R08 | Vereiste parameters, typen, onbekende velden en toegestane waarden controleren | Must | F04 |
| R09 | Dynamische argumenten zonder shellinterpolatie opbouwen | Must | F04 |
| R10 | Padparameters tot geconfigureerde toegestane locaties beperken | Must | F04 |
| R11 | MCP via HTTP aanbieden en met echte MCP-client testen | Must | F00/F05 |
| R12 | HTTP-routes voor ontdekken en uitvoeren beschikbaar maken | Must | F02/F03/F04 |
| R13 | Gestructureerde logging en consistente foutcategorieën | Must | Alle |
| R14 | Windows, Linux en macOS ondersteunen met platformgeschikte tools | Must | F03/F06/F07 |
| R15 | Gedeelde kern voor HTTP en MCP; modules hebben eigen verantwoordelijkheid | Must | Alle |
| R16 | Veilige lokale binding en protocolspecifieke HTTP-controles | Must | F00/F05 |
| R17 | README, architectuur, security, voorbeelden en demo opleveren | Must | Alle/F06 |
| R18 | API-key-authenticatie voor een expliciet ontworpen remote gebruiksscenario | Should | Later |
| R19 | CI op Windows, Linux en macOS | Should | F01/F06/F07 |
| R20 | YAML en hot-reload | Could | Later |
| R21 | Dashboard, database, jobqueue, WebSockets en distributed execution | Won't | Buiten eerste versie |

De huidige oplevering gebruikt loopbackbinding. Remote gebruik vraagt een afzonderlijk ontwerp voor authenticatie, transportbeveiliging en autorisatie bij R18.

## Definitie van klaar

- Acceptatiecriteria aangetoond.
- Normale én relevante fout- en grensgevallen gecontroleerd.
- Geen onbehandelde blokkades of bekende ernstige fouten.
- Documentatie en featurenotitie actueel.
- Relevante formatterings-, lint- en testcontroles geslaagd.
- Feature-PR beoordeeld en gemerged naar develop. Bij F06 wordt ook de opleverings-PR naar main afgerond.

## Traceerbaarheid

Het [testplan](testplan.md) bevat per requirement de implementatie, relevante tests, aanwezige resultaten en resterende verificatie. De Must-eisen hebben code- en testdekking. R17 is op 2026-11-11 bevestigd met de goedgekeurde README en werkende demo. De [demo](demo.md) is door de eigenaar als werkend bevestigd. Productgrenzen staan in [security](security.md).
