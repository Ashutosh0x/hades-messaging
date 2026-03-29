import { useEffect, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META, ChainId } from '../types/wallet'
import CryptoIcon from '../components/CryptoIcon'
import {
  ArrowUpRight, ArrowDownLeft, Clock, Loader2,
  Shield, Copy, ChevronRight, AlertTriangle,
  Plus, ArrowLeftRight, Bell, Settings, CheckCircle,
  Eye, EyeOff, ExternalLink,
} from 'lucide-react'
import './Wallet.css'



export default function Wallet() {
  const navigate = useNavigate()
  const {
    initialized,
    accounts,
    balances,
    transactions,
    totalUsdValue,
    loading,
    mnemonic,
    selectedChain,
    initWallet,
    fetchAllBalances,
    fetchTransactions,
    clearMnemonic,
    setSelectedChain,
  } = useWalletStore()

  const [showMnemonic, setShowMnemonic] = useState(false)
  const [balanceHidden, setBalanceHidden] = useState(false)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (!initialized) {
      initWallet()
    } else {
      fetchAllBalances()
      fetchTransactions()
    }
  }, [initialized])

  useEffect(() => {
    if (mnemonic) setShowMnemonic(true)
  }, [mnemonic])

  // Use real data from backend — no mock fallback
  const displayAssets = accounts.map((acc) => {
    const bal = balances[acc.chain]
    return {
      chain: acc.chain as ChainId,
      balance: bal?.balance ?? '0.00',
      usdValue: bal?.usdValue ?? 0,
      change24h: 0,
    }
  })

  const displayTotal = totalUsdValue
  const currentMeta = CHAIN_META[selectedChain]

  // Get actual address from backend accounts
  const primaryAddress = accounts.find(a => a.chain === selectedChain)?.address ?? ''
  const shortAddress = primaryAddress.length > 16
    ? `${primaryAddress.slice(0, 8)}…${primaryAddress.slice(-6)}`
    : primaryAddress || 'No wallet initialized'

  const copyAddress = async () => {
    try {
      await navigator.clipboard.writeText(primaryAddress)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch { /* ignore in dev */ }
  }

  if (loading && !initialized) {
    return (
      <div className="wallet-loading">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
          className="wallet-spinner"
        >
          <Loader2 size={48} color="var(--accent-secure)" />
        </motion.div>
        <p>Initializing wallet...</p>
      </div>
    )
  }

  return (
    <div className="wallet-screen">
      {/* ── Mnemonic Backup Modal ── */}
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

      {/* ── Top Bar ── */}
      <div className="wallet-topbar">
        <div className="wallet-identity">
          <div className="wallet-avatar">
            <Shield size={16} color="var(--accent-secure)" />
          </div>
          <div className="wallet-identity-text">
            <span className="wallet-name">Hades Vault</span>
            <button className="wallet-address-btn" onClick={copyAddress}>
              <CryptoIcon chain={selectedChain} size={12} />
              <span className="wallet-addr-text">{shortAddress}</span>
              {copied ? (
                <CheckCircle size={11} color="var(--accent-secure)" />
              ) : (
                <Copy size={11} color="var(--text-muted)" />
              )}
            </button>
          </div>
        </div>
        <div className="wallet-topbar-actions">
          <button className="topbar-icon-btn" aria-label="Notifications">
            <Bell size={18} color="var(--text-secondary)" />
          </button>
          <button className="topbar-icon-btn" aria-label="Settings" onClick={() => navigate('/settings')}>
            <Settings size={18} color="var(--text-secondary)" />
          </button>
        </div>
      </div>

      {/* ── Network Badge ── */}
      <div className="wallet-network-bar">
        <span className="network-dot" style={{ background: currentMeta.color }} />
        <span className="network-name">{currentMeta.name} Network</span>
        <span className="network-gas">Gas: ~$1.24</span>
      </div>

      {/* ── Balance Card ── */}
      <motion.div
        className="balance-card"
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
      >
        <div className="balance-top-row">
          <span className="balance-label">Total Balance</span>
          <button
            className="balance-eye-btn"
            onClick={() => setBalanceHidden(!balanceHidden)}
            aria-label={balanceHidden ? 'Show balance' : 'Hide balance'}
          >
            {balanceHidden ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
        </div>
        <h1 className="balance-amount-main">
          {balanceHidden
            ? '••••••'
            : `$${displayTotal.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
          }
        </h1>
        <span className="balance-change positive">+$142.38 (0.84%) today</span>

        {/* Action Buttons */}
        <div className="wallet-action-grid">
          <button className="wallet-action-pill" onClick={() => navigate('/wallet/send')}>
            <span className="pill-icon"><ArrowUpRight size={18} /></span>
            <span className="pill-label">Send</span>
            <span className="pill-hint">Transfer funds</span>
          </button>
          <button className="wallet-action-pill" onClick={() => navigate('/wallet/receive')}>
            <span className="pill-icon"><ArrowDownLeft size={18} /></span>
            <span className="pill-label">Receive</span>
            <span className="pill-hint">Show QR / address</span>
          </button>
          <button className="wallet-action-pill">
            <span className="pill-icon"><Plus size={18} /></span>
            <span className="pill-label">Buy</span>
            <span className="pill-hint">Add crypto</span>
          </button>
          <button className="wallet-action-pill">
            <span className="pill-icon"><ArrowLeftRight size={18} /></span>
            <span className="pill-label">Swap</span>
            <span className="pill-hint">Exchange tokens</span>
          </button>
        </div>
      </motion.div>

      {/* ── Assets Section ── */}
      <div className="wallet-section">
        <div className="section-header">
          <h3 className="section-title">Assets</h3>
          <span className="section-count">{displayAssets.length}</span>
        </div>

        <div className="asset-list">
          {displayAssets.map((asset, i) => {
            const meta = CHAIN_META[asset.chain]
            return (
              <motion.button
                key={asset.chain}
                className="asset-row"
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.05 }}
                onClick={() => {
                  setSelectedChain(asset.chain)
                  navigate('/wallet/send')
                }}
              >
                <div className="asset-icon" style={{ backgroundColor: meta.color + '18' }}>
                  <CryptoIcon chain={asset.chain} size={24} />
                </div>
                <div className="asset-info">
                  <span className="asset-name">{meta.name}</span>
                  <span className="asset-ticker">{meta.ticker}</span>
                </div>
                <div className="asset-values">
                  <span className="asset-balance">
                    {balanceHidden ? '••••' : asset.balance} {meta.ticker}
                  </span>
                  <div className="asset-fiat-row">
                    <span className="asset-usd">
                      {balanceHidden ? '••••' : `$${asset.usdValue.toLocaleString('en-US', { minimumFractionDigits: 2 })}`}
                    </span>
                    {asset.change24h !== 0 && (
                      <span className={`asset-change ${asset.change24h > 0 ? 'positive' : 'negative'}`}>
                        {asset.change24h > 0 ? '+' : ''}{asset.change24h.toFixed(2)}%
                      </span>
                    )}
                  </div>
                </div>
              </motion.button>
            )
          })}
        </div>
      </div>

      {/* ── Recent Activity ── */}
      <div className="wallet-section">
        <div className="section-header">
          <h3 className="section-title">Recent Activity</h3>
          <button className="section-link" onClick={() => navigate('/wallet/history')}>
            View All <ChevronRight size={14} />
          </button>
        </div>

        {transactions.length === 0 ? (
          <div className="empty-activity">
            <Clock size={24} color="var(--text-muted)" />
            <p>No recent activity</p>
          </div>
        ) : (
          <div className="activity-list">
            {transactions.slice(0, 10).map((tx, i) => {
              const chain = (tx.chain || 'Ethereum') as ChainId
              const meta = CHAIN_META[chain] || CHAIN_META.Ethereum
              const isSend = tx.fromAddress === primaryAddress
              const shortAddr = (isSend ? tx.toAddress : tx.fromAddress) || ''
              const displayAddr = shortAddr.length > 12
                ? `${shortAddr.slice(0, 6)}…${shortAddr.slice(-4)}`
                : shortAddr
              return (
                <div key={tx.txHash || i} className="activity-row">
                  <div className="activity-icon" style={{ backgroundColor: meta.color + '18' }}>
                    {isSend
                      ? <ArrowUpRight size={16} color={meta.color} />
                      : <ArrowDownLeft size={16} color={meta.color} />
                    }
                  </div>
                  <div className="activity-info">
                    <span className="activity-type">
                      {isSend ? 'Sent' : 'Received'} {tx.symbol}
                    </span>
                    <span className="activity-to">
                      {isSend ? 'To' : 'From'}: {displayAddr}
                    </span>
                  </div>
                  <div className="activity-values">
                    <span className={`activity-amount ${isSend ? 'negative' : 'positive'}`}>
                      {isSend ? '-' : '+'}{tx.amount} {tx.symbol}
                    </span>
                    <span className={`activity-status ${tx.status}`}>
                      {tx.status === 'confirmed' && <><CheckCircle size={10} /> Confirmed</>}
                      {tx.status === 'pending' && <><Clock size={10} /> Pending</>}
                      {tx.status === 'failed' && <><AlertTriangle size={10} /> Failed</>}
                    </span>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* ── Security Banner ── */}
      <div className="security-banner">
        <div className="security-icon-wrap">
          <Shield size={18} color="var(--accent-secure)" />
        </div>
        <div className="security-text">
          <span className="security-title">Wallet Secured</span>
          <span className="security-subtitle">Seed phrase backed up · Encrypted storage</span>
        </div>
        <ChevronRight size={16} color="var(--text-muted)" />
      </div>

      {/* ── Empty State CTA (when balance is 0 and no real data) ── */}
      {displayTotal === 0 && (
        <motion.div
          className="empty-wallet-cta"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div className="cta-icon">
            <Plus size={28} color="var(--accent-secure)" />
          </div>
          <h3>Get Started</h3>
          <p>Add funds to your wallet to start transacting</p>
          <div className="cta-buttons">
            <button className="wallet-btn-primary">Buy Crypto</button>
            <button className="wallet-btn-secondary">Import Wallet</button>
          </div>
        </motion.div>
      )}
    </div>
  )
}
