import { defineConfig } from 'vitepress'
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  srcDir: "src",

  title: "AHQ AI",
  description: "AI, Reimagined",
  base: "/ahqai/",
  head: [
    ['link', { rel: 'icon', href: '/ahqai/icon.png', type: 'image/png' }],
  ],
  markdown: {
    config(md) {
      md.use(tabsMarkdownPlugin)
    },
  },
  themeConfig: {
    logo: "/icon.png",
    lastUpdated: {
      text: "Last Updated "
    },
    outline: [2, 3],
    search: {
      provider: "local"
    },
    editLink: {
      pattern: 'https://github.com/ahqsoftorg/ahqai/edit/main/docs/src/:path'
    },
    footer: {
      copyright: "©AHQ Softwares",
      message: "Licensed under GNU Public License 3.0"
    },
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Download', link: '/download.md' },
      { text: 'Docs', activeMatch: '/docs/*', link: '/docs' },
    ],

    sidebar: [
      {
        text: 'Installation',
        collapsed: true,
        items: [
          { text: 'AHQ AI Client for MacOS', link: '/install/mac' },
          { text: 'AHQ AI Client for IOS', link: '/install/ios' }
        ]
      },
      {
        text: 'Docs',
        collapsed: false,
        items: [
          { text: "Introduction", link: "/docs/index.md" },
          { text: "Server Setup", link: "/docs/serversetup.md" },
          { text: "Client Setup", link: "/docs/clientsetup.md" },
        ]
      },
      {
        text: 'Contributing',
        collapsed: false,
        items: [
          { text: "Introduction", link: "/contributing/index.md" },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/ahqsoftorg/ahqai' }
    ]
  }
})
