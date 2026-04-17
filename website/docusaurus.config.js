import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: '20,000 BC',
  tagline: 'A prehistoric real-time strategy game',
  favicon: 'img/favicon.ico',

  future: { v4: true },

  url: 'https://site-two-green-31.vercel.app',
  baseUrl: '/',

  organizationName: 'manawenuz',
  projectName: '20KBC',

  onBrokenLinks: 'warn',

  markdown: {
    format: 'md',
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          routeBasePath: '/',
          editUrl: 'https://github.com/manawenuz/20KBC/tree/main/website/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        defaultMode: 'dark',
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: '20,000 BC',
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'designSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/manawenuz/20KBC',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Documentation',
            items: [
              { label: 'Game Overview', to: '/game-design/01-game-overview' },
              { label: 'Units', to: '/units-structures/08-units' },
              { label: 'Buildings', to: '/units-structures/09-buildings' },
            ],
          },
          {
            title: 'Research',
            items: [
              { label: 'Engine Comparison', to: '/research/engine-comparison' },
              { label: 'Networking', to: '/research/networking-architecture' },
              { label: 'Map Generation', to: '/research/map-generation' },
            ],
          },
          {
            title: 'Links',
            items: [
              { label: 'GitHub', href: 'https://github.com/manawenuz/20KBC' },
            ],
          },
        ],
        copyright: '20,000 BC Game Project',
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
      },
    }),
};

export default config;
