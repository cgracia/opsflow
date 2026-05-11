# Safety and Scope

## Project scope

OpsFlow is a personal open-source exploration, built on personal time using personal infrastructure. It is not affiliated with, sponsored by, or derived from any employer, company, or commercial project.

## Data

No company data, customer data, internal endpoints, or proprietary workflows are used in this project. All sample incidents, entities, tickets, runbooks, telemetry, deployments, and operational events are synthetic and invented for demonstration. The seed data describes a fictional company ("Meridian Logistics") operating fictional devices in a fictional scenario.

## Production readiness

The system is not designed for production use and has no security review. It runs locally via Docker Compose with default credentials intended for development only.

## Governance engine

The governance engine blocks EXECUTE actions in v1. This is an architectural design decision that reflects the project's posture — the system investigates and recommends, humans decide and execute. It is not a security guarantee. The governance layer runs in-process and has no authentication, authorization, or audit logging beyond the Langfuse trace.

## Contributions

Contributions that introduce real-world data, internal company information, or proprietary operational patterns are out of scope for this project. All contributions must use synthetic or publicly documented data only.
