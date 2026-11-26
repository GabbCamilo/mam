import { useI18n } from '../i18n'
import {
  APARENCIA_PADRAO,
  ConfigAparencia,
  FonteId,
  OPCOES_FONTES,
  TEMAS_PRESET,
  TemaBase,
} from '../themes'

interface ThemeModalProps {
  aberto: boolean
  config: ConfigAparencia
  onChange: (nova: ConfigAparencia) => void
  onFechar: () => void
}

export function ThemeModal({ aberto, config, onChange, onFechar }: ThemeModalProps) {
  const { idioma, tx } = useI18n()
  if (!aberto) return null

  const selecionandoCustom = config.presetId === 'custom'

  function selecionarPreset(id: string) {
    onChange({
      ...config,
      presetId: id,
    })
  }

  function atualizarCustom(patch: Partial<ConfigAparencia['custom']>) {
    onChange({
      ...config,
      presetId: 'custom',
      custom: {
        ...config.custom,
        ...patch,
      },
    })
  }

  return (
    <div className="mam-modal-backdrop" onClick={onFechar}>
      <div
        className="mam-modal"
        style={{ maxWidth: 640 }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mam-section-header">
          <div>
            <h3 className="mam-modal-title">🎨 {tx('Aparência e Temas', 'Appearance & Themes')}</h3>
            <p className="mam-section-desc">
              {tx(
                'Personalize gradientes de fundo, contornos, cores de destaque e fonte da interface.',
                'Customize background gradients, borders, accent colors, and UI font.',
              )}
            </p>
          </div>
          <button className="mam-btn" type="button" onClick={onFechar}>
            ✕
          </button>
        </div>

        <div style={{ display: 'grid', gap: '1rem' }}>
          <div>
            <div style={{ fontWeight: 600, fontSize: '0.88rem', marginBottom: '0.5rem' }}>
              {tx('Temas Coloridos & Degradês', 'Color Themes & Gradients')}
            </div>
            <div className="mam-theme-swatch-grid">
              <button
                type="button"
                className={`mam-theme-swatch mam-theme-swatch-custom ${selecionandoCustom ? 'active' : ''}`}
                onClick={() => selecionarPreset('custom')}
                title={tx('Criar degradê e contornos personalizados', 'Create custom gradient and borders')}
                style={{
                  background: `linear-gradient(${config.custom.angulo}deg, ${config.custom.cor1}, ${config.custom.cor2})`,
                }}
              >
                <span className="mam-theme-swatch-badge">{tx('Personalizado', 'Custom')}</span>
                <span style={{ fontSize: '1.15rem' }}>🎨</span>
              </button>

              {TEMAS_PRESET.map((t) => {
                const ativo = config.presetId === t.id
                const nomeTema = idioma === 'en' ? t.nomeEn : t.nome
                return (
                  <button
                    key={t.id}
                    type="button"
                    className={`mam-theme-swatch ${ativo ? 'active' : ''}`}
                    style={{ background: t.preview }}
                    onClick={() => selecionarPreset(t.id)}
                    title={`${nomeTema} (${t.base === 'dark' ? 'Dark' : 'Light'})`}
                  >
                    <span className="mam-theme-swatch-label">{nomeTema}</span>
                  </button>
                )
              })}
            </div>
          </div>

          {selecionandoCustom && (
            <div className="mam-panel" style={{ padding: '0.85rem', display: 'grid', gap: '0.75rem' }}>
              <div className="mam-section-header">
                <strong style={{ fontSize: '0.88rem' }}>{tx('Editor de Degradê & Contorno', 'Gradient & Border Editor')}</strong>
                <div className="mam-segmented">
                  <button
                    type="button"
                    className={`mam-segmented-btn ${config.custom.base === 'dark' ? 'active' : ''}`}
                    onClick={() => atualizarCustom({ base: 'dark' as TemaBase })}
                  >
                    🌙 {tx('Base Dark', 'Dark Base')}
                  </button>
                  <button
                    type="button"
                    className={`mam-segmented-btn ${config.custom.base === 'light' ? 'active' : ''}`}
                    onClick={() => atualizarCustom({ base: 'light' as TemaBase })}
                  >
                    ☀️ {tx('Base Light', 'Light Base')}
                  </button>
                </div>
              </div>

              <div className="mam-toolbar-row" style={{ gap: '0.9rem' }}>
                <label className="mam-color-field">
                  <span>{tx('Fundo 1', 'Bg 1')}</span>
                  <input
                    type="color"
                    value={config.custom.cor1}
                    onChange={(e) => atualizarCustom({ cor1: e.target.value })}
                  />
                </label>
                <label className="mam-color-field">
                  <span>{tx('Fundo 2', 'Bg 2')}</span>
                  <input
                    type="color"
                    value={config.custom.cor2}
                    onChange={(e) => atualizarCustom({ cor2: e.target.value })}
                  />
                </label>
                <label className="mam-color-field">
                  <span>{tx('Contorno', 'Border')}</span>
                  <input
                    type="color"
                    value={config.custom.corContorno}
                    onChange={(e) => atualizarCustom({ corContorno: e.target.value })}
                  />
                </label>
                <label className="mam-color-field">
                  <span>{tx('Destaque', 'Accent')}</span>
                  <input
                    type="color"
                    value={config.custom.corAcento}
                    onChange={(e) => atualizarCustom({ corAcento: e.target.value })}
                  />
                </label>
                <label className="mam-color-field" style={{ flex: 1, minWidth: 110 }}>
                  <span>{tx('Ângulo', 'Angle')} ({config.custom.angulo}°)</span>
                  <input
                    type="range"
                    min={0}
                    max={360}
                    step={15}
                    value={config.custom.angulo}
                    onChange={(e) => atualizarCustom({ angulo: Number(e.target.value) })}
                  />
                </label>
              </div>
            </div>
          )}

          <div>
            <div style={{ fontWeight: 600, fontSize: '0.88rem', marginBottom: '0.45rem' }}>
              {tx('Fonte da Interface', 'UI Font')}
            </div>
            <select
              className="mam-select"
              style={{ width: '100%' }}
              value={config.fonteId}
              onChange={(e) =>
                onChange({
                  ...config,
                  fonteId: e.target.value as FonteId,
                })
              }
            >
              {OPCOES_FONTES.map((f) => (
                <option key={f.id} value={f.id}>
                  {idioma === 'en' ? f.nomeEn : f.nome}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="mam-toolbar-row" style={{ justifyContent: 'space-between', marginTop: '0.4rem' }}>
          <button
            type="button"
            className="mam-btn"
            onClick={() => onChange(APARENCIA_PADRAO)}
          >
            {tx('Restaurar Padrão', 'Reset Default')}
          </button>
          <button type="button" className="mam-btn mam-btn-primary" onClick={onFechar}>
            {tx('Concluir', 'Done')}
          </button>
        </div>
      </div>
    </div>
  )
}
