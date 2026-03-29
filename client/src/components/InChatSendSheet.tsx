import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META, ChainId } from '../types/wallet'
import CryptoIcon from './CryptoIcon'
import {
  ArrowLeft, CheckCircle, AlertTriangle,
  ICON_SIZE,
} from '../ui/icons'

interface InChatSendSheetProps {
  isOpen: boolean
  onClose: () => void
  conversationId: string
  contactName: string
  contactWalletAddress?: string
}

// Step flow for the send sheet
type SendStep = 'amount' | 'confirm' | 'done'

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
    estimateGas,
  } = useWalletStore()

  const [amount, setAmount] = useState('')
  const [toAddress, setToAddress] = useState(contactWalletAddress || '')
  const [step, setStep] = useState<SendStep>('amount')
  const [txHash, setTxHash] = useState('')
  const [explorerUrl, setExplorerUrl] = useState('')
  const [addressError, setAddressError] = useState('')

  // Reset when sheet opens
  useEffect(() => {
    if (isOpen) {
      setStep('amount')
      setAmount('')
      setToAddress(contactWalletAddress || '')
      setTxHash('')
      setExplorerUrl('')
      setAddressError('')
    }
  }, [isOpen, contactWalletAddress])

  const meta = CHAIN_META[selectedChain]
  const balance = balances[selectedChain]
  // Use usdValue from balance if available for rate calculation
  const rate = (balance && parseFloat(balance.balance) > 0 && balance.usdValue)
    ? balance.usdValue / parseFloat(balance.balance)
    : 0
  const amountNum = parseFloat(amount) || 0
  const fiatValue = amountNum * rate
  const availableBalance = balance ? parseFloat(balance.balance) : 0
  const isOverBalance = amountNum > availableBalance
  const isReviewDisabled = !amount || amountNum <= 0 || (!contactWalletAddress && !toAddress.trim()) || isOverBalance || sending

  // Estimate network fee from backend (defaults shown initially)
  const [estimatedFeeUsd, setEstimatedFeeUsd] = useState(0)
  useEffect(() => {
    if (amountNum > 0 && toAddress) {
      estimateGas(selectedChain, toAddress, amount)
        .then(est => setEstimatedFeeUsd(est.standard.estimatedUsd))
        .catch(() => setEstimatedFeeUsd(0))
    }
  }, [selectedChain, amountNum > 0, toAddress])

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
    setAddressError('')
    onClose()
  }

  const validateAddress = (addr: string) => {
    if (!addr.trim()) {
      setAddressError('')
      return
    }
    if (addr.length < 20) {
      setAddressError('Address seems too short')
    } else {
      setAddressError('')
    }
  }

  const handleMaxAmount = () => {
    if (balance) {
      setAmount(balance.balance)
    }
  }

  const quickChains: ChainId[] = ['Bitcoin', 'Ethereum', 'Solana', 'Polygon']

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
            transition={{ type: 'spring', damping: 25, stiffness: 300 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="sheet-handle" />

            {/* ── Step: Amount Entry ── */}
            {step === 'amount' && (
              <div className="sheet-step-amount">
                {/* Header */}
                <div className="sheet-header-row">
                  <h3 className="sheet-title">Send to {contactName}</h3>
                </div>

                {/* Step Indicator */}
                <div className="sheet-steps-indicator">
                  <span className="step-dot active" />
                  <span className="step-line" />
                  <span className="step-dot" />
                  <span className="step-line" />
                  <span className="step-dot" />
                </div>

                {/* Chain Selector with Icons */}
                <div className="chain-chips-row">
                  {quickChains.map((chain) => {
                    const chainMeta = CHAIN_META[chain]
                    const isActive = selectedChain === chain
                    return (
                      <button
                        key={chain}
                        className={`chain-chip-v2 ${isActive ? 'active' : ''}`}
                        onClick={() => setSelectedChain(chain)}
                      >
                        <CryptoIcon chain={chain} size={18} />
                        <span>{chainMeta.ticker}</span>
                      </button>
                    )
                  })}
                </div>

                {/* Amount Input — Large, Central, Bold */}
                <div className="sheet-amount-section">
                  <label className="sheet-input-label">Amount</label>
                  <div className="sheet-amount-input-v2">
                    <input
                      type="number"
                      placeholder="0.00"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      autoFocus
                      className={isOverBalance ? 'error' : ''}
                    />
                    <div className="amount-suffix">
                      <CryptoIcon chain={selectedChain} size={20} />
                      <span className="sheet-ticker-label">{meta.ticker}</span>
                    </div>
                  </div>

                  {/* Fiat Equivalent */}
                  <div className="sheet-fiat-row">
                    <span className="fiat-value">
                      ≈ ${fiatValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                    </span>
                  </div>

                  {/* Balance */}
                  <div className="sheet-balance-row">
                    <span className="balance-label">Available:</span>
                    <span className={`balance-value ${isOverBalance ? 'error' : ''}`}>
                      {balance?.balance ?? '0.00'} {meta.ticker}
                    </span>
                    {balance && (
                      <button className="max-chip" onClick={handleMaxAmount}>MAX</button>
                    )}
                  </div>

                  {isOverBalance && (
                    <p className="sheet-error-text">
                      <AlertTriangle size={12} /> Insufficient balance
                    </p>
                  )}
                </div>

                {/* Address Input */}
                {!contactWalletAddress && (
                  <div className="sheet-address-section">
                    <label className="sheet-input-label">Recipient Address</label>
                    <input
                      type="text"
                      placeholder={`${contactName}'s ${meta.name} address`}
                      value={toAddress}
                      onChange={(e) => {
                        setToAddress(e.target.value)
                        validateAddress(e.target.value)
                      }}
                      className={`sheet-address-input-v2 ${addressError ? 'error' : ''}`}
                    />
                    {addressError && (
                      <p className="sheet-error-text small">{addressError}</p>
                    )}
                  </div>
                )}

                {/* Network Fee */}
                <div className="sheet-fee-row">
                  <span className="fee-label">Estimated Network Fee</span>
                  <span className="fee-value">~${estimatedFeeUsd.toFixed(4)}</span>
                </div>

                {/* Review Button */}
                <button
                  className={`sheet-review-btn ${isReviewDisabled ? 'disabled' : ''}`}
                  disabled={isReviewDisabled}
                  onClick={() => setStep('confirm')}
                >
                  Review Transaction
                </button>
              </div>
            )}

            {/* ── Step: Confirm ── */}
            {step === 'confirm' && (
              <div className="sheet-step-confirm">
                {/* Header */}
                <div className="sheet-header-row">
                  <button className="sheet-back-btn" onClick={() => setStep('amount')}>
                    <ArrowLeft size={ICON_SIZE.sm} />
                  </button>
                  <h3 className="sheet-title">Confirm Transfer</h3>
                </div>

                {/* Step Indicator */}
                <div className="sheet-steps-indicator">
                  <span className="step-dot completed" />
                  <span className="step-line completed" />
                  <span className="step-dot active" />
                  <span className="step-line" />
                  <span className="step-dot" />
                </div>

                {/* Summary Card */}
                <div className="confirm-card">
                  <div className="confirm-amount-display">
                    <CryptoIcon chain={selectedChain} size={36} />
                    <div className="confirm-amount-text">
                      <span className="confirm-amount-value">{amount} {meta.ticker}</span>
                      <span className="confirm-amount-fiat">
                        ≈ ${fiatValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                      </span>
                    </div>
                  </div>

                  <div className="confirm-details">
                    <div className="confirm-row-v2">
                      <span className="row-label">To</span>
                      <span className="row-value">{contactName}</span>
                    </div>
                    <div className="confirm-row-v2">
                      <span className="row-label">Network</span>
                      <span className="row-value network">
                        <CryptoIcon chain={selectedChain} size={14} />
                        {meta.name}
                      </span>
                    </div>
                    <div className="confirm-row-v2">
                      <span className="row-label">Network Fee</span>
                      <span className="row-value">~${estimatedFeeUsd.toFixed(4)}</span>
                    </div>
                    {!contactWalletAddress && toAddress && (
                      <div className="confirm-row-v2 address-row">
                        <span className="row-label">Address</span>
                        <span className="row-value mono">
                          {toAddress.slice(0, 10)}...{toAddress.slice(-6)}
                        </span>
                      </div>
                    )}
                  </div>
                </div>

                {/* Action Buttons */}
                <div className="confirm-actions-v2">
                  <button className="sheet-secondary-btn" onClick={() => setStep('amount')}>
                    Back
                  </button>
                  <button
                    className="sheet-confirm-btn"
                    disabled={sending}
                    onClick={handleSend}
                  >
                    {sending ? (
                      <motion.span
                        animate={{ rotate: 360 }}
                        transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                        style={{ display: 'inline-block' }}
                      >
                        ⟳
                      </motion.span>
                    ) : (
                      <>Send {meta.ticker}</>
                    )}
                  </button>
                </div>
              </div>
            )}

            {/* ── Step: Done ── */}
            {step === 'done' && (
              <motion.div
                className="sheet-step-done"
                initial={{ scale: 0.8, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
              >
                {/* Step Indicator */}
                <div className="sheet-steps-indicator">
                  <span className="step-dot completed" />
                  <span className="step-line completed" />
                  <span className="step-dot completed" />
                  <span className="step-line completed" />
                  <span className="step-dot completed" />
                </div>

                <div className="done-icon-ring">
                  <CheckCircle size={40} color="var(--accent-secure)" />
                </div>
                <h3 className="done-title">Sent!</h3>
                <p className="done-summary">
                  {amount} {meta.ticker} to {contactName}
                </p>
                <p className="done-fiat">
                  ≈ ${fiatValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                </p>
                {explorerUrl && (
                  <a href={explorerUrl} target="_blank" rel="noopener noreferrer" className="done-explorer-link">
                    View on Explorer →
                  </a>
                )}
                <button className="sheet-review-btn" onClick={handleClose}>
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
