import { useEffect, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META, ChainId } from '../types/wallet'
import './Wallet.css'

export default function Wallet() {
  const navigate = useNavigate()
  const {
    initialized,
    accounts,
    balances,
    totalUsdValue,
    loading,
    mnemonic,
    initWallet,
    fetchAllBalances,
    clearMnemonic,
  } = useWalletStore()

  const [showMnemonic, setShowMnemonic] = useState(false)

  useEffect(() => {
    if (!initialized) {
      initWallet()
    } else {
      fetchAllBalances()
    }
  }, [initialized])

  useEffect(() => {
    if (mnemonic) setShowMnemonic(true)
  }, [mnemonic])

  if (loading && !initialized) {
    return (
      <div className="wallet-loading">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
          className="wallet-spinner"
        >
          ◈
        </motion.div>
        <p>Initializing wallet...</p>
      </div>
    )
  }

  return (
    <div className="wallet-screen">
      {/* Mnemonic backup modal (shown once on creation) */}
      <AnimatePresence>
        {showMnemonic && mnemonic && (
          <motion.div
            className="mnemonic-overlay"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <motion.div
              className="mnemonic-modal"
              initial={{ scale: 0.8, y: 50 }}
              animate={{ scale: 1, y: 0 }}
            >
              <h2>🔐 Backup Your Seed Phrase</h2>
              <p className="mnemonic-warning">
                Write these words down and store them securely.
                Anyone with this phrase can access your funds.
              </p>
              <div className="mnemonic-grid">
                {mnemonic.split(' ').map((word, i) => (
                  <div key={i} className="mnemonic-word">
                    <span className="word-number">{i + 1}</span>
                    <span className="word-text">{word}</span>
                  </div>
                ))}
              </div>
              <button
                className="wallet-btn-primary"
                onClick={() => {
                  setShowMnemonic(false)
                  clearMnemonic()
                }}
              >
                I've saved my seed phrase
              </button>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Total Balance Header */}
      <div className="wallet-header">
        <p className="wallet-label">Total Balance</p>
        <motion.h1
          className="wallet-total"
          key={totalUsdValue}
          initial={{ scale: 0.9 }}
          animate={{ scale: 1 }}
        >
          ${totalUsdValue.toLocaleString('en-US', { minimumFractionDigits: 2 })}
        </motion.h1>

        <div className="wallet-actions">
          <button
            className="wallet-action-btn"
            onClick={() => navigate('/wallet/send')}
          >
            <span className="action-icon">↑</span>
            Send
          </button>
          <button
            className="wallet-action-btn"
            onClick={() => navigate('/wallet/receive')}
          >
            <span className="action-icon">↓</span>
            Receive
          </button>
          <button
            className="wallet-action-btn"
            onClick={() => navigate('/wallet/history')}
          >
            <span className="action-icon">⟳</span>
            History
          </button>
        </div>
      </div>

      {/* Token List */}
      <div className="wallet-tokens">
        <h3 className="section-title">Assets</h3>
        {accounts.map((account) => {
          const meta = CHAIN_META[account.chain as ChainId]
          const balance = balances[account.chain]

          return (
            <motion.div
              key={account.chain}
              className="token-row"
              whileTap={{ scale: 0.98 }}
              onClick={() => {
                useWalletStore.getState().setSelectedChain(account.chain as ChainId)
                navigate('/wallet/send')
              }}
            >
              <div className="token-icon" style={{ backgroundColor: meta?.color + '20' }}>
                <span style={{ color: meta?.color }}>{meta?.icon}</span>
              </div>
              <div className="token-info">
                <span className="token-name">{meta?.name || account.chain}</span>
                <span className="token-address">
                  {account.address.slice(0, 8)}...{account.address.slice(-6)}
                </span>
              </div>
              <div className="token-balance">
                <span className="balance-amount">
                  {balance?.balance ?? '—'} {meta?.ticker}
                </span>
                {balance?.usdValue && (
                  <span className="balance-usd">
                    ${balance.usdValue.toFixed(2)}
                  </span>
                )}
              </div>
            </motion.div>
          )
        })}
      </div>
    </div>
  )
}
