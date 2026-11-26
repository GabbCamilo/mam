export interface ConfirmModalData {
  titulo: string
  mensagem: string
  textoConfirmar: string
  perigo?: boolean
  onConfirm: () => Promise<void> | void
}

interface ConfirmModalProps {
  modal: ConfirmModalData | null
  onClose: () => void
}

export function ConfirmModal({ modal, onClose }: ConfirmModalProps) {
  if (!modal) return null

  return (
    <div className="mam-modal-backdrop" onClick={onClose}>
      <div
        className="mam-modal"
        style={{ maxWidth: 460 }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mam-modal-header">
          <strong>{modal.titulo}</strong>
          <button type="button" className="mam-btn" onClick={onClose}>
            ✕
          </button>
        </div>
        <p className="mam-section-desc" style={{ margin: 0 }}>
          {modal.mensagem}
        </p>
        <div className="mam-toolbar-row" style={{ justifyContent: 'flex-end' }}>
          <button type="button" className="mam-btn" onClick={onClose}>
            Voltar
          </button>
          <button
            type="button"
            className={`mam-btn ${modal.perigo ? 'mam-btn-danger' : 'mam-btn-primary'}`}
            onClick={async () => {
              const fn = modal.onConfirm
              onClose()
              await fn()
            }}
          >
            {modal.textoConfirmar}
          </button>
        </div>
      </div>
    </div>
  )
}
