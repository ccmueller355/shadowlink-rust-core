import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

// ── Base path ─────────────────────────────────────────────────
// GitHub Pages project site: https://<org>.github.io/<repo>/
// GitHub Pages user/org site: https://<org>.github.io/        → base = /
//
// For shadowlink-rust-core: https://ccmueller355.github.io/shadowlink-rust-core/
const base = '/shadowlink-rust-core/'

export default withMermaid(
  defineConfig({
    base,
    title: 'ShadowLink Rust Core',
    description: 'Privacy-first Matrix protocol bridge — Rust core for the ShadowLink family comms app',
    lang: 'en-US',
    lastUpdated: true,
    cleanUrls: true,
    ignoreDeadLinks: true,

    head: [
      ['link', { rel: 'icon', href: '/favicon.ico' }],
    ],

    themeConfig: {
      search: { provider: 'local' },
      lastUpdated: { text: 'Updated' },

      nav: [
        { text: 'Home', link: '/' },
        { text: 'Manifest', link: '/PROJECT_MANIFEST' },
        { text: 'Coverage', link: '/coverage' },
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/arc42/' },
            { text: '1. Introduction & Goals', link: '/arc42/01-introduction-and-goals' },
            { text: '2. Constraints', link: '/arc42/02-architecture-constraints' },
            { text: '3. Context & Scope', link: '/arc42/03-system-scope-and-context' },
            { text: '4. Solution Strategy', link: '/arc42/04-solution-strategy' },
            { text: '5. Building Block View', link: '/arc42/05-building-block-view' },
            { text: '6. Runtime View', link: '/arc42/06-runtime-view' },
            { text: '7. Deployment View', link: '/arc42/07-deployment-view' },
            { text: '8. Cross-cutting Concepts', link: '/arc42/08-concepts' },
            { text: '9. Architecture Decisions', link: '/arc42/09-architecture-decisions' },
            { text: '10. Quality', link: '/arc42/10-quality-requirements' },
            { text: '11. Risks & Debt', link: '/arc42/11-risks-and-technical-debt' },
            { text: '12. Glossary', link: '/arc42/12-glossary' },
          ],
        },
      ],

      sidebar: {
        '/arc42/': [
          {
            text: 'arc42 Architecture',
            items: [
              { text: 'Overview', link: '/arc42/' },
              { text: '1. Introduction & Goals', link: '/arc42/01-introduction-and-goals' },
              { text: '2. Architecture Constraints', link: '/arc42/02-architecture-constraints' },
              { text: '3. System Scope & Context', link: '/arc42/03-system-scope-and-context' },
              { text: '4. Solution Strategy', link: '/arc42/04-solution-strategy' },
              { text: '5. Building Block View', link: '/arc42/05-building-block-view' },
              { text: '6. Runtime View', link: '/arc42/06-runtime-view' },
              { text: '7. Deployment View', link: '/arc42/07-deployment-view' },
              { text: '8. Cross-cutting Concepts', link: '/arc42/08-concepts' },
              { text: '9. Architecture Decisions', link: '/arc42/09-architecture-decisions' },
              { text: '10. Quality Requirements', link: '/arc42/10-quality-requirements' },
              { text: '11. Risks & Technical Debt', link: '/arc42/11-risks-and-technical-debt' },
              { text: '12. Glossary', link: '/arc42/12-glossary' },
            ],
          },
        ],
        '/': [],
      },

      socialLinks: [
        { icon: 'github', link: 'https://github.com/ccmueller355/shadowlink-rust-core' },
      ],
    },

    mermaidPlugin: {
      theme: 'neutral',
      themeVariables: {
        primaryColor: '#1a1a2e',
        primaryBorderColor: '#0f3460',
        lineColor: '#e94560',
      },
    },
  }),
)
