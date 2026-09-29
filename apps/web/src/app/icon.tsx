import { ImageResponse } from "next/og";
import { faviconHex } from "@/lib/design-tokens";

export const size = { width: 32, height: 32 };
export const contentType = "image/png";

export default function Icon() {
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: faviconHex.background,
          color: faviconHex.foreground,
          fontSize: 20,
          fontWeight: 700,
          borderRadius: 6,
        }}
      >
        Z
      </div>
    ),
    { ...size },
  );
}
