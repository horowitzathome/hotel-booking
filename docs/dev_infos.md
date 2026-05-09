# This file contains general infors for developers

## SQLx

SQLx also needs sqlx-cli installed once: cargo install sqlx-cli --no-default-features --features rustls,postgres

## DB Migration

just db-migrate-add name=add_booking_notes   # creates 0002_add_booking_notes.up.sql + .down.sql

just db-migrate # applies it

just db-migrate-revert # rolls it back if needed

## Where to look at things (URLs + useful curl)

Once `just compose-up` is running, these endpoints are all reachable on `localhost`.

### Quick reference

| What | URL | Notes |
|---|---|---|
| App — Swagger UI | http://localhost:8080/swagger-ui/ | Interactive OpenAPI explorer |
| App — OpenAPI JSON | http://localhost:8080/api-docs/openapi.json | Machine-readable spec |
| App — health | http://localhost:8080/health | k8s liveness/readiness probe |
| App — metrics | http://localhost:8080/metrics | Prometheus text format |
| App — domain API | http://localhost:8080/api/v1/... | All resources under `/api/v1/` |
| Grafana | http://localhost:3000 | Anonymous Admin, no login form (dev only) |
| Jaeger UI | http://localhost:16686 | Search service `rental-api` |
| Prometheus UI | http://localhost:9090 | `Status → Targets`, `Graph` tab |
| Loki API | http://localhost:3100 | No UI — query via Grafana or curl |

### Following one request through all three signals

Every response includes an `x-request-id: <uuid>` header. That same UUID appears in:
- Loki log lines (in the embedded JSON span fields, lifted to a derived field link in Grafana)
- Jaeger span tags (on the root HTTP span)

Workflow: copy the UUID from the curl response → in Grafana, query Loki `{container="rental-api"}` and filter for the UUID → click the auto-generated TraceID link → land in Jaeger on the matching trace.

```bash
# Trigger a request and capture its id
RID=$(curl -sf -i http://localhost:8080/api/v1/countries | awk -F': ' '/^x-request-id/ {print $2}' | tr -d '\r')
echo "request_id=$RID"

# Find the corresponding logs in Loki
curl -sG "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode "query={container=\"rental-api\"} |= \"$RID\""

# Find the trace in Jaeger (by request_id tag — only works if the span was instrumented)
curl -sG "http://localhost:16686/api/traces" \
  --data-urlencode "service=rental-api" \
  --data-urlencode "tags={\"request_id\":\"$RID\"}"
```

### Loki (logs) via curl

```bash
# All label names currently known to Loki
curl -s http://localhost:3100/loki/api/v1/labels

# Possible values for one label
curl -s http://localhost:3100/loki/api/v1/label/level/values
curl -s http://localhost:3100/loki/api/v1/label/container/values

# Recent log lines for the app (LogQL — pipe lets you filter further)
curl -sG http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={container="rental-api"}' \
  --data-urlencode 'limit=20'

# Only ERROR-level lines from the app
curl -sG http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={container="rental-api", level="ERROR"}'

# Free-text search across all containers
curl -sG http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={container=~".+"} |= "migrations applied"'

# Loki's own readiness
curl -s http://localhost:3100/ready
```

In Grafana: **Explore → data source = Loki → query box = `{container="rental-api"}`**.

### Prometheus (metrics) via curl

```bash
# Scrape targets and their up/down state
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job:.labels.job, health}'

# Instant query
curl -sG http://localhost:9090/api/v1/query \
  --data-urlencode 'query=http_requests_total'

# Range query (last 5 minutes, 15s step)
curl -sG http://localhost:9090/api/v1/query_range \
  --data-urlencode 'query=rate(http_requests_total[1m])' \
  --data-urlencode "start=$(date -u -v-5M +%s)" \
  --data-urlencode "end=$(date -u +%s)" \
  --data-urlencode 'step=15'

# Raw scrape (what Prometheus sees from the app)
curl -s http://localhost:8080/metrics | head -40
```

Useful PromQL starters once data has accumulated:
- `sum by (endpoint, status) (rate(http_requests_total[1m]))`
- `histogram_quantile(0.95, sum by (le, endpoint) (rate(http_requests_duration_seconds_bucket[5m])))`
- `process_resident_memory_bytes{job="rental-api"}`

### Jaeger (traces) via curl

```bash
# Which services have reported any spans
curl -s http://localhost:16686/api/services | jq

# Operations seen for one service
curl -s 'http://localhost:16686/api/operations?service=rental-api' | jq

# Last 20 traces, any operation
curl -sG http://localhost:16686/api/traces \
  --data-urlencode 'service=rental-api' \
  --data-urlencode 'limit=20' | jq '.data[] | {traceID, spans: (.spans | length)}'

# Traces filtered by span tag (e.g. by HTTP route)
curl -sG http://localhost:16686/api/traces \
  --data-urlencode 'service=rental-api' \
  --data-urlencode 'tags={"http.route":"/api/v1/bookings"}'
```

In the UI: pick **Service: rental-api**, **Find Traces**, click a row to see the nested span timeline (root HTTP span → service span → repository span → SQL).

### Grafana

The `compose-up` config provisions three datasources (Prometheus default, Loki, Jaeger) and disables the login form. Open http://localhost:3000 and you land directly in the UI as Admin. Everything is then driven from **Explore** (per-datasource) or **Dashboards** (none provisioned yet — easy follow-up).

```bash
# List provisioned datasources programmatically
curl -s http://localhost:3000/api/datasources | jq '.[] | {name, type, url}'

# Health check
curl -s http://localhost:3000/api/health
```

### Generating traffic

```bash
just api-traffic           # one-shot — hits /api/v1/bookings + /api/v1/countries
just api-health            # one-shot — hits /health
while sleep 1; do curl -s http://localhost:8080/api/v1/countries > /dev/null; done   # sustained load
```
