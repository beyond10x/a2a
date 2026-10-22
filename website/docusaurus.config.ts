import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {withProductSite} from '@beyond10x/docs-system/product-site';

const config: Config = {
  title: 'a2a',
  tagline: 'The A2A protocol v1.0.1 as an executable ESS specification.',

  future: {
    v4: true,
  },

  url: 'https://beyond10x.github.io',
  baseUrl: '/a2a/',

  organizationName: 'beyond10x',
  projectName: 'a2a',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',

  markdown: {
    format: 'detect',
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
    mermaid: true,
  },

  themes: ['@docusaurus/theme-mermaid'],

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/a2a/tree/main/website/',
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'a2a',
      items: [
        {to: '/docs/', label: 'Documentation', position: 'left'},
        {to: '/docs/reference/downloads', label: 'Downloads', position: 'left'},
        {to: '/docs/reference/specification', label: 'Specification', position: 'left'},
        {href: 'https://github.com/beyond10x/a2a', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Overview', to: '/docs/'},
            {label: 'What the specification covers', to: '/docs/concepts/coverage'},
            {label: 'UNMAPPED and ESS gaps', to: '/docs/concepts/gaps'},
            {label: 'Use the artifacts', to: '/docs/guides/use-the-artifacts'},
            {label: 'Downloads', to: '/docs/reference/downloads'},
            {label: 'HTTP+JSON routes', to: '/docs/reference/http-routes'},
            {label: 'Specification', to: '/docs/reference/specification'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/a2a'},
            {label: 'Changelog', href: 'https://github.com/beyond10x/a2a/blob/main/CHANGELOG.md'},
            {label: 'Releases', href: 'https://github.com/beyond10x/a2a/releases'},
            {label: 'A2A upstream', href: 'https://github.com/a2aproject/A2A'},
          ],
        },
      ],
      copyright: 'A beyond10x project. a2a · Apache-2.0.',
    },
  } satisfies Preset.ThemeConfig,
};

export default withProductSite(config, {landing: './product.json', product: 'a2a'});
