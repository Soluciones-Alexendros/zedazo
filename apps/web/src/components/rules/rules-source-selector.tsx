"use client";

import formStyles from "@/styles/forms.module.css";
import { buttonClassName } from "@/components/ui/button";

type Mode = "builtin" | "toml";

type Props = {
  mode: Mode;
  onChange: (mode: Mode) => void;
};

export function RulesSourceSelector({ mode, onChange }: Props) {
  return (
    <fieldset className={formStyles.fieldset}>
      <legend className="zed-label">Origen de reglas</legend>
      <div className="zed-row">
        <label className={`${buttonClassName({ variant: "secondary" })} ${formStyles.choice}`}>
          <input
            type="radio"
            name="rules-mode"
            checked={mode === "builtin"}
            onChange={() => onChange("builtin")}
          />
          Reglas integradas
        </label>
        <label className={`${buttonClassName({ variant: "secondary" })} ${formStyles.choice}`}>
          <input
            type="radio"
            name="rules-mode"
            checked={mode === "toml"}
            onChange={() => onChange("toml")}
          />
          TOML personalizado
        </label>
      </div>
    </fieldset>
  );
}
