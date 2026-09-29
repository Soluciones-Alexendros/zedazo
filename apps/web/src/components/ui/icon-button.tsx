import {
  forwardRef,
  type ButtonHTMLAttributes,
  type ReactNode,
} from "react";
import { Button, type ButtonSize } from "./button";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  children: ReactNode;
  size?: ButtonSize;
};

export const IconButton = forwardRef<HTMLButtonElement, Props>(
  function IconButton({ label, children, size = "md", ...rest }, ref) {
    return (
      <Button
        ref={ref}
        variant="icon"
        size={size}
        aria-label={label}
        title={label}
        {...rest}
      >
        {children}
      </Button>
    );
  },
);
