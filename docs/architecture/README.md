# OpsFlow Architecture Documentation

C4 architecture models rendered with [LikeC4](https://likec4.dev).

## Views

| View | Description |
|------|-------------|
| **systemContext** | OpsFlow in its environment — actors and external dependencies |
| **containerView** | All containers: Python engine, data stores, observability stack |
| **investigationFlow** | The 7-phase investigation pipeline end to end |
| **observabilityStack** | Tracing, metrics, and dashboarding layer |

## Commands

This repository does not have a Node.js package manifest. Use LikeC4 CLI commands directly:

```bash
# Interactive development server
likec4 dev docs/architecture

# Build static site
likec4 build docs/architecture -o dist/architecture --base ./

# Export PNGs (dark theme, flat style)
likec4 export png docs/architecture -o docs/architecture/assets --theme dark --flat

# Validate and check formatting
likec4 validate docs/architecture && likec4 format docs/architecture --check
```

In this development environment, LikeC4 is installed through Home Manager. For plain npm environments, install LikeC4 globally with `npm install -g likec4` or use `npx likec4`.

## GitHub Pages

The interactive architecture site is built by `.github/workflows/architecture-pages.yml`.

After pushing the workflow, enable Pages in GitHub:

1. Open repository settings.
2. Go to **Pages**.
3. Set **Source** to **GitHub Actions**.

The site will be available at:

https://cgracia.github.io/opsflow/

## Assumptions

- **Signal sources** are modelled as a single external system. In production these would be distinct systems (Jira, PagerDuty, Datadog, etc.) but the current API receives signal IDs rather than direct integrations.
- **Langfuse, ClickHouse, and Redis** are grouped as the observability stack. They share the same lifecycle in docker-compose and exist to support AI trace collection and analytics.
- The **Governance Engine** currently runs in-process within the Python API container. It is modelled as a separate container for clarity of responsibility.
