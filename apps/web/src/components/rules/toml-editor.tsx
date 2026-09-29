"use client";

type Props = {
  value: string;
  onChange: (value: string) => void;
  id?: string;
  describedBy?: string;
  invalid?: boolean;
};

export function TomlEditor({
  value,
  onChange,
  id = "toml-editor",
  describedBy,
  invalid,
}: Props) {
  return (
    <textarea
      id={id}
      className="zed-textarea"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      spellCheck={false}
      aria-invalid={invalid || undefined}
      aria-describedby={describedBy}
      placeholder="# Reglas TOML de zedazo"
    />
  );
}
