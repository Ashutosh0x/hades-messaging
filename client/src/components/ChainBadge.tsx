import { CHAIN_META, ChainId } from '../types/wallet'

interface ChainBadgeProps {
  chain: ChainId
  size?: 'sm' | 'md' | 'lg'
}

export default function ChainBadge({ chain, size = 'md' }: ChainBadgeProps) {
  const meta = CHAIN_META[chain]
  const sizeClass = `chain-badge chain-badge-${size}`

  return (
    <span
      className={sizeClass}
      style={{
        backgroundColor: meta.color + '15',
        color: meta.color,
        borderColor: meta.color + '30',
      }}
    >
      {meta.icon} {meta.ticker}
    </span>
  )
}
