/**
 * Note: When using the Node.JS APIs, the config file
 * doesn't apply. Instead, pass options directly to the APIs.
 *
 * All configuration options: https://remotion.dev/docs/config
 */

import { Config } from "@remotion/cli/config";

// ── 4K UHD output (3840×2160 from 1920×1080 base) ──
Config.setScale(2);

// ── Highest quality settings ──
Config.setVideoImageFormat("png");
Config.setCrf(1);
Config.setPixelFormat("yuv444p");
Config.setCodec("h264");

// ── WSL / headless rendering ──
Config.setChromiumOpenGlRenderer("angle");
Config.setConcurrency(2);
Config.setDelayRenderTimeoutInMilliseconds(120000);
