import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META } from '../types/wallet'
import TokenSelector from '../components/TokenSelector'
import './Wallet.css'

export default function WalletReceive() {
  const navigate = useNavigate()
  const { selectedChain, setSelectedChain, getAddress } = useWalletStore()
  const [address, setAddress] = useState('')
  const [copied, setCopied] = useState(false)
  const [showPicker, setShowPicker] = useState(false)

  const meta = CHAIN_META[selectedChain]

  useEffect(() => {
    getAddress(selectedChain).then(setAddress).catch(console.error)
  }, [selectedChain])

  const copyAddress = async () => {
    await navigator.clipboard.writeText(address)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="wallet-receive-screen">
      <div className="send-header">
        <button className="wallet-back-btn" onClick={() => navigate(-1)}>←</button>
        <h2>Receive {meta.ticker}</h2>
        <div />
      </div>

      <button
        className="chain-selector-btn"
        onClick={() => setShowPicker(true)}
      >
        <span style={{ color: meta.color }}>{meta.icon}</span>
        <span>{meta.name}</span>
        <span className="chevron">▾</span>
      </button>

      {showPicker && (
        <TokenSelector
          onSelect={(chain) => {
            setSelectedChain(chain)
            setShowPicker(false)
          }}
          onClose={() => setShowPicker(false)}
        />
      )}

      {/* QR placeholder */}
      <motion.div
        className="qr-container"
        initial={{ scale: 0.8 }}
        animate={{ scale: 1 }}
      >
        <div
          className="qr-placeholder"
          style={{ borderColor: meta.color }}
        >
          <span className="qr-icon" style={{ color: meta.color }}>
            {meta.icon}
          </span>
          <p className="qr-text">QR Code</p>
          <p className="qr-hint">{meta.name} address</p>
        </div>
      </motion.div>

      <div className="address-display">
        <p className="address-text">{address}</p>
        <motion.button
          className="copy-btn"
          whileTap={{ scale: 0.9 }}
          onClick={copyAddress}
        >
          {copied ? '✓ Copied!' : '⧉ Copy Address'}
        </motion.button>
      </div>

      <p className="receive-warning">
        Only send {meta.ticker} on the {meta.name} network to this address.
        Sending other tokens may result in permanent loss.
      </p>
    </div>
  )
}
