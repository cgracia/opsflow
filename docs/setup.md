# Setup Guide

This guide walks you through setting up OpsFlow locally using Docker Compose.

## Prerequisites

Before you begin, ensure you have:

1. **Docker** installed
   - Version 20.10 or higher
   - Run `docker --version` to check

2. **Docker Compose** installed
   - Version 2.0 or higher
   - Run `docker compose version` to check

3. **Git** installed
   - Run `git --version` to check

4. **At least 8GB RAM** available
   - OpsFlow runs multiple services that collectively require adequate memory

## Clone and Configure

1. **Clone the repository**

   ```bash
   git clone https://github.com/carlos/opsflow.git
   cd opsflow
   ```

2. **Copy the example environment file**

   ```bash
   cp .env.example .env
   ```

3. **Edit the environment variables**

   Open `.env` in your text editor and configure at minimum:

   ```bash
   # Set your LLM API key (OpenAI-compatible)
   LLM_API_KEY=sk-your-openai-key-here

   # Set Langfuse credentials (optional, for observability)
   LANGFUSE_PUBLIC_KEY=pk-your-key-here
   LANGFUSE_SECRET_KEY=sk-your-key-here
   ```

   **Note:** The demo uses synthetic data when seeding. You only need an LLM key if you're running actual investigations with LLM-powered reasoning.

## Start the Stack

1. **Start all services**

   ```bash
   docker compose up -d
   ```

   This will build and start:
   - API server (FastAPI)
   - PostgreSQL database
   - Qdrant vector database
   - ClickHouse
   - Redis
   - Langfuse (web + worker)
   - Prometheus
   - Grafana

2. **Check the status**

   ```bash
   docker compose ps
   ```

   You should see all services with status `healthy`:

   ```
   NAME                    STATUS
   opsflow-api             healthy (retries: 5)
   opsflow-postgres        healthy (retries: 10)
   opsflow-qdrant          healthy (retries: 5)
   opsflow-clickhouse      healthy (retries: 5)
   opsflow-redis           healthy (retries: 5)
   opsflow-langfuse-web    healthy (retries: 5)
   opsflow-prometheus      healthy (retries: 5)
   opsflow-grafana         healthy (retries: 5)
   ```

   Wait until all services show `healthy` before proceeding. This typically takes 2-3 minutes.

## Seed Data

OpsFlow comes with synthetic entity data and evidence for demonstration.

1. **Wait for API health check**

   ```bash
   curl -sf http://localhost:8000/api/v1/healthz
   ```

   Expected output: `{"status":"ok"}`

2. **Seed the database**

   ```bash
   curl -sf -X POST http://localhost:8000/api/v1/seed
   ```

   Expected output: `{"message":"Database seeded successfully with 3 fleets, 9 devices, and 12 evidence items"}`

## Run an Investigation

Now let's run a multi-signal investigation using synthetic data.

```bash
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids": {"ticket_id": "TCK-1001", "alert_id": "ALT-2001", "event_id": "EVT-3001"}}'
```

Expected response snippet:

```json
{
  "investigation_id": "INV-1001",
  "trace_id": "trace-abc123",
  "entity_context": {
    "account": null,
    "site": null,
    "fleet": null,
    "devices": [],
    "deployment": null,
    "software_revision": null
  },
  "evidence": [],
  "telemetry_analysis": null,
  "historical_analysis": null,
  "hypotheses": [],
  "governance_decision": null,
  "operator_briefing": "",
  "customer_response_draft": "",
  "created_at": "2026-05-06T10:00:00Z"
}
```

## View Observability

### Langfuse (Traces)

Langfuse provides full traceability of the investigation pipeline.

1. **Open Langfuse**

   Navigate to: http://localhost:3000

2. **Login**

   Default credentials:
   - Email: `opsflow@example.com`
   - Password: `admin`

3. **View a trace**

   - Click on "Traces" in the sidebar
   - Select the latest trace (e.g., `INV-1001`)
   - Observe the 7-phase investigation flow:
     1. Signal Ingestion
     2. Entity Resolution
     3. Evidence Retrieval
     4. Specialist Investigation
     5. Hypothesis Generation
     6. Governance Evaluation
     7. Output Generation

### Grafana (Dashboards)

Grafana shows operational metrics and service health.

1. **Open Grafana**

   Navigate to: http://localhost:3100

2. **Login**

   Default credentials:
   - Username: `admin`
   - Password: `admin`

3. **View dashboards**

   Grafana comes pre-configured with dashboards showing:
   - Service health
   - Request latency
   - Database queries
   - Vector search performance

## Stop the Stack

When you're done, stop all services:

```bash
docker compose down
```

To remove volumes (clear all data):

```bash
docker compose down -v
```

## Troubleshooting

### Services won't start

- Check Docker is running: `docker info`
- Verify ports 8000, 3000, 3100, 5432, 6333 are available
- Review logs: `docker compose logs`

### API health check fails

```bash
# Check API logs
docker compose logs api

# Restart API
docker compose restart api
```

### Database connection errors

- Ensure PostgreSQL is healthy: `docker compose ps postgres`
- Check DATABASE_URL in .env
- Restart services: `docker compose restart`

## Next Steps

- Read the [Investigation Flow](../README.md#investigation-flow) to understand how investigations work
- Explore the [Demo Walkthrough](./demo-walkthrough.md) for a narrative example
- Review the [API Documentation](http://localhost:8000/docs) for available endpoints

## Support

For issues or questions:
- Check the [README](../README.md)
- Review service logs: `docker compose logs -f`
