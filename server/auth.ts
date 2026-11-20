import { createRemoteJWKSet, jwtVerify } from 'jose';
import type { Request, Response, NextFunction } from 'express';
import {
  isEmailAuthorizedForDocs,
  isPostgresConfigured,
  resolveOrMintDidPeer,
} from './db.ts';

let jwksCacheUrl: string | null = null;
let jwksSet: ReturnType<typeof createRemoteJWKSet> | null = null;

export function isAuthConfigured(): boolean {
  return Boolean(process.env.NEON_AUTH_BASE_URL && process.env.NEON_AUTH_JWKS_URL);
}

function getJwks() {
  const jwksUrl = process.env.NEON_AUTH_JWKS_URL;
  if (!jwksUrl) {
    throw new Error('NEON_AUTH_JWKS_URL is not configured.');
  }
  if (!jwksSet || jwksCacheUrl !== jwksUrl) {
    jwksSet = createRemoteJWKSet(new URL(jwksUrl));
    jwksCacheUrl = jwksUrl;
  }
  return jwksSet;
}

export interface VerifiedSessionUser {
  authUserId: string;
  email: string | null;
  name: string | null;
  didPeer: string | null;
  authorized: boolean;
  isAdmin: boolean;
}

export interface AuthenticatedRequest extends Request {
  authUser?: VerifiedSessionUser;
}

export async function verifyNeonAuthToken(token: string): Promise<VerifiedSessionUser> {
  const jwks = getJwks();
  const baseUrl = process.env.NEON_AUTH_BASE_URL;
  const expectedOrigin = baseUrl ? new URL(baseUrl).origin : undefined;

  const { payload } = await jwtVerify(token, jwks, {
    issuer: expectedOrigin ? [baseUrl!, expectedOrigin] : undefined,
  });

  const authUserId = typeof payload.sub === 'string' ? payload.sub : '';
  if (!authUserId) {
    throw new Error('Token missing subject.');
  }

  const email =
    typeof payload.email === 'string' ? payload.email.trim().toLowerCase() : null;
  const name = typeof payload.name === 'string' ? payload.name : null;

  let didPeer: string | null = null;
  if (isPostgresConfigured()) {
    const identity = await resolveOrMintDidPeer({
      authUserId,
      email,
      provider: 'neon-better-auth',
    });
    didPeer = identity.didPeer;
  }

  const access = await isEmailAuthorizedForDocs(email);

  return {
    authUserId,
    email,
    name,
    didPeer,
    authorized: access.authorized,
    isAdmin: access.isAdmin,
  };
}

export async function requireAuthenticatedUser(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> {
  if (!isAuthConfigured()) {
    res.status(503).json({
      error: 'Neon Auth is not configured yet. Deploy Neon services first.',
    });
    return;
  }

  const authHeader = req.headers.authorization || '';
  const token = authHeader.startsWith('Bearer ') ? authHeader.slice(7).trim() : '';

  if (!token) {
    res.status(401).json({ error: 'Authentication required.' });
    return;
  }

  try {
    req.authUser = await verifyNeonAuthToken(token);
    next();
  } catch (err) {
    console.error('JWT verification failed:', err);
    res.status(401).json({ error: 'Invalid or expired session token.' });
  }
}

export function requireDocsAccess(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): void {
  if (!req.authUser) {
    res.status(401).json({ error: 'Authentication required.' });
    return;
  }
  if (!req.authUser.authorized) {
    res.status(403).json({ error: 'Your account is not authorized to view documentation.' });
    return;
  }
  next();
}

export function requireAdminAccess(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): void {
  if (!req.authUser) {
    res.status(401).json({ error: 'Authentication required.' });
    return;
  }
  if (!req.authUser.isAdmin) {
    res.status(403).json({ error: 'Admin access required.' });
    return;
  }
  next();
}
