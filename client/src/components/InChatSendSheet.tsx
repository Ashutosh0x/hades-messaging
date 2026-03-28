import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META, ChainId } from '../types/wallet'

interface InChatSendSheetProps {
  isOpen: boolean
  onClose: () => void
  conversationId: string
  contactName: string
  contactWalletAddress?: string
}

export default function InChatSendSheet({
  isOpen,
  onClose,
  conversationId,
  contactName,
  contactWalletAddress,
}: InChatSendSheetProps) {
  const {
    selectedChain,
    balances,
    sending,
    setSelectedChain,
    sendCrypto,
  } = useWalletStore()

  const [amount, setAmount] = useState('')
  const [toAddress, setToAddress] = useState(contactWalletAddress || '')
  const [step, setStep] = useState<'amount' | 'confirm' | 'done'>('amount')
  const [txHash, setTxHash] = useState('')
  const [explorerUrl, setExplorerUrl] = useState('')

  const meta = CHAIN_META[selectedChain]
  const balance = balances[selectedChain]

  const handleSend = async () => {
    try {
      const result = await sendCrypto({
        chain: selectedChain,
        toAddress,
        amount,
        conversationId,
      })
      setTxHash(result.txHash)
      setExplorerUrl(result.explorerUrl)
      setStep('done')
    } catch (err) {
      console.error('Send failed:', err)
    }
  }

  const handleClose = () => {
    setStep('amount')
    setAmount('')
    setTxHash('')
    setExplorerUrl('')
    onClose()
  }

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          className="inchat-sheet-overlay"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={handleClose}
        >
          <motion.div
            className="inchat-sheet"
            initial={{ y: '100%' }}
            animate={{ y: 0 }}
            exit={{ y: '100%' }}
            transition={{ type: 'spring', damping: 25 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="sheet-handle" />

            {step === 'amount' && (
              <>
                <h3>Send Crypto to {contactName}</h3>

                {/* Quick chain chips */}
                <div className="chain-chips">
                  {(['Bitcoin', 'Ethereum', 'Solana', 'Polygon'] as ChainId[]).map(
                    (chain) => (
                      <button
                        key={chain}
                        className={`chain-chip ${selectedChain === chain ? 'active' : ''}`}
                        style={{
                          borderColor: selectedChain === chain ? CHAIN_META[chain].color : 'transparent',
                        }}
                        onClick={() => setSelectedChain(chain)}
                      >
                        {CHAIN_META[chain].icon} {CHAIN_META[chain].ticker}
                      </button>
                    )
                  )}
                </div>

                {/* Amount */}
                <div className="sheet-amount-input">
                  <input
                    type="number"
                    placeholder="0.00"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    autoFocus
                  />
                  <span className="sheet-ticker">{meta.ticker}</span>
                </div>
                {balance && (
                  <p className="sheet-balance">
                    Balance: {balance.balance} {meta.ticker}
                  </p>
                )}

                {/* Address (if not provided by contact) */}
                {!contactWalletAddress && (
                  <input
                    type="text"
                    placeholder={`${contactName}'s ${meta.name} address`}
                    value={toAddress}
                    onChange={(e) => setToAddress(e.target.value)}
                    className="sheet-address-input"
                  />
                )}

                <button
                  className="wallet-btn-primary sheet-send-btn"
                  disabled={!amount || !toAddress || sending}
                  onClick={() => setStep('confirm')}
                >
                  Review
                </button>
              </>
            )}

            {step === 'confirm' && (
              <div className="sheet-confirm">
                <h3>Confirm Transfer</h3>
                <div className="confirm-row">
                  <span>Send</span>
                  <strong>
                    {amount} {meta.ticker}
                  </strong>
                </div>
                <div className="confirm-row">
                  <span>To</span>
                  <strong>{contactName}</strong>
                </div>
                <div className="confirm-row">
                  <span>Network</span>
                  <strong>{meta.name}</strong>
                </div>

                <div className="sheet-buttons">
                  <button className="wallet-btn-secondary" onClick={() => setStep('amount')}>
                    Back
                  </button>
                  <button
                    className="wallet-btn-primary"
                    disabled={sending}
                    onClick={handleSend}
                  >
                    {sending ? '⟳ Sending...' : `Send ${meta.ticker}`}
                  </button>
                </div>
              </div>
            )}

            {step === 'done' && (
              <motion.div
                className="sheet-done"
                initial={{ scale: 0.8 }}
                animate={{ scale: 1 }}
              >
                <span className="done-icon">✓</span>
                <h3>Sent!</h3>
                <p>
                  {amount} {meta.ticker} to {contactName}
                </p>
                <a href={explorerUrl} target="_blank" rel="noopener noreferrer">
                  View on Explorer →
                </a>
                <button className="wallet-btn-primary" onClick={handleClose}>
                  Done
                </button>
              </motion.div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
