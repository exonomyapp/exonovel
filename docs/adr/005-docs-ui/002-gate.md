# 005-002. Private Docs Auth Gate & Admin Settings

Date: 2026-11-19
Status: Accepted (in implementation)
Parent: [005-docs-ui](../005-docs-ui.md)

## Decision

1. **Sign-In Gate (Neon Better Auth)**:
   - Docs are strictly private; `/api/docs` and `/api/doc` reject unauthenticated or unauthorized requests with `401`/`403`.
   - Unauthenticated users see a Sign-In page supporting Neon Better Auth with compact, side-by-side branded **Google** and **GitHub** buttons plus Email/Password sign-in/sign-up.
   - Signed-in users without docs permission see an explicit **Access Restricted** view showing their signed-in email and a sign-out button.
2. **Admin Account & Access Settings**:
   - The admin account (`ADMIN_EMAIL`, defaulting to `exonomyapp@gmail.com`) always has full docs access plus access to the **Settings** panel.
   - Inside Settings, the admin can add/remove allowed email addresses, toggle access on/off, and inspect user access records stored in Neon PostgreSQL (`docs_access_allowlist`).
   - On every valid login, the backend also ensures the user has an authoritative `did:peer` record in `user_identities` (resolving existing or minting a new `did:peer`).
