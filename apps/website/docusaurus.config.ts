import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Demo As Code",
  tagline: "Browser recording with zoom and mouse telemetry",
  favicon: "img/favicon.svg",

  url: "https://newbiecoderxd.github.io",
  baseUrl: "/DemoAsCode/",

  organizationName: "frook",
  projectName: "DemoAsCode",

  onBrokenLinks: "warn",

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: "warn",
    },
  },

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          lastVersion: "1.0",
          includeCurrentVersion: false,
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themes: [
    [
      require.resolve("docusaurus-lunr-search"),
      {
        indexBlog: false,
      },
    ],
  ],

  plugins: [
    [
      "docusaurus-plugin-typedoc",
      {
        entryPoints: ["../engine/src/index.ts"],
        tsconfig: "./tsconfig.typedoc.json",
        out: "docs/api",
        readme: "none",
        exclude: ["**/example/**"],
      },
    ],
  ],

  themeConfig: {
    navbar: {
      title: "DemoAsCode",
      items: [
        {
          type: "docSidebar",
          sidebarId: "docsSidebar",
          position: "left",
          label: "Docs",
        },
        {
          type: "docsVersionDropdown",
          position: "right",
        },
        {
          href: "https://github.com/NewBieCoderXD/DemoAsCode",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Getting Started",
              to: "/docs/getting-started",
            },
          ],
        },
        {
          title: "Community",
          items: [
            {
              label: "GitHub",
              href: "https://github.com/frook/DemoAsCode",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} DemoAsCode contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["rust", "bash"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
