import {
  forwardRef,
  type ButtonHTMLAttributes,
  type ReactNode,
} from "react";

type ButtonVariant =
  | "primary"
  | "secondary"
  | "tertiary"
  | "danger"
  | "icon";

export type ButtonSize = "sm" | "md";

type ButtonClassNameOptions = {
  variant?: ButtonVariant;
  size?: ButtonSize;
  className?: string;
};

/** Receta única `.zed-button` para `<Button>`, `<Link>` y `<label>`. */
export function buttonClassName({
  variant = "primary",
  size = "md",
  className = "",
}: ButtonClassNameOptions = {}) {
  return [
    "zed-button",
    `zed-button--${variant}`,
    size === "sm" ? "zed-button--sm" : "",
    className,
  ]
    .filter(Boolean)
    .join(" ");
}

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  children?: ReactNode;
};

export const Button = forwardRef<HTMLButtonElement, Props>(function Button(
  {
    variant = "primary",
    size = "md",
    loading = false,
    children,
    className = "",
    disabled,
    ...rest
  },
  ref,
) {
  return (
    <button
      ref={ref}
      className={buttonClassName({ variant, size, className })}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <span className="zed-spinner" aria-hidden /> : null}
      {children}
    </button>
  );
});
