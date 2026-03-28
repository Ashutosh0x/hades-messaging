export type ChainId =
  | 'Bitcoin'
  | 'Ethereum'
  | 'Solana'
  | 'Polygon'
  | 'Arbitrum'
  | 'Optimism'
  | 'Avalanche'
  | 'Base'
  | 'BnbSmartChain'
  | 'Litecoin'
  | 'Dogecoin'
  | 'Tron';

export interface AccountInfo {
  chain: ChainId;
  address: string;
  ticker: string;
  chainName: string;
  derivationPath: string;
}

export interface BalanceResult {
  chain: ChainId;
  symbol: string;
  balance: string;
  usdValue: number | null;
}

export interface WalletTx {
  txHash: string;
  chain: ChainId;
  fromAddress: string;
  toAddress: string;
  amount: string;
  symbol: string;
  status: 'pending' | 'confirmed' | 'failed';
  explorerUrl: string | null;
  messageId: string | null;
  conversationId: string | null;
  timestamp: number;
}

export interface GasEstimate {
  slow: GasTier;
  standard: GasTier;
  fast: GasTier;
}

export interface GasTier {
  gasPriceGwei: string;
  estimatedSeconds: number;
  estimatedUsd: number;
}

export interface SendRequest {
  chain: ChainId;
  toAddress: string;
  amount: string;
  tokenContract?: string;
  conversationId?: string;
}

export interface CryptoTransferMessage {
  type: 'crypto_transfer';
  chain: ChainId;
  symbol: string;
  amount: string;
  to: string;
  txHash: string;
  explorerUrl: string;
  status: 'pending' | 'confirmed' | 'failed';
}

// Chain metadata for UI
export const CHAIN_META: Record<
  ChainId,
  { name: string; ticker: string; color: string; icon: string; decimals: number }
> = {
  Bitcoin: { name: 'Bitcoin', ticker: 'BTC', color: '#F7931A', icon: '₿', decimals: 8 },
  Ethereum: { name: 'Ethereum', ticker: 'ETH', color: '#627EEA', icon: 'Ξ', decimals: 18 },
  Solana: { name: 'Solana', ticker: 'SOL', color: '#9945FF', icon: '◎', decimals: 9 },
  Polygon: { name: 'Polygon', ticker: 'MATIC', color: '#8247E5', icon: '⬡', decimals: 18 },
  Arbitrum: { name: 'Arbitrum', ticker: 'ETH', color: '#28A0F0', icon: '⟠', decimals: 18 },
  Optimism: { name: 'Optimism', ticker: 'ETH', color: '#FF0420', icon: '⟠', decimals: 18 },
  Avalanche: { name: 'Avalanche', ticker: 'AVAX', color: '#E84142', icon: '△', decimals: 18 },
  Base: { name: 'Base', ticker: 'ETH', color: '#0052FF', icon: '⟠', decimals: 18 },
  BnbSmartChain: { name: 'BNB Chain', ticker: 'BNB', color: '#F3BA2F', icon: '◆', decimals: 18 },
  Litecoin: { name: 'Litecoin', ticker: 'LTC', color: '#BFBBBB', icon: 'Ł', decimals: 8 },
  Dogecoin: { name: 'Dogecoin', ticker: 'DOGE', color: '#C2A633', icon: 'Ð', decimals: 8 },
  Tron: { name: 'Tron', ticker: 'TRX', color: '#FF0013', icon: '◈', decimals: 6 },
};
