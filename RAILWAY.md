# Deploying Terrier on Railway

This guide walks through every service in a Railway project for Terrier. Use [`example/.env.railway`](example/.env.railway) as your variable checklist.

## Architecture

```text
Railway project
├── terrier          ← main app (Dockerfile), public URL
├── Postgres         ← Railway template
├── Redis            ← Railway template
├── minio            ← Docker image, optional public URL for file downloads
└── (external) OIDC  ← Keycloak, Auth0, ScottyLabs IdP, etc.
```

Terrier needs all four data/auth dependencies at runtime. OIDC stays outside Railway.

---

## 1. Create the Railway project

1. [railway.app](https://railway.app) → **New Project** → **Empty Project**.
2. Connect your GitHub repo (or deploy from this repo later).

---

## 2. PostgreSQL

### Add the service

1. In the project canvas: **+ New** → **Database** → **PostgreSQL**.
2. Railway provisions Postgres and exposes connection variables.

### Variables (on the Postgres service)

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | Full connection string (public) |
| `DATABASE_PRIVATE_URL` | Preferred for service-to-service traffic |

### Connect to Terrier

On the **Terrier** service → **Variables**:

```bash
DATABASE_URL=${{Postgres.DATABASE_PRIVATE_URL}}
```

Use **Reference** in the UI to pick the Postgres service. Prefer the private URL so traffic stays on Railway’s internal network.

### Notes

- Terrier runs migrations on startup; no manual migration step.
- Default database name is often `railway` — that is fine.

---

## 3. Redis

### Add the service

1. **+ New** → **Database** → **Redis**.

### Connect to Terrier

On **Terrier** → **Variables**:

```bash
REDIS_URL=${{Redis.REDIS_URL}}
```

Sessions are stored in Redis. Terrier must reach Redis before accepting logins.

### Notes

- Use the variable Railway provides (`REDIS_URL` or `REDIS_PRIVATE_URL` if shown); reference it from Terrier.
- If Redis has a password, it is already included in the URL.

---

## 4. MinIO (object storage)

Railway has no MinIO template. Run MinIO as its own service.

### Add the service

1. **+ New** → **Empty Service** → name it `minio`.
2. **Settings** → **Source** → **Docker Image**: `minio/minio:latest`
3. **Settings** → **Deploy** → **Custom Start Command** (must include the `minio` binary):

   ```bash
   minio server /data --console-address ":9001"
   ```

   If you see `The executable 'server' could not be found`, Railway is running `server` as the program name. Use the full command above, or **clear** the start command and rely on the image default (`minio server /data`).

4. **Settings** → **Volumes** → mount a volume at `/data` (required for persistent storage).

5. **Variables** on the MinIO service:

   | Variable | Example |
   |----------|---------|
   | `MINIO_ROOT_USER` | `terrier` |
   | `MINIO_ROOT_PASSWORD` | (strong secret, `openssl rand -base64 24`) |

6. **Networking** → generate a **public domain** for MinIO if browsers must load files directly (banners, resumes, icons). Example: `minio-production.up.railway.app`. Target port **9000** (S3 API).

### Connect to Terrier

On **Terrier** → **Variables**:

```bash
MINIO_ENDPOINT=http://minio.railway.internal:9000
MINIO_PUBLIC_ENDPOINT=https://<minio-public-domain>
MINIO_ROOT_USER=terrier
MINIO_ROOT_PASSWORD=<same-as-minio-service>
MINIO_BUCKET=terrier
```

- **`MINIO_ENDPOINT`**: internal hostname. Railway private DNS is typically `<service-name>.railway.internal`. Confirm the MinIO service name matches (e.g. `minio`).
- **`MINIO_PUBLIC_ENDPOINT`**: the **public** MinIO URL (no trailing path). Terrier builds public file URLs from this + bucket + object key.
- **`MINIO_BUCKET`**: Terrier creates the bucket and a public-read policy for specific paths on first boot.

### MinIO console (optional)

- Console listens on port `9001` inside the container.
- For admin UI access, you can expose port 9001 via a second public domain or use `mc` locally; not required for Terrier to work.

### Alternative: external S3

Instead of MinIO on Railway, use **Cloudflare R2**, **AWS S3**, or another S3-compatible API. Set `MINIO_ENDPOINT` to the provider’s API endpoint and `MINIO_PUBLIC_ENDPOINT` to the public bucket/CDN URL. Credentials map to `MINIO_ROOT_USER` / `MINIO_ROOT_PASSWORD`.

---

## 5. Terrier (main application)

Use **either** GHCR (faster, recommended if you already publish images) **or** build from GitHub.

### Option A: Deploy from GHCR (no Railway build)

CI pushes images on every push to `main` (see [`.github/workflows/build-and-push.yml`](.github/workflows/build-and-push.yml)):

```text
ghcr.io/scottylabs/terrier-production:latest
```

Also tagged as `main-<sha>` for pinned deploys.

1. **+ New** → **Empty Service** → name it `terrier`.
2. **Settings** → **Source** → **Docker Image**:
   ```text
   ghcr.io/scottylabs/terrier-production:latest
   ```
3. **If the package is private** (ScottyLabs org GHCR):
   - **Settings** → **Registry credentials** (or service variables):
     - Username: your GitHub username
     - Password: GitHub PAT with `read:packages` (not `GITHUB_TOKEN` from Actions — that only works in CI)
4. **Settings** → **Deploy**:
   - Health check path: `/health`
   - Health check timeout: `300`
5. Set all env vars below (same as a source build). Still set `IP=0.0.0.0`; Railway still injects `PORT`.

**Pros:** No 15–20 min Railway build; deploys track `main` after CI finishes.  
**Cons:** Image updates only when CI pushes; you need registry access to `scottylabs/terrier-production`. For a fork, build and push to your own `ghcr.io/<you>/terrier-production` and use that image instead.

Use the full image reference:

```text
ghcr.io/scottylabs/terrier-production:latest
```

**Important:** Older GHCR images may only contain `/app/public/` (no `/app/terrier`) due to a Dockerfile bug. If you see `the executable /app/terrier could not be found`, you need an image built after the Dockerfile fix (merge to `main` and wait for CI), or deploy via **Option B** (build from repo on Railway).

To redeploy a new release: push to `main` (CI publishes `:latest`), then **Redeploy** the Railway service or enable **Watch Paths** / webhook if configured.

### Option B: Build from GitHub (Dockerfile on Railway)

1. **+ New** → **GitHub Repo** (or **Empty Service** + connect repo).
2. Select this repository.
3. **Settings** → **Build**:
   - Builder: **Dockerfile**
   - Dockerfile path: `Dockerfile`
4. **Settings** → **Deploy**:
   - Health check path: `/health`
   - Health check timeout: `300` (first build can take 15–20+ minutes)

### Public URL

1. **Terrier** service → **Settings** → **Networking** → **Generate Domain** (or add custom domain).
2. Set:

   ```bash
   APP_BASE_URL=https://<your-terrier-domain>
   ```

### Required variables

Copy from [`example/.env.railway`](example/.env.railway). Minimum set:

| Variable | Value |
|----------|--------|
| `APP_BASE_URL` | Public Terrier URL |
| `IP` | `0.0.0.0` |
| `DATABASE_URL` | `${{Postgres.DATABASE_PRIVATE_URL}}` |
| `REDIS_URL` | `${{Redis.REDIS_URL}}` |
| `MINIO_ENDPOINT` | `http://minio.railway.internal:9000` |
| `MINIO_PUBLIC_ENDPOINT` | Public MinIO URL |
| `MINIO_ROOT_USER` | Same as MinIO service |
| `MINIO_ROOT_PASSWORD` | Same as MinIO service |
| `MINIO_BUCKET` | `terrier` |
| `OIDC_ISSUER` | Your IdP issuer URL |
| `OIDC_CLIENT_ID` | Client ID from IdP |
| `OIDC_CLIENT_SECRET` | Client secret from IdP |
| `ADMIN_EMAILS` | Comma-separated admin emails |
| `RUST_LOG` | `info,terrier=debug` |

Do **not** set `PORT` unless debugging; Railway sets it automatically.

### Service dependencies (recommended)

On **Terrier** → **Settings**, ensure deploy order waits for Postgres and Redis (Railway often handles this when you use variable references). MinIO should be running before first Terrier deploy that uploads files.

### First deploy

- Build installs Rust deps and runs `dx bundle` — expect **10–20+ minutes** on the first build.
- Check **Deploy Logs** for migration success: `Database migrations completed successfully`.

---

## 6. OIDC (external — not on Railway)

Terrier does not host login. Configure an existing OpenID Connect provider.

### IdP client settings

| Setting | Value |
|---------|--------|
| Redirect URI (web) | `{APP_BASE_URL}/auth/callback` |
| Redirect URI (mobile) | `terrier://auth/callback` |
| Scopes | `openid`, `email`, `profile` |

### Terrier variables

```bash
OIDC_ISSUER=https://<issuer-url>
OIDC_CLIENT_ID=<client-id>
OIDC_CLIENT_SECRET=<client-secret>
```

`OIDC_ISSUER` must be the issuer URL your provider documents (often ends with `/realms/<name>` for Keycloak).

---

## 7. Optional: analytics & AI

| Variable | When to set |
|----------|-------------|
| `POSTHOG_KEY` | PostHog project API key |
| `POSTHOG_HOST` | e.g. `https://us.i.posthog.com` |
| `OPENROUTER_API_KEY` | AI features that call OpenRouter |

---

## Variable wiring cheat sheet

Paste into Terrier’s **Raw Editor** after replacing placeholders:

```bash
APP_BASE_URL=https://YOUR_TERRIER_DOMAIN
IP=0.0.0.0

DATABASE_URL=${{Postgres.DATABASE_PRIVATE_URL}}
REDIS_URL=${{Redis.REDIS_URL}}

MINIO_ENDPOINT=http://minio.railway.internal:9000
MINIO_PUBLIC_ENDPOINT=https://YOUR_MINIO_PUBLIC_DOMAIN
MINIO_ROOT_USER=terrier
MINIO_ROOT_PASSWORD=YOUR_MINIO_SECRET
MINIO_BUCKET=terrier

OIDC_ISSUER=https://YOUR_IDP_ISSUER
OIDC_CLIENT_ID=terrier
OIDC_CLIENT_SECRET=YOUR_OIDC_SECRET

ADMIN_EMAILS=you@example.com

RUST_LOG=info,terrier=debug
RUST_BACKTRACE=1
```

Service names in `${{...}}` must match your Railway service names (e.g. `Postgres` vs `postgres` — use the UI reference picker to avoid typos).

---

## Health checks & troubleshooting

| Symptom | Check |
|---------|--------|
| `the executable /app/terrier could not be found` | GHCR image built before Dockerfile fix — redeploy after CI pushes a new `:latest`, or build from GitHub (Option B). Clear any custom **Start Command** on Railway. |
| Deploy stuck / unhealthy | `IP=0.0.0.0`, health path `/health`, logs for crash on startup |
| `Failed to load configuration` | Missing required env var |
| `Failed to connect to database` | `DATABASE_URL` reference, Postgres running |
| `Failed to connect to Redis` | `REDIS_URL`, Redis running |
| MinIO errors on boot | MinIO service up, matching credentials, reachable at `MINIO_ENDPOINT` |
| Login redirect fails | `APP_BASE_URL` matches public URL; IdP redirect URI exact match |
| Uploaded images 403/broken | `MINIO_PUBLIC_ENDPOINT` is the **browser-facing** URL |

### Logs

**Terrier** → **Deployments** → latest → **View Logs**. Useful lines:

- `Configuration loaded successfully`
- `Database connected successfully`
- `Database migrations completed successfully`
- `Starting server at 0.0.0.0:<port>`

---

## Local `.env` vs Railway

- **`.env`** in the repo root is gitignored; use it only for local development.
- **Railway** stores variables in the dashboard (or via `railway variables` CLI).
- [`example/.env.railway`](example/.env.railway) is a **template** — copy values into Railway, not into git with real secrets.

---

## Related docs

- [SETUP.md](SETUP.md) — Docker Compose & NixOS production setup
- [CONTRIBUTING.md](CONTRIBUTING.md) — local development with devenv
- [example/.env.prod](example/.env.prod) — self-hosted Docker reference
