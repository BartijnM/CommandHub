# Huidige voortgang

Bijgewerkt: 2026-11-04.

## Actief

[F04 — Dynamische parameters en padbeleid](features/F04-parameters.md) is in uitvoering. De eigenaar meldt F03 na review afgesloten en macOS-ondersteuning toegevoegd; de sessie gaat verder met F04. De actuele Git-branch en merges zijn niet door de assistent gecontroleerd. Beoogde branch: feature/F04-parameters.

## Aanwezig

De eerste dynamische keten is aangesloten: JSON-object → registry → parametervalidatie → afzonderlijke argumenten → bestaande procesuitvoering → HTTP-response. Parameterloze tools blijven werken met een leeg object. src/validation.rs controleert verplichte/onbekende velden, stringtypen, NUL, allowed_values en enkele risicovolle prefixes. allowed_roots maakt van een stringparameter een padparameter: bestaande absolute paden worden gecanonicaliseerd, op padcomponenten tegen roots gecontroleerd en als gecanonicaliseerd argument doorgegeven.

De configuratie controleert niet-lege waardelijsten zonder NUL en niet-lege rootlijsten met bestaande absolute directories. Eén optionele placeholder mag uitsluitend als laatste argument voorkomen; combinatie met vaste argumenten die met - of / beginnen wordt afgewezen. Timeout, capaciteit, outputbudget en shutdown gebruiken de bestaande kern. De loggingwrapper wordt gedeeld door beide uitvoeringspaden.

## Verificatie

De eigenaar heeft de voorgestelde controles lokaal op Windows uitgevoerd en telkens succes gemeld. Eén tussentijdse volledige suite telde 107 geslaagde tests; daarna zijn meer tests toegevoegd. Het huidige totale aantal is niet aangeleverd. De laatste volledige suite vóór de nieuwe dynamische routetests is als geslaagd gemeld; daarna is routes::tests:: met de nieuwe tests als geslaagd gemeld.

Aanwezig zijn configuratie-, validatie-, argument-, service- en HTTP-routetests, inclusief echte rustc-uitvoering en dynamische bodies van exact 16 KiB en één byte daarboven. Deze nieuwe routertests gebruiken geen TCP. De Unix-symlinktest wordt op Windows niet gecompileerd en is voor F04 nog niet via CI bevestigd. Oudere groene F07-CI is geen bewijs voor F04.

De assistent heeft alleen bestanden gelezen en op uitdrukkelijk verzoek deze documentatie bijgewerkt. Geen shell-, Cargo- of Git-commando’s uitgevoerd. Formatter, Clippy en drieplatform-CI zijn voor de eindstand nog niet bevestigd.

## Open en volgende stap

Eerst HTTP-padtests toevoegen. Daarna dynamische logging gericht controleren en GET /tools uitbreiden/beoordelen: het huidige schema toont nog geen enum voor allowed_values en geen additionalProperties: false. Vervolgens laatste review van toolafhankelijk optiebeleid, optionele argumenten, padgrenzen en testhelpers. Prefixcontroles zijn geen algemene garantie tegen alle toolopties; canonicalisatie biedt geen sandbox of bescherming tegen wijzigingen tussen controle en gebruik.

Nog controleren: niet-bestaande configuratieroot, aanvullende padgrenzen en Windows-symlinkgedrag. Paden zonder gebruikte placeholder krijgen momenteel geen padcontrole in validate_parameters; de padcontrole gebeurt bij argumentopbouw. Review moet bepalen of ongebruikte parameterdefinities worden geweigerd of ook volledig gevalideerd.

Daarna cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings en cargo test --locked; CI op Windows, Linux en macOS. Eventuele waarschuwingen na de stapsgewijze refactor moeten nog worden opgelost. Documentatie bij afronding opnieuw nalopen, PR-review en merge; F04 is nog niet Klaar.

Werkvorm bij hervatten: één kort implementatiefragment met exact bestand/plaats en Rust-uitleg; eigenaar voert in en controleert. Samenhangende tests mogen gebundeld. Geen commando’s of broncodewijzigingen door de assistent zonder specifieke opdracht.
