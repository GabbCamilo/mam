export type TemaBase = 'dark' | 'light'

export type FonteId = 'sistema' | 'moderna' | 'editorial' | 'mono' | 'suave'

export interface OpcaoFonte {
  id: FonteId
  nome: string
  nomeEn: string
  family: string
}

export const OPCOES_FONTES: OpcaoFonte[] = [
  {
    id: 'sistema',
    nome: 'Padrão do Sistema',
    nomeEn: 'System Default',
    family: "'Segoe UI', system-ui, -apple-system, sans-serif",
  },
  {
    id: 'moderna',
    nome: 'Moderna (Clean Sans)',
    nomeEn: 'Modern (Clean Sans)',
    family: "'Inter', 'Segoe UI Variable', 'Segoe UI', 'Roboto', sans-serif",
  },
  {
    id: 'suave',
    nome: 'Suave (Leitura)',
    nomeEn: 'Soft (Reading)',
    family: "'Trebuchet MS', 'Segoe UI', sans-serif",
  },
  {
    id: 'editorial',
    nome: 'Editorial (Clássica)',
    nomeEn: 'Editorial (Classic)',
    family: "'Georgia', 'Cambria', 'Times New Roman', serif",
  },
  {
    id: 'mono',
    nome: 'Técnica (Monoespaçada)',
    nomeEn: 'Technical (Monospace)',
    family: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
  },
]

export interface TemaPreset {
  id: string
  nome: string
  nomeEn: string
  base: TemaBase
  preview: string
  bgApp: string
  bgSurface: string
  bgCard: string
  bgCardHover: string
  bgInput: string
  borderSubtle: string
  borderAccent: string
  accent: string
  accentStrong: string
}

export const TEMAS_PRESET: TemaPreset[] = [
  {
    id: 'dark-padrao',
    nome: 'Dark Padrão',
    nomeEn: 'Default Dark',
    base: 'dark',
    preview: 'linear-gradient(135deg, #0a0e17, #19253c)',
    bgApp: '#0a0e17',
    bgSurface: '#111827',
    bgCard: '#131c2e',
    bgCardHover: '#19253c',
    bgInput: '#0e1523',
    borderSubtle: '#1e293b',
    borderAccent: '#1d4ed8',
    accent: '#0ea5e9',
    accentStrong: '#2563eb',
  },
  {
    id: 'light-padrao',
    nome: 'Light Padrão',
    nomeEn: 'Default Light',
    base: 'light',
    preview: 'linear-gradient(135deg, #f1f5f9, #dbeafe)',
    bgApp: '#f1f5f9',
    bgSurface: '#ffffff',
    bgCard: '#f8fafc',
    bgCardHover: '#e2e8f0',
    bgInput: '#ffffff',
    borderSubtle: '#cbd5e1',
    borderAccent: '#2563eb',
    accent: '#0284c7',
    accentStrong: '#2563eb',
  },
  {
    id: 'menta-suave',
    nome: 'Menta Suave',
    nomeEn: 'Soft Mint',
    base: 'light',
    preview: 'linear-gradient(135deg, #bbf7d0, #a7f3d0)',
    bgApp: 'linear-gradient(135deg, #dcfce7 0%, #cffafe 100%)',
    bgSurface: 'rgba(255, 255, 255, 0.78)',
    bgCard: 'rgba(255, 255, 255, 0.72)',
    bgCardHover: 'rgba(236, 253, 245, 0.92)',
    bgInput: 'rgba(255, 255, 255, 0.9)',
    borderSubtle: 'rgba(16, 185, 129, 0.35)',
    borderAccent: '#059669',
    accent: '#059669',
    accentStrong: '#0d9488',
  },
  {
    id: 'pessego-dourado',
    nome: 'Pêssego Dourado',
    nomeEn: 'Golden Peach',
    base: 'light',
    preview: 'linear-gradient(135deg, #fde68a, #fecaca)',
    bgApp: 'linear-gradient(135deg, #fef3c7 0%, #ffe4e6 100%)',
    bgSurface: 'rgba(255, 251, 245, 0.8)',
    bgCard: 'rgba(255, 255, 255, 0.74)',
    bgCardHover: 'rgba(255, 247, 237, 0.95)',
    bgInput: 'rgba(255, 255, 255, 0.92)',
    borderSubtle: 'rgba(245, 158, 11, 0.35)',
    borderAccent: '#d97706',
    accent: '#ea580c',
    accentStrong: '#d97706',
  },
  {
    id: 'lavanda-celeste',
    nome: 'Lavanda Celeste',
    nomeEn: 'Sky Lavender',
    base: 'light',
    preview: 'linear-gradient(135deg, #c7d2fe, #bae6fd)',
    bgApp: 'linear-gradient(135deg, #e0e7ff 0%, #e0f2fe 100%)',
    bgSurface: 'rgba(255, 255, 255, 0.78)',
    bgCard: 'rgba(255, 255, 255, 0.72)',
    bgCardHover: 'rgba(238, 242, 255, 0.95)',
    bgInput: 'rgba(255, 255, 255, 0.92)',
    borderSubtle: 'rgba(99, 102, 241, 0.32)',
    borderAccent: '#4f46e5',
    accent: '#4f46e5',
    accentStrong: '#0284c7',
  },
  {
    id: 'rosa-nevoa',
    nome: 'Rosa Névoa',
    nomeEn: 'Misty Rose',
    base: 'light',
    preview: 'linear-gradient(135deg, #fbcfe8, #e9d5ff)',
    bgApp: 'linear-gradient(135deg, #fce7f3 0%, #f3e8ff 100%)',
    bgSurface: 'rgba(255, 255, 255, 0.78)',
    bgCard: 'rgba(255, 255, 255, 0.74)',
    bgCardHover: 'rgba(253, 242, 248, 0.95)',
    bgInput: 'rgba(255, 255, 255, 0.92)',
    borderSubtle: 'rgba(219, 39, 119, 0.28)',
    borderAccent: '#db2777',
    accent: '#db2777',
    accentStrong: '#9333ea',
  },
  {
    id: 'cyber-neon',
    nome: 'Cyber Neon',
    nomeEn: 'Cyber Neon',
    base: 'dark',
    preview: 'linear-gradient(135deg, #7e22ce, #2563eb, #06b6d4)',
    bgApp: 'linear-gradient(135deg, #1e1b4b 0%, #4c1d95 50%, #0c4a6e 100%)',
    bgSurface: 'rgba(15, 16, 38, 0.76)',
    bgCard: 'rgba(24, 24, 58, 0.72)',
    bgCardHover: 'rgba(49, 46, 129, 0.82)',
    bgInput: 'rgba(15, 16, 38, 0.88)',
    borderSubtle: 'rgba(168, 85, 247, 0.42)',
    borderAccent: '#c084fc',
    accent: '#c084fc',
    accentStrong: '#38bdf8',
  },
  {
    id: 'carmim-sombrio',
    nome: 'Carmim Sombrio',
    nomeEn: 'Dark Crimson',
    base: 'dark',
    preview: 'linear-gradient(135deg, #7f1d1d, #09090b)',
    bgApp: 'linear-gradient(135deg, #450a0a 0%, #09090b 75%)',
    bgSurface: 'rgba(20, 10, 12, 0.8)',
    bgCard: 'rgba(34, 14, 18, 0.76)',
    bgCardHover: 'rgba(69, 18, 24, 0.88)',
    bgInput: 'rgba(18, 8, 10, 0.9)',
    borderSubtle: 'rgba(239, 68, 68, 0.35)',
    borderAccent: '#ef4444',
    accent: '#f87171',
    accentStrong: '#dc2626',
  },
  {
    id: 'meia-noite-indigo',
    nome: 'Meia-Noite Índigo',
    nomeEn: 'Midnight Indigo',
    base: 'dark',
    preview: 'linear-gradient(135deg, #1e1b4b, #020617)',
    bgApp: 'linear-gradient(135deg, #1e1b4b 0%, #0f172a 55%, #020617 100%)',
    bgSurface: 'rgba(12, 14, 34, 0.8)',
    bgCard: 'rgba(20, 24, 54, 0.76)',
    bgCardHover: 'rgba(34, 40, 88, 0.86)',
    bgInput: 'rgba(10, 12, 28, 0.9)',
    borderSubtle: 'rgba(129, 140, 248, 0.34)',
    borderAccent: '#818cf8',
    accent: '#818cf8',
    accentStrong: '#4f46e5',
  },
  {
    id: 'floresta-esmeralda',
    nome: 'Floresta Esmeralda',
    nomeEn: 'Emerald Forest',
    base: 'dark',
    preview: 'linear-gradient(135deg, #064e3b, #111827)',
    bgApp: 'linear-gradient(135deg, #022c22 0%, #064e3b 45%, #0f172a 100%)',
    bgSurface: 'rgba(6, 28, 24, 0.8)',
    bgCard: 'rgba(10, 40, 34, 0.75)',
    bgCardHover: 'rgba(16, 60, 50, 0.88)',
    bgInput: 'rgba(5, 22, 18, 0.9)',
    borderSubtle: 'rgba(52, 211, 153, 0.32)',
    borderAccent: '#10b981',
    accent: '#34d399',
    accentStrong: '#059669',
  },
  {
    id: 'crepusculo-ambar',
    nome: 'Crepúsculo Âmbar',
    nomeEn: 'Amber Twilight',
    base: 'dark',
    preview: 'linear-gradient(135deg, #701a75, #b45309)',
    bgApp: 'linear-gradient(135deg, #4a044e 0%, #78350f 100%)',
    bgSurface: 'rgba(28, 12, 26, 0.78)',
    bgCard: 'rgba(42, 18, 36, 0.74)',
    bgCardHover: 'rgba(66, 28, 48, 0.86)',
    bgInput: 'rgba(22, 10, 20, 0.9)',
    borderSubtle: 'rgba(251, 146, 60, 0.38)',
    borderAccent: '#f59e0b',
    accent: '#fb923c',
    accentStrong: '#d97706',
  },
  {
    id: 'aurora-abissal',
    nome: 'Aurora Abissal',
    nomeEn: 'Abyssal Aurora',
    base: 'dark',
    preview: 'linear-gradient(135deg, #0f766e, #1d4ed8, #090d16)',
    bgApp: 'linear-gradient(135deg, #134e4a 0%, #1e3a8a 50%, #090d16 100%)',
    bgSurface: 'rgba(10, 20, 34, 0.78)',
    bgCard: 'rgba(16, 30, 50, 0.74)',
    bgCardHover: 'rgba(24, 48, 78, 0.86)',
    bgInput: 'rgba(8, 16, 28, 0.9)',
    borderSubtle: 'rgba(45, 212, 191, 0.34)',
    borderAccent: '#2dd4bf',
    accent: '#2dd4bf',
    accentStrong: '#2563eb',
  },
]

export interface ConfigAparencia {
  presetId: string
  fonteId: FonteId
  custom: {
    base: TemaBase
    cor1: string
    cor2: string
    angulo: number
    corContorno: string
    corAcento: string
  }
}

export const APARENCIA_PADRAO: ConfigAparencia = {
  presetId: 'dark-padrao',
  fonteId: 'sistema',
  custom: {
    base: 'dark',
    cor1: '#1e1b4b',
    cor2: '#0c4a6e',
    angulo: 135,
    corContorno: '#38bdf8',
    corAcento: '#0ea5e9',
  },
}

const STORAGE_KEY = 'mam:aparencia_v1'

export function carregarAparencia(): ConfigAparencia {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return APARENCIA_PADRAO
    const parsed = JSON.parse(raw) as Partial<ConfigAparencia>
    return {
      presetId: parsed.presetId || APARENCIA_PADRAO.presetId,
      fonteId: parsed.fonteId || APARENCIA_PADRAO.fonteId,
      custom: {
        ...APARENCIA_PADRAO.custom,
        ...(parsed.custom || {}),
      },
    }
  } catch {
    return APARENCIA_PADRAO
  }
}

export function salvarAparencia(cfg: ConfigAparencia): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg))
  } catch {
    // Ignora indisponibilidade de storage
  }
}

export function aplicarAparenciaNoDocumento(cfg: ConfigAparencia): TemaBase {
  const root = document.documentElement
  const fonte = OPCOES_FONTES.find((f) => f.id === cfg.fonteId) ?? OPCOES_FONTES[0]
  root.style.setProperty('--font-app', fonte.family)

  if (cfg.presetId === 'custom') {
    const { base, cor1, cor2, angulo, corContorno, corAcento } = cfg.custom
    root.setAttribute('data-theme', base)
    root.style.setProperty('--bg-app', `linear-gradient(${angulo}deg, ${cor1} 0%, ${cor2} 100%)`)
    if (base === 'dark') {
      root.style.setProperty('--bg-surface', 'rgba(13, 17, 28, 0.78)')
      root.style.setProperty('--bg-card', 'rgba(19, 26, 44, 0.74)')
      root.style.setProperty('--bg-card-hover', 'rgba(28, 38, 64, 0.88)')
      root.style.setProperty('--bg-input', 'rgba(10, 14, 24, 0.88)')
    } else {
      root.style.setProperty('--bg-surface', 'rgba(255, 255, 255, 0.8)')
      root.style.setProperty('--bg-card', 'rgba(255, 255, 255, 0.74)')
      root.style.setProperty('--bg-card-hover', 'rgba(241, 245, 249, 0.94)')
      root.style.setProperty('--bg-input', 'rgba(255, 255, 255, 0.92)')
    }
    root.style.setProperty('--border-subtle', `${corContorno}55`)
    root.style.setProperty('--border-accent', corContorno)
    root.style.setProperty('--accent', corAcento)
    root.style.setProperty('--accent-strong', corContorno)
    root.style.setProperty('--accent-soft', `${corAcento}24`)
    return base
  }

  const preset = TEMAS_PRESET.find((t) => t.id === cfg.presetId) ?? TEMAS_PRESET[0]
  root.setAttribute('data-theme', preset.base)
  root.style.setProperty('--bg-app', preset.bgApp)
  root.style.setProperty('--bg-surface', preset.bgSurface)
  root.style.setProperty('--bg-card', preset.bgCard)
  root.style.setProperty('--bg-card-hover', preset.bgCardHover)
  root.style.setProperty('--bg-input', preset.bgInput)
  root.style.setProperty('--border-subtle', preset.borderSubtle)
  root.style.setProperty('--border-accent', preset.borderAccent)
  root.style.setProperty('--accent', preset.accent)
  root.style.setProperty('--accent-strong', preset.accentStrong)
  root.style.setProperty('--accent-soft', `${preset.accent}24`)
  return preset.base
}
