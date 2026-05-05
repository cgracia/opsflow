# Praxis

A control plane for running a technical support operation with AI in the loop, built to replace the spreadsheets, Slack threads, and tribal knowledge that break down at scale.

## The problem

Technical support at scale generates more signal than any manager can process manually. Tickets arrive from multiple channels, SLA clocks run in parallel, fleet telemetry produces anomalies overnight, and the team's accumulated knowledge lives across wiki pages that are hard to search well. The standard response is more headcount or more dashboards. Neither scales. What I needed was a reasoning layer that could sit across ticketing, documentation, device data, and team context, and surface the five things I actually need to act on before standup.

## What it does

- **Ticket triage and prioritisation** -- classifies incoming tickets by urgency, customer impact, and SLA proximity
- **SLA monitoring** -- tracks response and resolution deadlines, flags breaches before they happen
- **Daily operational briefings** -- generates a morning status report: what's on fire, what's due today, what changed overnight
- **Fleet anomaly detection** -- ingests device telemetry, identifies patterns that precede known failure modes
- **Performance review support** -- aggregates individual and team metrics over arbitrary time windows for quarterly reviews
- **Incident management** -- correlates ticket spikes with fleet events, drafts initial incident summaries
- **Knowledge retrieval** -- searches and synthesises answers from documentation, past tickets, and runbooks before the team has to ask

## Architecture

The system uses a multi-agent architecture in a Council of Experts pattern: each agent owns a domain (triage, fleet health, knowledge, scheduling, etc.) and contributes assessments that a coordinator agent reconciles into prioritised actions. Retrieval-augmented generation over a knowledge graph provides context. An eval harness validates agent outputs against historical decisions before they reach production. Every recommendation passes through a human-in-the-loop checkpoint; the system advises, it does not act autonomously.

## Why I built it

I was running a support function where the ticket backlog had grown beyond what manual triage could handle reliably. The existing tooling worked at low volume but fell apart under load. I built Praxis to bring those sources into a single reasoning layer so I could make better decisions faster, and so the team could stop answering the same questions from memory.

## Status

Alpha. This repository is a sanitised subset of the internal system, published as a demonstration of the architecture and approach. The operational data and integrations are not included.

## Future improvements

- Architecture diagram and sample `praxis status` output

## Licence

MIT
