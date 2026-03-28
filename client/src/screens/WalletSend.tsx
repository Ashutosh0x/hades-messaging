import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { useWalletStore } from '../store/walletStore'
import TokenSelector from '../components/TokenSelector'
import { CHAIN_META, ChainId, GasEstimate } from '../types/wallet'
import './Wallet.css'

export default function WalletSend() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()

  const {
    selectedChain,
    balances,
    sending,
    error,
    sendCrypto,
    estimateGas,
    setSelectedChain,
  } = useWalletStore()

  const [toAddress, setToAddress] = useState(searchParams.get('to') || '')
  const [amount, setAmount] = useState('')
  const [showChainPicker, setShowChainPicker] = useState(false)
  const [gasEstimate, setGasEstimate] = useState<GasEstimate | null>(null)
  const [selectedSpeed, setSelectedSpeed] = useState<'slow' | 'standard' | 'fast'>('standard')
  const [step, setStep] = useState<'input' | 'confirm' | 'success'>('input')
  const [txResult, setTxResult] = useState<{ txHash: string; explorerUrl: string } | null>(null)

  const conversationId = searchParams.get('conversationId')
  const contactName = searchParams.get('contactName')

  const meta = CHAIN_META[selectedChain]
  const balance = balances[selectedChain]

  // Estimate gas when amount changes
  useEffect(() => {
    if (amount && toAddress && selectedChain) {
      const timer = setTimeout(async () => {
        try {
          const est = await estimateGas(selectedChain, toAddress, amount)
          setGasEstimate(est)
        } catch { /* ignore */ }
      }, 500)
      return () => clearTimeout(timer)
    }
  }, [amount, toAddress, selectedChain])

  const handleSend = async () => {
    try {
      const result = await sendCrypto({
        chain: selectedChain,
        toAddress,
        amount,
        conversationId: conversationId ?? undefined,
      })
      setTxResult(result)
      setStep('success')
    } catch { /* error handled by store */ }
  }

  const handleMaxAmount = () => {
    if (balance) {
      setAmount(balance.balance)
    }
  }

  if (step === 'success' && txResult) {
    return (
      <div className="wallet-send-screen">
        <motion.div
          className="send-success"
          initial={{ scale: 0.5, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
        >
          <div className="success-icon">✓</div>
          <h2>Transaction Sent</h2>
          <p className="tx-hash">
            {txResult.txHash.slice(0, 12)}...{txResult.txHash.slice(-8)}
          </p>
          <a
            href={txResult.explorerUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="explorer-link"
          >
            View on Explorer →
          </a>
          <button
            className="wallet-btn-primary"
            onClick={() => navigate(-1)}
          >
            Done
          </button>
        </motion.div>
      </div>
    )
  }

  return (
    <div className="wallet-send-screen">
      {/* Header */}
      <div className="send-header">
        <button className="wallet-back-btn" onClick={() => navigate(-1)}>←</button>
        <h2>Send {meta.ticker}</h2>
        <div />
      </div>

      {/* Chain Selector */}
      <button
        className="chain-selector-btn"
        onClick={() => setShowChainPicker(true)}
      >
        <span style={{ color: meta.color }}>{meta.icon}</span>
        <span>{meta.name}</span>
        <span className="chevron">▾</span>
      </button>

      {showChainPicker && (
        <TokenSelector
          onSelect={(chain) => {
            setSelectedChain(chain)
            setShowChainPicker(false)
          }}
          onClose={() => setShowChainPicker(false)}
        />
      )}

      {step === 'input' ? (
        <>
          {/* Recipient */}
          <div className="input-group">
            <label>To</label>
            {contactName ? (
              <div className="contact-recipient">
                <span className="contact-avatar">
                  {contactName.charAt(0).toUpperCase()}
                </span>
                <span>{contactName}</span>
              </div>
            ) : (
              <input
                type="text"
                placeholder={`${meta.name} address`}
                value={toAddress}
                onChange={(e) => setToAddress(e.target.value)}
                className="address-input"
              />
            )}
          </div>

          {/* Amount */}
          <div className="input-group">
            <label>Amount</label>
            <div className="amount-input-row">
              <input
                type="number"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="amount-input"
                step="any"
              />
              <span className="amount-ticker">{meta.ticker}</span>
              <button className="max-btn" onClick={handleMaxAmount}>MAX</button>
            </div>
            {balance && (
              <p className="balance-hint">
                Available: {balance.balance} {meta.ticker}
              </p>
            )}
          </div>

          {/* Gas Estimate */}
          {gasEstimate && (
            <div className="gas-section">
              <label>Network Fee</label>
              <div className="gas-tiers">
                {(['slow', 'standard', 'fast'] as const).map((speed) => {
                  const tier = gasEstimate[speed]
                  return (
                    <button
                      key={speed}
                      className={`gas-tier ${selectedSpeed === speed ? 'selected' : ''}`}
                      onClick={() => setSelectedSpeed(speed)}
                    >
                      <span className="speed-label">{speed}</span>
                      <span className="speed-time">~{tier.estimatedSeconds}s</span>
                      <span className="speed-cost">${tier.estimatedUsd.toFixed(4)}</span>
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          {error && <p className="error-text">{error}</p>}

          <button
            className="wallet-btn-primary send-btn"
            disabled={!toAddress || !amount || sending}
            onClick={() => setStep('confirm')}
          >
            Review
          </button>
        </>
      ) : (
        /* Confirmation Step */
        <motion.div
          className="confirm-section"
          initial={{ y: 30, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
        >
          <div className="confirm-detail">
            <span>Sending</span>
            <strong>{amount} {meta.ticker}</strong>
          </div>
          <div className="confirm-detail">
            <span>To</span>
            <strong className="confirm-address">
              {contactName || `${toAddress.slice(0, 10)}...${toAddress.slice(-8)}`}
            </strong>
          </div>
          <div className="confirm-detail">
            <span>Network</span>
            <strong>{meta.name}</strong>
          </div>
          {gasEstimate && (
            <div className="confirm-detail">
              <span>Fee</span>
              <strong>${gasEstimate[selectedSpeed].estimatedUsd.toFixed(4)}</strong>
            </div>
          )}

          <div className="confirm-buttons">
            <button className="wallet-btn-secondary" onClick={() => setStep('input')}>
              Back
            </button>
            <button
              className="wallet-btn-primary"
              disabled={sending}
              onClick={handleSend}
            >
              {sending ? (
                <motion.span
                  animate={{ rotate: 360 }}
                  transition={{ duration: 1, repeat: Infinity }}
                >
                  ⟳
                </motion.span>
              ) : (
                'Confirm & Send'
              )}
            </button>
          </div>
        </motion.div>
      )}
    </div>
  )
}
