import { useState, type FormEvent } from 'react';
import { Lock, LogOut, ShieldAlert } from 'lucide-react';
import { getAuthInstance } from '../lib/authClient';

interface AuthGateProps {
  authConfigured: boolean;
  authUrl: string | null;
  signedInEmail: string | null;
  didPeer: string | null;
  accessDenied: boolean;
  onAuthSuccess: () => void;
  onSignOut: () => void;
}

export function AuthGate({
  authConfigured,
  authUrl,
  signedInEmail,
  didPeer,
  accessDenied,
  onAuthSuccess,
  onSignOut,
}: AuthGateProps) {
  const [mode, setMode] = useState<'signin' | 'signup'>('signin');
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (accessDenied && signedInEmail) {
    return (
      <div className="min-h-screen flex items-center justify-center px-5 py-12 bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2]">
        <div className="w-full max-w-md rounded-xl border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-7 sm:p-8">
          <div className="flex items-center gap-2.5 text-[#57534E] dark:text-[#9CA3AF] text-xs mb-4">
            <ShieldAlert className="w-4 h-4 shrink-0" />
            <span>Private Documentation</span>
          </div>

          <h1 className="font-display text-2xl font-semibold tracking-tight text-balance">
            Access not granted
          </h1>

          <p className="mt-3 text-sm leading-relaxed text-[#57534E] dark:text-[#9CA3AF]">
            Signed in as <strong className="text-[#1C1917] dark:text-[#ECEEF2]">{signedInEmail}</strong>.
            ExoNovel documentation is private and requires administrator approval.
          </p>

          {didPeer && (
            <div className="mt-4 p-3 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/60 dark:bg-[#111317]">
              <div className="text-[11px] font-medium text-[#57534E] dark:text-[#9CA3AF]">
                Authoritative did:peer
              </div>
              <div className="mt-1 font-mono text-xs break-all text-[#1C1917] dark:text-[#ECEEF2]">
                {didPeer}
              </div>
            </div>
          )}

          <div className="mt-6">
            <button
              type="button"
              onClick={onSignOut}
              className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] hover:bg-[#F3F0EA] dark:hover:bg-[#1E232D] transition-colors min-h-[44px]"
            >
              <LogOut className="w-4 h-4" />
              <span>Sign out</span>
            </button>
          </div>
        </div>
      </div>
    );
  }

  const handleEmailSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!authUrl) return;
    setError(null);
    setSubmitting(true);

    try {
      const client = getAuthInstance(authUrl).adapter;
      if (mode === 'signin') {
        const res = await client.signIn.email({ email: email.trim(), password });
        if (res.error) {
          throw new Error(res.error.message || 'Sign in failed.');
        }
      } else {
        const res = await client.signUp.email({
          email: email.trim(),
          password,
          name: name.trim() || email.split('@')[0],
        });
        if (res.error) {
          throw new Error(res.error.message || 'Account creation failed.');
        }
      }
      onAuthSuccess();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed.');
    } finally {
      setSubmitting(false);
    }
  };

  const handleSocial = async (provider: 'google' | 'github') => {
    if (!authUrl) return;
    setError(null);
    setSubmitting(true);
    try {
      const client = getAuthInstance(authUrl).adapter;
      const res = await client.signIn.social({
        provider,
        callbackURL: `${window.location.origin}/`,
      });
      if (res?.error) {
        throw new Error(res.error.message || `Could not sign in with ${provider}.`);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Social sign-in failed.';
      const inIframe = window.self !== window.top;
      if (inIframe && msg.toLowerCase().includes('popup was closed')) {
        setError(
          'AI Studio preview popup could not return to this sandbox because the popup landed on your default Google account (/u/0/) instead of this tab (/u/1/). Use Email/Password below to test in preview, or open AI Studio in a separate browser profile/container where exonomyapp@gmail.com is the primary account. Production on exonovel.exosystems.net redirects directly without this proxy.'
        );
      } else {
        setError(msg);
      }
      setSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center px-5 py-12 bg-[#FAF8F4] dark:bg-[#111317] text-[#1C1917] dark:text-[#ECEEF2]">
      <div className="w-full max-w-md rounded-xl border border-[#E5E0D5] dark:border-[#262B36] bg-[#FFFFFF] dark:bg-[#171A21] p-7 sm:p-8">
        <div className="flex items-center gap-2 text-xs text-[#57534E] dark:text-[#9CA3AF] mb-3">
          <Lock className="w-3.5 h-3.5 shrink-0" />
          <span>ExoNovel · Private Documentation</span>
        </div>

        <h1 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-balance">
          Sign in to continue
        </h1>
        <p className="mt-2 text-sm text-[#57534E] dark:text-[#9CA3AF] leading-relaxed">
          Documentation is restricted to authorized contributors and automatically resolves your{' '}
          <span className="font-mono text-xs">did:peer</span> identity.
        </p>

        {!authConfigured || !authUrl ? (
          <div className="mt-6 p-4 rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#F3F0EA]/70 dark:bg-[#111317] text-sm text-[#44403C] dark:text-[#D1D5DB]">
            Neon Auth is not yet deployed for this environment. Complete{' '}
            <code className="font-mono text-xs px-1.5 py-0.5 rounded bg-[#EAE5DB] dark:bg-[#1F2430]">
              neon deploy
            </code>{' '}
            with an authenticated Neon CLI / <code className="font-mono text-xs">NEON_API_KEY</code>{' '}
            to enable sign-in.
          </div>
        ) : (
          <>
            <div className="mt-6 grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => handleSocial('google')}
                disabled={submitting}
                className="inline-flex items-center justify-center gap-2.5 px-3.5 py-2 text-sm font-medium rounded-lg border border-[#DADCE0] dark:border-[#8E918F] bg-[#FFFFFF] dark:bg-[#131314] text-[#3C4043] dark:text-[#E3E3E3] hover:bg-[#F8FAFD] dark:hover:bg-[#1E1F20] hover:border-[#C4C7C5] dark:hover:border-[#A8AAAA] transition-colors whitespace-nowrap min-h-[42px] disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#1D4ED8]"
              >
                <svg
                  className="w-4 h-4 shrink-0"
                  viewBox="0 0 48 48"
                  aria-hidden="true"
                >
                  <path
                    fill="#EA4335"
                    d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
                  />
                  <path
                    fill="#4285F4"
                    d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
                  />
                  <path
                    fill="#FBBC05"
                    d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
                  />
                  <path
                    fill="#34A853"
                    d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
                  />
                </svg>
                <span>Google</span>
              </button>

              <button
                type="button"
                onClick={() => handleSocial('github')}
                disabled={submitting}
                className="inline-flex items-center justify-center gap-2.5 px-3.5 py-2 text-sm font-medium rounded-lg border border-[#24292F] dark:border-[#3B434B] bg-[#24292F] dark:bg-[#21262D] text-[#FFFFFF] hover:bg-[#1B1F23] dark:hover:bg-[#30363D] transition-colors whitespace-nowrap min-h-[42px] disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#1D4ED8]"
              >
                <svg
                  className="w-4 h-4 shrink-0"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    clipRule="evenodd"
                    d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                  />
                </svg>
                <span>GitHub</span>
              </button>
            </div>

            <div className="my-5 flex items-center gap-3 text-xs text-[#78716C] dark:text-[#6B7280]">
              <div className="h-px flex-1 bg-[#E5E0D5] dark:bg-[#262B36]" />
              <span>or email</span>
              <div className="h-px flex-1 bg-[#E5E0D5] dark:bg-[#262B36]" />
            </div>

            <form onSubmit={handleEmailSubmit} className="space-y-4">
              {mode === 'signup' && (
                <div>
                  <label className="block text-xs font-medium text-[#57534E] dark:text-[#9CA3AF] mb-1.5">
                    Name
                  </label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Author name"
                    className="w-full px-3.5 py-2.5 text-sm rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
                  />
                </div>
              )}

              <div>
                <label className="block text-xs font-medium text-[#57534E] dark:text-[#9CA3AF] mb-1.5">
                  Email
                </label>
                <input
                  type="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  className="w-full px-3.5 py-2.5 text-sm rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[#57534E] dark:text-[#9CA3AF] mb-1.5">
                  Password
                </label>
                <input
                  type="password"
                  required
                  minLength={8}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Minimum 8 characters"
                  className="w-full px-3.5 py-2.5 text-sm rounded-lg border border-[#E5E0D5] dark:border-[#262B36] bg-[#FAF8F4] dark:bg-[#111317] focus:outline-none focus:border-[#1D4ED8] dark:focus:border-[#60A5FA]"
                />
              </div>

              {error && (
                <div className="p-3 rounded-lg border border-red-300 dark:border-red-900/60 bg-red-50 dark:bg-red-950/40 text-xs text-red-700 dark:text-red-300">
                  {error}
                </div>
              )}

              <button
                type="submit"
                disabled={submitting}
                className="w-full px-4 py-2.5 text-sm font-medium rounded-lg bg-[#1D4ED8] text-white hover:bg-[#1E40AF] transition-colors min-h-[44px] disabled:opacity-50"
              >
                {submitting
                  ? 'Authenticating...'
                  : mode === 'signin'
                    ? 'Sign in'
                    : 'Create account'}
              </button>
            </form>

            <div className="mt-5 text-center text-xs text-[#57534E] dark:text-[#9CA3AF]">
              {mode === 'signin' ? (
                <button
                  type="button"
                  onClick={() => {
                    setError(null);
                    setMode('signup');
                  }}
                  className="hover:underline text-[#1D4ED8] dark:text-[#60A5FA]"
                >
                  Need an account? Create one
                </button>
              ) : (
                <button
                  type="button"
                  onClick={() => {
                    setError(null);
                    setMode('signin');
                  }}
                  className="hover:underline text-[#1D4ED8] dark:text-[#60A5FA]"
                >
                  Already have an account? Sign in
                </button>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
