import { motion } from 'framer-motion'
import { CHAIN_META, ChainId } from '../types/wallet'
import { useWalletStore } from '../store/walletStore'
import CryptoIcon from './CryptoIcon'

interface TokenSelectorProps {
  onSelect: (chain: ChainId) => void
  onClose: () => void
}

export default function TokenSelector({ onSelect, onClose }: TokenSelectorProps) {
  const { balances } = useWalletStore()
  const chains = Object.keys(CHAIN_META) as ChainId[]

  return (
    <motion.div
      className="token-selector-overlay"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      onClick={onClose}
    >
      <motion.div
        className="token-selector-sheet"
        initial={{ y: '100%' }}
        animate={{ y: 0 }}
        exit={{ y: '100%' }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="sheet-handle" />
        <h3>Select Network</h3>

        <div className="chain-list">
          {chains.map((chain) => {
            const meta = CHAIN_META[chain]
            const balance = balances[chain]

            return (
              <button
                key={chain}
                className="chain-option"
                onClick={() => onSelect(chain)}
              >
                <span className="chain-icon-wrap">
                  <CryptoIcon chain={chain} size={28} />
                </span>
                <div className="chain-details">
                  <span className="chain-name">{meta.name}</span>
                  <span className="chain-ticker">{meta.ticker}</span>
                </div>
                {balance && (
                  <span className="chain-balance">
                    {balance.balance} {meta.ticker}
                  </span>
                )}
              </button>
            )
          })}
        </div>
      </motion.div>
    </motion.div>
  )
}
