# Newsletter signup service — HTTP API

This document describes the REST-style API exposed by the Actix Web server (see [`src/startup.rs`](../src/startup.rs)). Use it to build a frontend or generate client code.

## Base URL and configuration

- Default local base URL: **`http://localhost:8000`** (see `application.port` and `application.external_hostname` in configuration).
- The frontend app URL used in Stripe redirects and password-reset emails is **`web_app_host`** (e.g. `http://localhost:3000` in [`configuration/base.yaml`](../configuration/base.yaml)).

All paths below are relative to the API base URL.

## Conventions

| Item | Detail |
|------|--------|
| **JSON** | Request bodies use `Content-Type: application/json` unless noted. |
| **UUIDs** | Path segments like `{id}` are UUID strings (e.g. `550e8400-e29b-41d4-a716-446655440000`). |
| **CORS** | Server allows any origin; methods `GET`, `POST`, `PUT`, `DELETE`; headers include `Authorization`, `Accept`, `Content-Type`. |
| **Empty success bodies** | Many endpoints return **`200 OK`** with an **empty body** (no JSON). |

## Authentication

Protected routes require a **JWT** in the header:

```http
Authorization: Bearer <token>
```

Tokens are **HS512** JWTs. Successful **`POST /sign_up`**, **`POST /login`**, and **`GET /forgot_password/otp/{otp}`** return a `LoginResponse` that includes `token`.

| Claim / field | Meaning |
|---------------|---------|
| `user_id` | Authenticated user id (string). |
| `group` | `"USER"` or `"ADMIN"`. |

Token lifetime is **1 hour** (`exp` ≈ `iat` + 3600 seconds). Invalid or missing tokens yield **`401 Unauthorized`** on routes that extract `Claims`.

**Path vs token:** Several routes put `user_id` in the URL. The server checks that this value matches the JWT’s `user_id` (and for admin routes, that the caller is an admin). Mismatches return **`401 Unauthorized`**.

---

## Public endpoints (no `Authorization`)

### `GET /health_check`

Liveness probe. Returns **`200 OK`** (empty body).

---

### `POST /sign_up`

Creates a user (default role **USER**), a linked subscriber row, and returns a session token.

**Body — `SignUp`**

| Field | Type | Notes |
|-------|------|--------|
| `email_address` | string | Normalized/stripped by server; must be unique. |
| `password` | string | Hashed server-side. |
| `name` | string | Must satisfy **ValidName** (see [Validation](#validation-rules)). |

**Responses**

| Status | Meaning |
|--------|---------|
| `200` | JSON **`LoginResponse`** |
| `400` | Validation failed |
| `409` | Email already registered |
| `500` | Server error |

---

### `POST /login`

**Body — `LogIn`**

| Field | Type |
|-------|------|
| `email_address` | string |
| `password` | string |

**Responses**

| Status | Meaning |
|--------|---------|
| `200` | JSON **`LoginResponse`** |
| `400` | Wrong credentials or user missing |

---

### `POST /forgot_password`

Starts password reset: if the email exists, creates a one-time passcode and emails a link `{web_app_host}/reset-password?otp=<passcode>`.

**Body — `ForgotPassword`**

| Field | Type |
|-------|------|
| `email_address` | string |

**Responses**

| Status | Body |
|--------|------|
| `200` | `{}` (always, including unknown email — no enumeration) |

---

### `GET /forgot_password/otp/{otp}`

Exchanges a one-time passcode (from the reset link) for a normal session.

**Path**

| Param | Description |
|-------|-------------|
| `otp` | Opaque passcode string |

**Responses**

| Status | Meaning |
|--------|---------|
| `200` | JSON **`LoginResponse`** |
| `400` | Invalid, expired, or already used OTP |

---

## Authenticated — session and password

### `POST /check_token/{user_id}`

Verifies the bearer token belongs to **`user_id`** in the path.

**Responses:** `200` + `{}` if ok; `401` if path id ≠ token or invalid token.

---

### `POST /check_admin_token/{user_id}`

Same as above, but **`401`** unless the token’s `group` is **ADMIN** and path matches.

---

### `POST /reset_password`

**Requires:** Bearer token for the **same** user as `email_address`.

**Body — `ResetPassword`**

| Field | Type |
|-------|------|
| `email_address` | string |
| `old_password` | string |
| `new_password` | string |

**Responses:** `200` empty; `400` wrong old password; `401` user mismatch; `500` on failure.

---

### `POST /reset_password_from_forgot_password`

Used after **`GET /forgot_password/otp/{otp}`** — caller must send the **new** JWT.

**Body — `ResetPasswordFromForgotPassword`**

| Field | Type |
|-------|------|
| `user_id` | string (must match JWT) |
| `new_password` | string |

**Responses:** `200` + `{}`; `401` / `400` / `500` as applicable.

---

## Subscribers

### `POST /subscribers`

Creates a subscriber for the authenticated user.

**Body — `OverTheWireCreateSubscriber`**

| Field | Type | Notes |
|-------|------|--------|
| `name` | string | ValidName |
| `email_address` | string | ValidEmail |
| `user_id` | string | **Must equal** JWT `user_id` |

**Responses:** `200` + `{}`; `400` validation; `401` user_id mismatch; `500` error.

---

### `GET /subscribers`

Query subscriber(s). **Requires** at least one of `user_id` or `email`.

| Query | Behavior |
|-------|----------|
| `?user_id=<uuid>` only | Subscriber for that user id (must own record). |
| `?email=<addr>` only | Lookup by email (must own record). |
| Both | Match by user id + email. |

**Responses:** `200` + **`OverTheWireSubscriber`**; `404` not found; `401` not owner; `404` if neither query param provided.

---

### `GET /subscribers/{id}`

`id` = subscriber UUID. Returns **`OverTheWireSubscriber`** if the subscriber’s `user_id` matches the JWT.

---

### `GET /subscribers/{id}/subscriptions`

Lists subscriptions for that subscriber. Caller must own the subscriber (`user_id` match).

**Response:** `200` + **`OverTheWireSubscription[]`**; `401` / `400` / `404` as applicable.

---

## Subscriptions

### `GET /subscriptions/{id}`

`id` = subscription UUID. Authorized if the subscription’s subscriber belongs to the JWT user.

**Response:** `200` + **`OverTheWireSubscription`**; `404` not found.

---

### `PUT /subscriptions/{id}`

Updates subscription fields. Body **`OverTheWireSubscription`** must match the existing subscription id and pass validation (name/email format; **`subscriber_id`** must belong to JWT user).

**Responses:** `200` + `{}`; `400` / `401` / `404` / `500`.

---

### `DELETE /subscriptions/{id}`

Cancels the subscription (DB + Stripe). Idempotent for already-cancelled: returns `200` + `{}`.

**Responses:** `200` + `{}`; `401` / `404` / `500` (e.g. Stripe failure rolls back).

---

## Checkout (Stripe)

### `POST /checkout/{user_id}`

Creates a Stripe Checkout session and returns a URL for redirect.

**Path:** `user_id` must match JWT.

**Body — `CreateCheckoutSession`**

| Field | Type |
|-------|------|
| `price_lookup_key` | string (stored with session; server resolves Stripe price from **`subscription.subscription_type`**) |
| `subscription` | **`OverTheWireCreateSubscription`** |

**`OverTheWireCreateSubscription`**

| Field | Type |
|-------|------|
| `subscriber_id` | string (UUID of subscriber) |
| `subscription_name` | string |
| `subscription_mailing_address_line_1` | string |
| `subscription_mailing_address_line_2` | string or `null` |
| `subscription_city` | string |
| `subscription_state` | string |
| `subscription_postal_code` | string |
| `subscription_email_address` | string |
| `subscription_type` | `"Digital"` or `"Paper"` |

**Response:** `200` + **`CreateStripeSessionRedirect`**: `{ "location": "<stripe checkout url>" }`

**Errors:** `400` / `401` / `500`.

---

### `POST /checkout/{user_id}/session/{session_id}`

Completes checkout after Stripe redirect (typically from success URL with `session_id`). Creates the subscription record, notifies, etc.

**Path:** `user_id` must match JWT; `session_id` is the Stripe Checkout Session id.

**Response:** `200` + `{}`; `401` / `404` / `500`.

---

### `POST /checkout/{user_id}/manage`

Opens Stripe Customer/Billing Portal. Requires subscriber to have a **`stripe_customer_id`** (e.g. after checkout).

**Response:** `200` + **`CreateStripeSessionRedirect`** (`location` = portal URL); `400` if no customer id; `401` / `500`.

---

## Admin (JWT must be **ADMIN**; path `user_id` / `admin_user_id` must match caller)

### `GET /admin/subscribers/{user_id}`

**Response:** `200` + array of **`OverTheWireSubscriber`** (all subscribers).

---

### `GET /admin/subscriptions/{user_id}`

**Response:** `200` + array of **`OverTheWireSubscription`** (all subscriptions).

---

### `GET /admin/users/{user_id}`

**Response:** `200` + **`OverTheWireUser[]`**

**`OverTheWireUser`**

| Field | Type |
|-------|------|
| `user_id` | UUID |
| `email_address` | string |
| `user_group` | `"USER"` or `"ADMIN"` |

---

### `POST /admin/users/{admin_user_id}/promote/{target_user_id}`

Promotes `target_user_id` to admin. Cannot promote self (`400`).

**Response:** `200` empty; `401` / `400` / `500`.

---

### `POST /admin/users/{admin_user_id}/demote/{target_user_id}`

Demotes an admin to user.

**Response:** `200` empty; `401` / `400` / `500`.

---

## Shared JSON types

### `LoginResponse`

```json
{
  "user_id": "<string>",
  "token": "<jwt>",
  "expires_on": 1234567890
}
```

`expires_on` is a Unix timestamp (seconds).

---

### `OverTheWireSubscriber`

```json
{
  "id": "<uuid>",
  "name": "<string>",
  "email_address": "<string>",
  "user_id": "<string>",
  "stripe_customer_id": "<string> | null"
}
```

---

### `OverTheWireSubscription`

Returned by list/get endpoints. `subscription_mailing_address_line_2` is a **string** in this shape (may be empty).

```json
{
  "id": "<uuid>",
  "subscriber_id": "<uuid>",
  "subscription_name": "<string>",
  "subscription_mailing_address_line_1": "<string>",
  "subscription_mailing_address_line_2": "<string>",
  "subscription_city": "<string>",
  "subscription_state": "<string>",
  "subscription_postal_code": "<string>",
  "subscription_email_address": "<string>",
  "subscription_creation_date": "<ISO-8601 datetime>",
  "subscription_cancelled_on_date": "<ISO-8601 datetime> | null",
  "subscription_anniversary_day": 1,
  "subscription_anniversary_month": 12,
  "subscription_renewal_date": "<string>",
  "active": true,
  "subscription_type": "Digital",
  "stripe_subscription_id": "<string>"
}
```

`subscription_type` is **`"Digital"`** or **`"Paper"`** (serde default enum tagging).

---

### `CreateStripeSessionRedirect`

```json
{
  "location": "https://..."
}
```

---

## Validation rules

| Rule | Applies to |
|------|------------|
| **ValidEmail** | Emails validated with `validator` email rules (non-empty, well-formed). |
| **ValidName** | Non-empty after trim; max **256 graphemes**; characters `/ ( ) " < > \ { }` forbidden. |

---

## Typical frontend flows

1. **Register:** `POST /sign_up` → store `token` → `Authorization` on later calls.
2. **Login:** `POST /login` → store `token`.
3. **Subscribe (paid):** `POST /checkout/{user_id}` with `CreateCheckoutSession` → redirect browser to `location` → on success, Stripe sends user to `web_app_host` success URL → call `POST /checkout/{user_id}/session/{session_id}` with returned `session_id`.
4. **Manage billing:** `POST /checkout/{user_id}/manage` → open `location` in browser.
5. **Forgot password:** `POST /forgot_password` → user opens emailed link → `GET /forgot_password/otp/{otp}` → `POST /reset_password_from_forgot_password` with new password.

---

## Not exposed in `startup.rs`

The module [`src/routes/stripe_webhook.rs`](../src/routes/stripe_webhook.rs) exists but is **not** mounted on the HTTP server in `startup.rs`. Only the routes listed above are served by the current application.
