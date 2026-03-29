import { useEffect } from 'react'
import { motion } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../store/walletStore'
import { CHAIN_META, ChainId } from '../types/wallet'
import CryptoIcon from '../components/CryptoIcon'
import { ArrowLeft, ArrowUpRight, ArrowDownLeft, Clock, CheckCircle, XCircle } from 'lucide-react'
import './Wallet.css'

export default function WalletHistory() {
  const navigate = useNavigate()
  const { transactions, fetchTransactions } = useWalletStore()

  useEffect(() => {
    fetchTransactions()
  }, [])

  return (
    <div className="wallet-history-screen">
      <div className="send-header">
        <button className="wallet-back-btn" onClick={() => navigate(-1)}>
          <ArrowLeft size={18} />
        </button>
        <h2>Transaction History</h2>
        <div />
      </div>

      {transactions.length === 0 ? (
        <div className="empty-state">
          <span className="empty-icon">📭</span>
          <p>No transactions yet</p>
        </div>
      ) : (
        <div className="tx-list">
          {transactions.map((tx) => {
            const meta = CHAIN_META[tx.chain as ChainId]
            const isSend = tx.fromAddress !== tx.toAddress

            return (
              <motion.a
                key={tx.txHash}
                className="tx-row"
                href={tx.explorerUrl || '#'}
                target="_blank"
                rel="noopener noreferrer"
                whileTap={{ scale: 0.98 }}
              >
                <div className="tx-icon" style={{ backgroundColor: meta?.color + '20' }}>
                  {isSend ? (
                    <ArrowUpRight size={18} color={meta?.color} />
                  ) : (
                    <ArrowDownLeft size={18} color={meta?.color} />
                  )}
                </div>
                <div className="tx-info">
                  <span className="tx-type">
                    {isSend ? 'Sent' : 'Received'} {meta?.ticker}
                  </span>
                  <span className="tx-address">
                    {isSend
                      ? `To: ${tx.toAddress.slice(0, 8)}...${tx.toAddress.slice(-4)}`
                      : `From: ${tx.fromAddress.slice(0, 8)}...${tx.fromAddress.slice(-4)}`}
                  </span>
                </div>
                <div className="tx-amount-col">
                  <span className={`tx-amount ${isSend ? 'negative' : 'positive'}`}>
                    {isSend ? '-' : '+'}{tx.amount} {tx.symbol}
                  </span>
                  <span className={`tx-status ${tx.status}`}>
                    {tx.status === 'pending' && <><Clock size={10} /> pending</>}
                    {tx.status === 'confirmed' && <><CheckCircle size={10} /> confirmed</>}
                    {tx.status === 'failed' && <><XCircle size={10} /> failed</>}
                  </span>
                </div>
              </motion.a>
            )
          })}
        </div>
      )}
    </div>
  )
}
