# WriteSpace Deployment Guide

## Overview

WriteSpace is a Rust + Axum blogging platform deployed as a serverless function on Vercel using `@vercel/rust`. The backend compiles to a single binary that runs inside Vercel's serverless infrastructure via `vercel_runtime`. Static frontend assets are served separately through Vercel's static file hosting.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│                   Vercel                     │
│                                              │
│  ┌──────────────┐    ┌───────────────────┐   │
│  │ Static Files │    │ Serverless Rust   │   │
│  │ (public/*)   │    │ (api/main.rs)     │   │
│  │              │    │                   │   │
│  │ index.html   │    │ /api/auth/*       │   │
│  │ blogs.html   │    │ /api/posts/*      │   │
│  │ js/app.js    │    │ /api/admin/*      │   │
│  └──────────────┘    └────────┬──────────┘   │
│                               │              │
└───────────────────────────────┼──────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  Neon PostgreSQL      │
                    │  (managed database)   │
                    └──────────────────────┘
```

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Vercel CLI](https://vercel.com/docs/cli) (`npm i -g vercel`)
- A [Vercel](https://vercel.com) account
- A [Neon](https://neon.tech) PostgreSQL database (or any PostgreSQL provider)

---

## Neon PostgreSQL Setup

1. **Create a Neon account** at [https://neon.tech](https://neon.tech).

2. **Create a new project** and note the connection string. It will look like:

   ```
   postgres://username:password@ep-example-123456.us-east-2.aws.neon.tech/dbname?sslmode=require
   ```

3. **Enable connection pooling** (recommended for serverless). In the Neon dashboard, go to your project settings and copy the pooled connection string.

4. **No manual migration is needed.** The application runs migrations automatically on startup using `sqlx::migrate!("./migrations")`. The migration file `migrations/001_initial.sql` creates the `users` and `posts` tables along with required indexes.

5. **Verify connectivity** by running the application locally (see Local Development below).

---

## Environment Variables

The following environment variables are required for deployment:

| Variable | Required | Description | Example |
|---|---|---|---|
| `DATABASE_URL` | **Yes** | PostgreSQL connection string | `postgres://user:pass@host/db?sslmode=require` |
| `JWT_SECRET` | **Yes** | Secret key for signing JWT tokens. Must be at least 16 characters. Use a strong random string in production. | `a-very-long-random-secret-key-here` |
| `DEFAULT_ADMIN_USERNAME` | No | Username for the auto-seeded admin account. Defaults to `admin`. | `admin` |
| `DEFAULT_ADMIN_PASSWORD` | No | Password for the auto-seeded admin account. Defaults to `admin123`. Must be at least 8 characters. | `MySecureAdminPass!` |
| `RUST_LOG` | No | Controls log verbosity. | `writespace_rust=debug,tower_http=debug,info` |

### Security Notes

- **Never commit `.env` files** to version control. The `.gitignore` already excludes `.env`.
- **Change `JWT_SECRET`** from the example value. Use a cryptographically random string of at least 32 characters.
- **Change `DEFAULT_ADMIN_PASSWORD`** immediately after first deployment. The default `admin123` is insecure.
- **Use `sslmode=require`** in your `DATABASE_URL` when connecting to remote PostgreSQL instances.

---

## Vercel Deployment

### 1. Link Your Repository

```bash
# From the project root
vercel link
```

Follow the prompts to link to your Vercel account and project.

### 2. Configure Environment Variables

Set environment variables in the Vercel dashboard or via CLI:

```bash
vercel env add DATABASE_URL
vercel env add JWT_SECRET
vercel env add DEFAULT_ADMIN_USERNAME
vercel env add DEFAULT_ADMIN_PASSWORD
```

Alternatively, configure them in the Vercel dashboard under **Project Settings → Environment Variables**. Set them for all environments (Production, Preview, Development) or scope them as needed.

### 3. Deploy

```bash
# Preview deployment
vercel

# Production deployment
vercel --prod
```

### 4. Verify

After deployment, verify the API is working:

```bash
# Health check — list posts (public endpoint)
curl https://your-project.vercel.app/api/posts

# Login with default admin
curl -X POST https://your-project.vercel.app/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

---

## vercel.json Explanation

```json
{
  "version": 2,
  "framework": null,
  "builds": [
    {
      "src": "api/main.rs",
      "use": "@vercel/rust"
    },
    {
      "src": "public/**",
      "use": "@vercel/static"
    }
  ],
  "routes": [
    {
      "src": "/api/(.*)",
      "dest": "api/main.rs"
    },
    {
      "src": "/(.*)",
      "dest": "public/$1"
    }
  ]
}
```

| Section | Purpose |
|---|---|
| `builds[0]` | Compiles `api/main.rs` using `@vercel/rust`, which builds the Rust binary and wraps it as a serverless function. The binary entry point calls `vercel_runtime::run()` to handle incoming requests. |
| `builds[1]` | Serves all files under `public/` as static assets via `@vercel/static`. |
| `routes[0]` | Routes all `/api/*` requests to the compiled Rust serverless function. This covers auth, posts, and admin endpoints. |
| `routes[1]` | Routes all other requests to the `public/` directory for static HTML, JS, and CSS files. |

The `"framework": null` setting tells Vercel not to auto-detect a framework, since we are using a custom Rust build.

---

## Local Development

### 1. Set Up Environment

Copy the example environment file and fill in your values:

```bash
cp .env.example .env
```

Edit `.env` with your local or remote PostgreSQL connection string:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/writespace
JWT_SECRET=your-super-secret-jwt-key-change-in-production
DEFAULT_ADMIN_USERNAME=admin
DEFAULT_ADMIN_PASSWORD=admin123
RUST_LOG=writespace_rust=debug,tower_http=debug,info
```

### 2. Set Up Local PostgreSQL

If using a local PostgreSQL instance:

```bash
createdb writespace
```

The application will run migrations automatically on startup.

### 3. Run Locally with Vercel CLI

```bash
vercel dev
```

This simulates the Vercel serverless environment locally. The Rust binary is compiled and served at `http://localhost:3000`.

### 4. Run Tests (if applicable)

```bash
cargo check
cargo clippy
```

---

## Troubleshooting

### Cold Starts

Serverless Rust functions on Vercel experience cold starts when the function has not been invoked recently. During a cold start:

1. The Rust binary is loaded into memory.
2. The application connects to the PostgreSQL database.
3. Migrations are checked (no-op if already applied).
4. The default admin user is seeded (no-op if already exists).

**Mitigation strategies:**

- **Use Neon's connection pooling.** Neon provides a pooled connection endpoint that reduces connection establishment time. Use the pooled URL in `DATABASE_URL`.
- **Keep the database pool small.** The application uses `max_connections(5)` which is appropriate for serverless. Larger pools waste resources during cold starts.
- **Minimize migration checks.** The `sqlx::migrate!()` macro embeds migrations at compile time. The runtime check is fast — it only verifies the migration table.
- **Consider Vercel's Pro plan.** Pro plans offer faster cold starts and the ability to configure function regions closer to your database.
- **Co-locate function and database.** Deploy your Vercel function in the same region as your Neon database to minimize network latency. Set the region in Vercel project settings.

### Common Errors

#### `DATABASE_URL environment variable is required`

The `DATABASE_URL` environment variable is not set. Add it to your Vercel project environment variables or your local `.env` file.

#### `JWT_SECRET must be at least 16 characters long`

Your `JWT_SECRET` is too short. Use a random string of at least 16 characters (32+ recommended).

#### `Failed to connect to database`

- Verify your `DATABASE_URL` is correct and the database is accessible.
- Ensure `sslmode=require` is included for remote databases.
- Check that your database provider allows connections from Vercel's IP ranges (Neon allows all IPs by default).
- If using Neon, ensure the project is not suspended (free tier projects suspend after inactivity).

#### `Failed to run database migrations`

- The database user must have permissions to create tables and indexes.
- If you manually modified the database schema, the migration checksums may not match. Check the `_sqlx_migrations` table.

#### Build Failures on Vercel

- Ensure `Cargo.toml` lists all required dependencies.
- The `@vercel/rust` builder compiles the binary target defined in `Cargo.toml` (`[[bin]] path = "api/main.rs"`).
- Check Vercel build logs for specific Rust compiler errors.
- Verify that the `sqlx` `migrate!()` macro can find the `migrations/` directory relative to `CARGO_MANIFEST_DIR`.

#### CORS Errors in Browser

The application configures CORS to allow all origins (`CorsLayer::new().allow_origin(Any)`). If you see CORS errors:

- Verify the API is responding (not returning a 500 error that lacks CORS headers).
- Check that the `Authorization`, `Content-Type`, and `Accept` headers are in the allowed headers list (they are by default).

---

## CI/CD Notes

### GitHub Integration

Vercel automatically deploys when you push to your connected GitHub repository:

- **Push to `main`** → Production deployment
- **Push to any other branch** → Preview deployment
- **Pull requests** → Preview deployment with a unique URL

### Build Caching

The `@vercel/rust` builder caches compiled dependencies between builds. Subsequent deployments are faster because only changed code is recompiled. If you experience stale builds, you can clear the build cache in the Vercel dashboard under **Project Settings → General → Build Cache**.

### Environment Variable Scoping

Vercel supports scoping environment variables to specific environments:

- **Production** — Used for `vercel --prod` deployments
- **Preview** — Used for branch/PR deployments
- **Development** — Used for `vercel dev`

Use separate databases for production and preview environments to avoid data conflicts. For example:

```
# Production
DATABASE_URL=postgres://user:pass@prod-host/writespace_prod?sslmode=require

# Preview
DATABASE_URL=postgres://user:pass@dev-host/writespace_preview?sslmode=require
```

### Recommended CI Pipeline

If you want to add CI checks before deployment, create a GitHub Actions workflow:

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check
      - run: cargo clippy -- -D warnings
```

This runs `cargo check` and `cargo clippy` on every push and pull request. Vercel deployment is handled separately by the Vercel GitHub integration.

### Secrets Management

- Never store secrets in `vercel.json` or commit them to the repository.
- Use Vercel's environment variable system for all sensitive configuration.
- Rotate `JWT_SECRET` periodically. Note that rotating the secret invalidates all existing JWT tokens, forcing users to log in again.
- Rotate `DEFAULT_ADMIN_PASSWORD` after initial setup by logging into the admin panel and creating a new admin account, or by updating the environment variable and redeploying (which only affects new seed operations — the password is only set if no default admin exists).

---

## Production Checklist

- [ ] `DATABASE_URL` points to a production PostgreSQL instance with SSL enabled
- [ ] `JWT_SECRET` is a strong random string (32+ characters)
- [ ] `DEFAULT_ADMIN_PASSWORD` is changed from the default value
- [ ] Vercel function region is co-located with the database region
- [ ] Database connection pooling is enabled (Neon pooled endpoint)
- [ ] Environment variables are scoped correctly (production vs preview)
- [ ] CORS configuration is reviewed (currently allows all origins — restrict in production if needed)
- [ ] Database backups are configured (Neon provides point-in-time recovery on paid plans)
- [ ] Monitoring and alerting are set up for the Vercel function and database