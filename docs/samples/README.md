# Sample investigation outputs

Real `POST /api/v1/investigations` responses captured from the stack running
this commit, after `POST /api/v1/seed`. They document the three governance
shapes the fixture scenarios exercise, so a reviewer can see the intended
behaviour without starting the stack. All data is synthetic.

| File | Scenario | Governance outcome |
|------|----------|--------------------|
| `investigation-a.json` | Meridian (strong evidence, high severity, enterprise) | `escalate`, escalation flagged, `customer_response_draft: null` (withheld; briefing says WITHHELD) |
| `investigation-b.json` | Northwind (weak evidence, low severity, standard) | `investigate`, drafts blocked below the confidence threshold, no root-cause claim |
| `investigation-c.json` | Atlas (strong evidence, medium severity, enterprise) | `communicate`, evidence-grounded `customer_response_draft` present |
| `investigation-unknown-ids.json` | unknown signal IDs | 404 naming each unknown ID — never an unrelated incident |

## Reproducing

```sh
docker compose up -d
curl -sf -X POST http://localhost:8000/api/v1/seed
curl -sf -X POST http://localhost:8000/api/v1/investigations \
  -H 'Content-Type: application/json' \
  -d '{"signal_ids":{"ticket_id":"TCK-1001","alert_id":"ALT-2001","event_id":"EVT-3001"}}'
```

(substitute the signal IDs per scenario: A `TCK-1001`/`ALT-2001`/`EVT-3001`,
B `TCK-2001`/`EVT-4001`, C `TCK-3001`/`ALT-4002`/`EVT-4002`).

`investigation_id`, `trace_id`, `created_at` and evidence ordering vary between
runs; the governance action, hypothesis confidences, evidence counts, draft
presence and briefing content are deterministic for the seeded fixtures.
