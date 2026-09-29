"use client";

import { type FormEvent, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ApiError, getVersion, loginWithToken } from "@/lib/api";
import { ProductLockup } from "@/components/brand/product-lockup";
import { Button } from "@/components/ui/button";
import { Callout } from "@/components/ui/callout";
import { Card } from "@/components/ui/card";
import { ErrorState } from "@/components/ui/error-state";
import formStyles from "@/styles/forms.module.css";

export default function AccesoPage() {
  const router = useRouter();
  const [token, setToken] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [httpsOk, setHttpsOk] = useState(true);

  useEffect(() => {
    setHttpsOk(
      typeof window === "undefined" ||
        window.location.protocol === "https:" ||
        window.location.hostname === "127.0.0.1" ||
        window.location.hostname === "localhost",
    );
    void getVersion()
      .then(() => router.replace("/"))
      .catch(() => {
        /* sin sesión: permanecer en acceso */
      });
  }, [router]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await loginWithToken(token.trim());
      router.replace("/");
    } catch (err) {
      if (err instanceof ApiError && err.status === 503) {
        setError("Esta instancia no requiere token (auth deshabilitada).");
        router.replace("/");
      } else if (err instanceof ApiError && err.status === 401) {
        setError("Token incorrecto.");
      } else {
        setError(String(err));
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="zed-stack zed-animate-fade zed-auth-layout">
      <ProductLockup href="" subtitle="Archivo Vivo · acceso self-hosted" />
      <Card variant="document" className={formStyles.form}>
        <h1 className="zed-title-section">Acceso</h1>
        <p className="zed-muted zed-flush">
          Instancia self-hosted de un solo operador. Introduce el token
          configurado en <code className="zed-mono">ZEDAZO_AUTH_TOKEN</code>.
        </p>
        {!httpsOk ? (
          <Callout variant="warning">
            La conexión no es HTTPS. En acceso remoto usa Caddy o un túnel con
            TLS antes de enviar el token.
          </Callout>
        ) : null}
        <form onSubmit={onSubmit} className={formStyles.form}>
          <div className={formStyles.field}>
            <label className="zed-label" htmlFor="acceso-token">
              Token de acceso
            </label>
            <input
              id="acceso-token"
              className="zed-input"
              type="password"
              autoComplete="current-password"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              required
            />
          </div>
          {error ? <ErrorState title="No se pudo entrar" message={error} /> : null}
          <Button type="submit" loading={busy} disabled={!token.trim()}>
            Entrar
          </Button>
        </form>
      </Card>
    </div>
  );
}
