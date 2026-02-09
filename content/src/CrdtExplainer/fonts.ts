import { loadFont } from "@remotion/fonts";
import { staticFile } from "remotion";

/**
 * Load Google Sans Code from local .ttf file.
 * Must be called at module level (top of file) before rendering.
 */
const fontPromise = loadFont({
  family: "Google Sans Code",
  url: staticFile("GoogleSansCode-Regular.ttf"),
  weight: "400",
  style: "normal",
  format: "truetype",
});

/** The CSS font-family string to use in styles */
export const FONT_PRIMARY = '"Google Sans Code", monospace';
export const FONT_DISPLAY = '"Google Sans Code", system-ui, sans-serif';

/** Await this if you need to block rendering until the font is ready */
export { fontPromise };
