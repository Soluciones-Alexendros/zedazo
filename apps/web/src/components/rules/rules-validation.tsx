import { Callout } from "@/components/ui/callout";

type Props = {
  ok?: boolean | null;
  diagnostics?: { message: string }[];
};

export function RulesValidation({ ok, diagnostics = [] }: Props) {
  if (ok == null) {
    return (
      <Callout variant="info" title="Validación" icon="flask">
        Ejecuta la validación para comprobar la sintaxis TOML contra la API.
      </Callout>
    );
  }
  if (ok) {
    return (
      <Callout variant="success" title="Reglas válidas" icon="circle-check">
        La configuración es sintácticamente válida.
      </Callout>
    );
  }
  return (
    <Callout variant="danger" title="Diagnósticos" icon="circle-exclamation">
      <ul className="zed-prose-list zed-flush">
        {diagnostics.map((d, i) => (
          <li key={i}>{d.message}</li>
        ))}
      </ul>
    </Callout>
  );
}
