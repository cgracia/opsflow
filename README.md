# opsflow

An AI control plane for running a technical support operation, built to replace the spreadsheets, Slack threads, and tribal knowledge that break down at scale.

## The problem

Technical support at scale generates more signal than any manager can process manually. Tickets arrive from multiple channels, SLA clocks run in parallel, fleet telemetry produces anomalies overnight, and the team's accumulated knowledge lives across wiki pages that are hard to search well. The standard response is more headcount or more dashboards. Neither scales. What is needed is a reasoning layer that can sit across ticketing, documentation, device data, and team context, and surface the five things that actually need action before standup.

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

I wanted a reasoning layer for support operations that I could actually run: something that surfaces the five things worth acting on before standup rather than adding another dashboard to watch. Built from personal experience with the problem; the integrations and operational data are not included in this repository.

## Status

Alpha, in active development. Architecture and agent design are stable; integrations and configuration are a work in progress.

## Licence

MIT
