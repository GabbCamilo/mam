import { createContext, createElement, ReactNode, useContext, useEffect, useState } from 'react'

export type IdiomaUi = 'pt-BR' | 'en'

const STORAGE_KEY = 'mam:idioma_v1'

export function carregarIdioma(): IdiomaUi {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw === 'en' || raw === 'pt-BR') {
      return raw
    }
  } catch {
    // Ignora indisponibilidade de storage
  }
  return 'pt-BR'
}

export function salvarIdioma(idioma: IdiomaUi): void {
  try {
    localStorage.setItem(STORAGE_KEY, idioma)
  } catch {
    // Ignora indisponibilidade de storage
  }
}

export function aplicarIdiomaNoDocumento(idioma: IdiomaUi): void {
  document.documentElement.lang = idioma
}

function interpolar(texto: string, vars?: Record<string, string | number>): string {
  if (!vars) return texto
  return Object.entries(vars).reduce(
    (acc, [k, v]) => acc.replaceAll(`{${k}}`, String(v)),
    texto,
  )
}

interface I18nContextValue {
  idioma: IdiomaUi
  setIdioma: (idioma: IdiomaUi) => void
  alternarIdioma: () => void
  tx: (pt: string, en: string, vars?: Record<string, string | number>) => string
}

const I18nContext = createContext<I18nContextValue | null>(null)

export function I18nProvider({ children }: { children: ReactNode }) {
  const [idioma, setIdiomaState] = useState<IdiomaUi>(() => carregarIdioma())

  useEffect(() => {
    salvarIdioma(idioma)
    aplicarIdiomaNoDocumento(idioma)
  }, [idioma])

  function setIdioma(novo: IdiomaUi) {
    setIdiomaState(novo)
  }

  function alternarIdioma() {
    setIdiomaState((prev) => (prev === 'pt-BR' ? 'en' : 'pt-BR'))
  }

  function tx(pt: string, en: string, vars?: Record<string, string | number>): string {
    return interpolar(idioma === 'en' ? en : pt, vars)
  }

  return createElement(
    I18nContext.Provider,
    { value: { idioma, setIdioma, alternarIdioma, tx } },
    children,
  )
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) {
    throw new Error('useI18n deve ser usado dentro de I18nProvider')
  }
  return ctx
}
