# ADR-001 — Rust en Axum

Status: gekozen op 2026-10-19.

## Context

Eerdere projecten zijn gebouwd met Laravel en .NET. In RustTest zijn routing, JSON, foutafhandeling, ownership, modules, State/Arc, configuratie, procesuitvoering, timeouts, logging en tests verkend.

## Besluit

CommandHub wordt gebouwd in Rust met Axum en Tokio. Voor MCP gebruiken we de officiële Rust SDK; de exacte integratie wordt onderzocht in F00.

## Reden

Verdieping in ownership, expliciete foutafhandeling, concurrency en procesbeheer. Het oefenproject heeft voldoende vertrouwen gegeven om hiermee verder te gaan.

## Overwogen alternatief

C# met ASP.NET Core: bekendere ontwikkelomgeving en minder nieuwe taalconcepten. Laravel is bekend, maar staat niet in de expliciete talenlijst van deze opdracht.

## Gevolgen

Extra tijd voor Rust en MCP-leren inplannen. Kleine features en gerichte tests gebruiken. Rust maakt commandouitvoering niet automatisch veilig; uitvoeringsbeleid en OS-procesbeheer blijven ontwerpwerk.

