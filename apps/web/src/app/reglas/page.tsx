"use client";

import { useState } from "react";
import { validateRules } from "@/lib/data-adapter";
import { PageHeader } from "@/components/shell/page-header";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { Button } from "@/components/ui/button";
import { RulesSourceSelector } from "@/components/rules/rules-source-selector";
import { RulesValidation } from "@/components/rules/rules-validation";
import { RulesHash } from "@/components/rules/rules-hash";
import { TomlEditor } from "@/components/rules/toml-editor";
import { ErrorState } from "@/components/ui/error-state";
import formStyles from "@/styles/forms.module.css";

const EXAMPLE = `[zedazo]
replace = false
prefijo_pais = "+34"

[clasificacion]
replace = false
`;

export default function ReglasPage() {
  const [mode, setMode] = useState<"builtin" | "toml">("toml");
  const [toml, setToml] = useState(EXAMPLE);
  const [ok, setOk] = useState<boolean | null>(null);
  const [diagnostics, setDiagnostics] = useState<{ message: string }[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="zed-stack">
      <PageHeader
        title="Reglas"
        description="Configuración TOML de precisión. Los cambios no alteran ejecuciones anteriores."
      />

      <Callout variant="verification" icon="clock-rotate-left">
        Editar reglas aquí no modifica ejecuciones ya creadas. Solo se aplican al
        crear una nueva ejecución.
      </Callout>

      <div className={formStyles.split}>
        <Card variant="document" className={formStyles.form}>
          <RulesSourceSelector
            mode={mode}
            onChange={(m) => {
              setMode(m);
              setOk(null);
            }}
          />
          {mode === "builtin" ? (
            <Callout variant="info" title="Reglas integradas">
              Al crear una ejecución con modo integrado se usan los defaults del
              core. Duplica el ejemplo TOML para personalizar.
            </Callout>
          ) : (
            <div className={formStyles.field}>
              <label className="zed-label" htmlFor="reglas-toml">
                Editor TOML
              </label>
              <TomlEditor
                id="reglas-toml"
                value={toml}
                onChange={(v) => {
                  setToml(v);
                  setOk(null);
                }}
                describedBy="reglas-help"
              />
              <p id="reglas-help" className="zed-muted zed-flush">
                <code className="zed-mono">replace=false</code> hace append;
                <code className="zed-mono"> replace=true</code> sustituye.
              </p>
            </div>
          )}
          <div className={formStyles.actions}>
            <Button
              loading={busy}
              disabled={mode !== "toml"}
              onClick={async () => {
                setBusy(true);
                setError(null);
                try {
                  const r = await validateRules(toml);
                  setOk(r.ok);
                  setDiagnostics(r.diagnostics || []);
                } catch (e) {
                  setError(String(e));
                } finally {
                  setBusy(false);
                }
              }}
            >
              Validar
            </Button>
            {mode === "builtin" ? (
              <Button
                variant="secondary"
                onClick={() => {
                  setMode("toml");
                  setToml(EXAMPLE);
                }}
              >
                Duplicar como editable
              </Button>
            ) : null}
          </div>
        </Card>

        <Card variant="document" className="zed-stack">
          <h2 className="zed-title-section">Validación y versión</h2>
          {error ? <ErrorState message={error} /> : null}
          <RulesValidation ok={ok} diagnostics={diagnostics} />
          <RulesHash hash={null} />
        </Card>
      </div>
    </div>
  );
}
