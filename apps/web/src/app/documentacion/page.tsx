import Link from "next/link";
import { PageHeader } from "@/components/shell/page-header";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { buttonClassName } from "@/components/ui/button";

export default function DocumentacionPage() {
  return (
    <div className="zed-stack">
      <PageHeader
        title="Documentación"
        description="Referencias del despliegue self-hosted y la paridad CLI ↔ GUI."
      />

      <Callout variant="info" icon="book">
        La CLI sigue siendo la interfaz oficial de automatización. Completions de
        shell no tienen pantalla GUI equivalente.
      </Callout>

      <Card variant="document">
        <ul className="zed-stack zed-prose-list">
          <li>
            Matriz de paridad:{" "}
            <code className="zed-mono">docs/gui/functional-parity-matrix.md</code>
          </li>
          <li>
            OpenAPI: <code className="zed-mono">docs/api/openapi.yaml</code>
          </li>
          <li>
            Completions CLI:{" "}
            <code className="zed-mono">zedazo completions bash|zsh|fish</code>
          </li>
          <li>ADR-0015: core compartido + API + web self-hosted.</li>
          <li>
            Catálogo del sistema de diseño (átomos y patrones de producto, epic{" "}
            <a href="https://github.com/Iniciativas-Alexendros/zedazo/issues/59">
              #59
            </a>
            ):{" "}
            <Link href="/documentacion/ds">/documentacion/ds</Link>
          </li>
        </ul>
        <p className="zed-row zed-catalog-actions">
          <Link
            className={buttonClassName({ variant: "secondary" })}
            href="/procesar"
          >
            Ir a Procesar
          </Link>
          <Link
            className={buttonClassName({ variant: "tertiary" })}
            href="/documentacion/ds"
          >
            Abrir catálogo
          </Link>
        </p>
      </Card>
    </div>
  );
}
