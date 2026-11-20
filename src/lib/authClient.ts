import { createInternalNeonAuth, type NeonAuth } from '@neon/auth';
import { BetterAuthReactAdapter } from '@neon/auth/react/adapters';

type ReactAdapterBuilder = ReturnType<typeof BetterAuthReactAdapter>;
type ReactAdapterInstance = ReturnType<ReactAdapterBuilder>;
export type ExoAuthInstance = NeonAuth<ReactAdapterInstance>;

let cachedUrl: string | null = null;
let cachedAuth: ExoAuthInstance | null = null;

export function getAuthInstance(authUrl: string): ExoAuthInstance {
  if (!cachedAuth || cachedUrl !== authUrl) {
    cachedUrl = authUrl;
    cachedAuth = createInternalNeonAuth(authUrl, {
      adapter: BetterAuthReactAdapter(),
    });
  }
  return cachedAuth;
}
