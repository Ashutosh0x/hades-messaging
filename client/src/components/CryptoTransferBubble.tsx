import { motion } from 'framer-motion'
import { CHAIN_META, ChainId, CryptoTransferMessage } from '../types/wallet'
import CryptoIcon from './CryptoIcon'
import { CheckCircle, Clock, XCircle } from 'lucide-react'

interface CryptoTransferBubbleProps {
  transfer: CryptoTransferMessage
  isMine: boolean
}

export default function CryptoTransferBubble({
  transfer,
  isMine,
}: CryptoTransferBubbleProps) {
  const meta = CHAIN_META[transfer.chain]

  return (
    <motion.div
      className={`crypto-transfer-bubble ${isMine ? 'mine' : 'theirs'}`}
      initial={{ scale: 0.9, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
    >
      <div className="transfer-header">
        <CryptoIcon chain={transfer.chain} size={22} />
        <span className="transfer-label">
          {isMine ? 'You sent' : 'Received'}
        </span>
      </div>

      <div className="transfer-amount">
        <span className="amount-value">{transfer.amount}</span>
        <span className="amount-symbol" style={{ color: meta.color }}>
          {transfer.symbol}
        </span>
      </div>

      <div className="transfer-meta">
        <span className="transfer-network">{meta.name}</span>
        <span className={`transfer-status ${transfer.status}`}>
          {transfer.status === 'pending' && <><Clock size={11} /> Pending</>}
          {transfer.status === 'confirmed' && <><CheckCircle size={11} /> Confirmed</>}
          {transfer.status === 'failed' && <><XCircle size={11} /> Failed</>}
        </span>
      </div>

      <a
        href={transfer.explorerUrl}
        target="_blank"
        rel="noopener noreferrer"
        className="transfer-explorer"
        onClick={(e) => e.stopPropagation()}
      >
        View Transaction →
      </a>
    </motion.div>
  )
}
