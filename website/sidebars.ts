import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  docsSidebar: [
    "intro",
    "getting-started",
    {
      type: "category",
      label: "Guides",
      items: ["guides/recording", "guides/post-processing"],
    },
  ],
};

export default sidebars;
