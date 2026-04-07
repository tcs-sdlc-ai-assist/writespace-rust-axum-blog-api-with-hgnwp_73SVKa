# WriteSpace

A clean, minimal blogging platform built with **Rust**, **Axum**, and **PostgreSQL**, deployed as a serverless function on **Vercel**.

## Architecture

WriteSpace follows a serverless architecture where the Rust backend is compiled and deployed as a single Vercel serverless function. The frontend consists of static HTML pages with vanilla JavaScript that communicate with the backend via a JSON REST API.

```
┌─────────────────────────────────────────────────┐
│                   Vercel CDN                     │
│                                                  │
│  ┌──────────────┐       ┌─────────────────────┐  │
│  │ Static Files │       │  Rust Serverless Fn  │  │
│  │  (public/)   │       │   (api/main.rs)      │  │
│  │              │       │                      │  │
│  │  HTML/JS/CSS │       │  Axum Router         │  │
│  │              │       │  ├─ Auth Handlers     │  │
│  │              │       │  ├─ Post Handlers     │  │
│  │              │       │  ├─ Admin Handlers    │  │
│  │              │       │  └─ Middleware (JWT)  │  │
│  └──────────────┘       └──────────┬────────────┘  │
│                                    │              │
└────────────────────────────────────┼──────────────┘
                                     │
                              ┌──────▼──────┐
                              │  PostgreSQL  │
                              │  Database    │
                              └─────────────┘
```

## Tech Stack

| Layer       | Technology                          |
|-------------|-------------------------------------|
| Runtime     | Rust (2021 edition)                 |
| Web Framework | Axum 0.8                          |
| Database    | PostgreSQL via sqlx 0.8             |
| Auth        | JWT (jsonwebtoken) + bcrypt         |
| Logging     | tracing + tracing-subscriber        |
| CORS        | tower-http CorsLayer                |
| Deployment  | Vercel (via @vercel/rust builder)    |
| Frontend    | Static HTML + Tailwind CSS (CDN) + Vanilla JS |

## Folder Structure

```
writespace-rust/
├── api/
│   └── main.rs              # Vercel serverless entry point
├── migrations/
│   └── 001_initial.sql      # Database schema
├── public/
│   ├── index.html            # Landing page
│   ├── login.html            # Login page
│   ├── register.html         # Registration page
│   ├── blogs.html            # Blog listing page
│   ├── blog.html             # Single post view (post.html alias)
│   ├── write.html            # Create/edit post page
│   ├── admin.html            # Admin dashboard
│   ├── users.html            # User management (admin)
│   └── js/
│       └── app.js            # Shared frontend module (auth, API, UI)
├── src/
│   ├── lib.rs                # Library crate root
│   ├── config.rs             # Environment config loader
│   ├── db.rs                 # Database pool, migrations, admin seeding
│   ├── errors.rs             # AppError type implementing IntoResponse
│   ├── models.rs             # Database models, DTOs, JWT Claims
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs           # Login and registration handlers
│   │   ├── posts.rs          # CRUD handlers for blog posts
│   │   └── admin.rs          # Admin dashboard and user management
│   └── middleware/
│       ├── mod.rs
│       └── auth.rs           # JWT encode/decode, require_auth, require_admin, optional_auth
├── Cargo.toml
├── vercel.json               # Vercel build and routing config
├── .env.example              # Example environment variables
├── .gitignore
└── README.md
```

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [PostgreSQL](https://www.postgresql.org/) 14+
- [Vercel CLI](https://vercel.com/docs/cli) (for deployment)

### Environment Variables

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

| Variable                  | Required | Default   | Description                                      |
|---------------------------|----------|-----------|--------------------------------------------------|
| `DATABASE_URL`            | Yes      | —         | PostgreSQL connection string                     |
| `JWT_SECRET`              | Yes      | —         | Secret key for signing JWTs (min 16 characters)  |
| `DEFAULT_ADMIN_USERNAME`  | No       | `admin`   | Username for the auto-seeded admin account       |
| `DEFAULT_ADMIN_PASSWORD`  | No       | `admin123`| Password for the auto-seeded admin account (min 8 chars) |
| `RUST_LOG`                | No       | `info`    | Log level filter for tracing                     |

Example `.env`:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/writespace
JWT_SECRET=your-super-secret-jwt-key-change-in-production
DEFAULT_ADMIN_USERNAME=admin
DEFAULT_ADMIN_PASSWORD=admin123
RUST_LOG=writespace_rust=debug,tower_http=debug,info
```

### Database Setup

1. Create the PostgreSQL database:

```bash
createdb writespace
```

2. Migrations run automatically on application startup via `sqlx::migrate!()`. The migration file at `migrations/001_initial.sql` creates the `users` and `posts` tables along with required indexes.

3. On first startup, a default admin user is automatically seeded using the `DEFAULT_ADMIN_USERNAME` and `DEFAULT_ADMIN_PASSWORD` environment variables.

## Local Development

### Running Locally

Since the project is configured for Vercel serverless deployment, local development requires building and running the binary directly:

```bash
# Ensure your .env file is configured
source .env

# Build and run
cargo run
```

> **Note:** The binary entry point (`api/main.rs`) uses `vercel_runtime::run()` which is designed for the Vercel serverless environment. For local development, you may need to adapt the entry point or use the Vercel CLI dev server.

### Using Vercel CLI for Local Development

```bash
# Install Vercel CLI
npm i -g vercel

# Run the development server
vercel dev
```

This will start a local server that mimics the Vercel deployment environment, serving static files from `public/` and routing API requests to the Rust serverless function.

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Checking SQL Queries (Compile-Time)

If using sqlx compile-time query checking, ensure the database is running and `DATABASE_URL` is set:

```bash
cargo sqlx prepare
```

## Deployment

### Vercel Deployment

1. Install the Vercel CLI and log in:

```bash
npm i -g vercel
vercel login
```

2. Set environment variables in the Vercel dashboard or via CLI:

```bash
vercel env add DATABASE_URL
vercel env add JWT_SECRET
vercel env add DEFAULT_ADMIN_USERNAME
vercel env add DEFAULT_ADMIN_PASSWORD
```

3. Deploy:

```bash
vercel --prod
```

The `vercel.json` configuration handles:
- Building `api/main.rs` with the `@vercel/rust` builder
- Serving static files from `public/` with `@vercel/static`
- Routing `/api/*` requests to the Rust serverless function
- Routing all other requests to static files

## API Reference

### Authentication

All protected endpoints require a `Bearer` token in the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

### Endpoints

#### Auth

| Method | Path                | Auth     | Description              |
|--------|---------------------|----------|--------------------------|
| POST   | `/api/auth/login`   | None     | Login with credentials   |
| POST   | `/api/auth/register`| None     | Register a new account   |

**POST /api/auth/login**

```json
// Request
{ "username": "admin", "password": "admin123" }

// Response 200
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "uuid",
    "username": "admin",
    "display_name": "admin",
    "role": "admin"
  }
}
```

**POST /api/auth/register**

```json
// Request
{ "username": "newuser", "display_name": "New User", "password": "password123" }

// Response 201
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "uuid",
    "username": "newuser",
    "display_name": "New User",
    "role": "user"
  }
}
```

#### Posts

| Method | Path               | Auth       | Description                          |
|--------|--------------------|------------|--------------------------------------|
| GET    | `/api/posts`       | Optional   | List posts (3 if anonymous, all if authenticated) |
| GET    | `/api/posts/{id}`  | Optional   | Get a single post by ID              |
| POST   | `/api/posts`       | Required   | Create a new post                    |
| PUT    | `/api/posts/{id}`  | Required   | Update a post (owner or admin)       |
| DELETE | `/api/posts/{id}`  | Required   | Delete a post (owner or admin)       |

**POST /api/posts**

```json
// Request
{ "title": "My First Post", "content": "Hello, world!" }

// Response 201
{
  "id": "uuid",
  "title": "My First Post",
  "content": "Hello, world!",
  "created_at": "2024-01-01T00:00:00Z",
  "author": {
    "id": "uuid",
    "display_name": "admin",
    "role": "admin"
  }
}
```

**PUT /api/posts/{id}**

```json
// Request
{ "title": "Updated Title", "content": "Updated content." }

// Response 200
{
  "id": "uuid",
  "title": "Updated Title",
  "content": "Updated content.",
  "created_at": "2024-01-01T00:00:00Z",
  "author": {
    "id": "uuid",
    "display_name": "admin",
    "role": "admin"
  }
}
```

#### Admin (Requires admin role)

| Method | Path                   | Auth          | Description                |
|--------|------------------------|---------------|----------------------------|
| GET    | `/api/admin/stats`     | Admin         | Dashboard statistics       |
| GET    | `/api/admin/users`     | Admin         | List all users             |
| POST   | `/api/admin/users`     | Admin         | Create a new user          |
| DELETE | `/api/admin/users/{id}`| Admin         | Delete a user              |

**GET /api/admin/stats**

```json
// Response 200
{
  "total_posts": 10,
  "total_users": 5,
  "total_admins": 1,
  "recent_posts": [
    { "id": "uuid", "title": "Latest Post", "created_at": "2024-01-01T00:00:00Z" }
  ]
}
```

**POST /api/admin/users**

```json
// Request
{
  "username": "newadmin",
  "display_name": "New Admin",
  "password": "securepass123",
  "role": "admin"
}

// Response 201
{
  "id": "uuid",
  "username": "newadmin",
  "display_name": "New Admin",
  "role": "admin",
  "is_deletable": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Error Responses

All error responses follow a consistent format:

```json
{
  "error": "Human-readable error message"
}
```

| Status Code | Meaning                                    |
|-------------|--------------------------------------------|
| 400         | Bad Request — invalid input                |
| 401         | Unauthorized — missing or invalid token    |
| 403         | Forbidden — insufficient permissions       |
| 404         | Not Found — resource does not exist        |
| 409         | Conflict — resource already exists         |
| 500         | Internal Server Error                      |

## Frontend Pages

| Page               | Path               | Auth Required | Description                    |
|--------------------|--------------------|---------------|--------------------------------|
| Landing            | `/`                | No            | Welcome page with latest posts |
| Login              | `/login.html`      | No            | User login form                |
| Register           | `/register.html`   | No            | User registration form         |
| Blog Listing       | `/blogs.html`      | Yes           | All blog posts                 |
| Single Post        | `/post.html?id=`   | No            | View a single post             |
| Write/Edit Post    | `/write.html`      | Yes           | Create or edit a post          |
| Edit Post          | `/write.html?id=`  | Yes           | Edit an existing post          |
| Admin Dashboard    | `/admin.html`      | Admin         | Platform statistics            |
| User Management    | `/users.html`      | Admin         | Create and manage users        |

## License

Private — All rights reserved.