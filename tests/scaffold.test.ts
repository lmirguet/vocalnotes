import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import config from "../vite.config";
import pkg from "../package.json";
import tauri from "../src-tauri/tauri.conf.json";
import capability from "../src-tauri/capabilities/main.json";

describe("frontières du scaffold", () => {
  it("expose des commandes build/test/typecheck réelles, sans test vide", () => {
    expect(pkg.private).toBe(true);
    expect(pkg.scripts.build).toContain("typecheck");
    expect(pkg.scripts.test).toBe("vitest run");
    expect(pkg.scripts["test:unit"]).toBe(pkg.scripts.test);
    expect(pkg.scripts.typecheck).toBe("tsc --noEmit");
  });

  it("ne charge pas l'environnement ou les ressources publiques de la racine", () => {
    expect(config.envDir).toBe(
      resolve(fileURLToPath(new URL("../tooling/empty-env", import.meta.url))),
    );
    expect(config.envPrefix).toEqual([]);
    expect(config.publicDir).toBe(false);
    expect(config.server?.host).toBe("127.0.0.1");
    expect(config.server?.fs?.strict).toBe(true);
    expect(config.server?.fs?.deny).toEqual(expect.arrayContaining(["**/.env*", "**/*key*"]));
    expect(config.server?.fs?.allow).toHaveLength(3);
    expect(config.build?.sourcemap).toBe(false);
  });

  it("embarque seulement dist, sans filesystem/shell/tray dans la webview", () => {
    expect(tauri.build.frontendDist).toBe("../dist");
    expect(tauri.bundle.targets).toEqual(["rpm", "nsis"]);
    expect(tauri.app.windows.map((w) => w.label)).toEqual(["main"]);
    expect(capability.windows).toEqual(["main"]);
    expect(capability.permissions).toEqual([
      "allow-startup-status", "allow-prototype-snapshot", "allow-prototype-manual",
      "allow-prototype-action", "allow-prototype-set-key", "allow-prototype-delete-key",
      "allow-prototype-key-presence", "allow-prototype-finish-close",
      "allow-preferences-get", "allow-preferences-set",
    ]);
    expect(tauri.app.security.csp).toContain("default-src 'self'");
    expect(tauri.app.security.csp).toContain("object-src 'none'");
    expect(tauri.app.security.csp).not.toContain("https:");
    // Tauri's code generator requires actual PNG/ICO assets, not an empty list.
    expect(tauri.bundle.icon).toEqual([
      "icons/32x32.png", "icons/128x128.png", "icons/icon.ico",
    ]);
    for (const icon of tauri.bundle.icon) {
      const bytes = readFileSync(new URL(`../src-tauri/${icon}`, import.meta.url));
      expect([...bytes.subarray(0, 4)]).toEqual(
        icon.endsWith(".png") ? [137, 80, 78, 71] : [0, 0, 1, 0],
      );
    }
  });
});
