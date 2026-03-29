import { CHAIN_META, ChainId } from '../types/wallet'
import CryptoIcon from './CryptoIcon'

interface ChainBadgeProps {
  chain: ChainId
  size?: 'sm' | 'md' | 'lg'
}

export default function ChainBadge({ chain, size = 'md' }: ChainBadgeProps) {
  const meta = CHAIN_META[chain]
  const sizeClass = `chain-badge chain-badge-${size}`
  const iconSize = size === 'sm' ? 12 : size === 'md' ? 16 : 20

  return (
    <span
      className={sizeClass}
      style={{
        backgroundColor: meta.color + '15',
        color: meta.color,
        borderColor: meta.color + '30',
      }}
    >
      <CryptoIcon chain={chain} size={iconSize} /> {meta.ticker}
    </span>
  )
}
