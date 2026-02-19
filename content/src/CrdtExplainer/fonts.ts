import { loadFont } from "@remotion/google-fonts/GoogleSansCode";

const { fontFamily } = loadFont("normal", {
  weights: ["400"],
  subsets: ["latin"],
});

/** The CSS font-family string to use in styles */
export const FONT_PRIMARY = `${fontFamily}, monospace`;
export const FONT_DISPLAY = `${fontFamily}, system-ui, sans-serif`;
