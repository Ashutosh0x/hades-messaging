import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  AccountInfo,
  BalanceResult,
  ChainId,
  GasEstimate,
  SendRequest,
  WalletTx,
} from '../types/wallet';

interface WalletState {
  // State
  initialized: boolean;
  accounts: AccountInfo[];
  balances: Record<string, BalanceResult>;
  transactions: WalletTx[];
  mnemonic: string | null;
  loading: boolean;
  sending: boolean;
  error: string | null;
  selectedChain: ChainId;
  totalUsdValue: number;

  // Actions
  initWallet: () => Promise<void>;
  importWallet: (mnemonic: string) => Promise<void>;
  fetchAllBalances: () => Promise<void>;
  fetchBalance: (chain: ChainId) => Promise<void>;
  sendCrypto: (req: SendRequest) => Promise<{ txHash: string; explorerUrl: string; messageId?: string }>;
  estimateGas: (chain: ChainId, to: string, amount: string) => Promise<GasEstimate>;
  fetchTransactions: (chain?: ChainId) => Promise<void>;
  getAddress: (chain: ChainId) => Promise<string>;
  exportMnemonic: () => Promise<string>;
  setSelectedChain: (chain: ChainId) => void;
  clearMnemonic: () => void;
}

export const useWalletStore = create<WalletState>((set, get) => ({
  initialized: false,
  accounts: [],
  balances: {},
  transactions: [],
  mnemonic: null,
  loading: false,
  sending: false,
  error: null,
  selectedChain: 'Ethereum',
  totalUsdValue: 0,

  initWallet: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{
        accounts: AccountInfo[];
        mnemonic: string | null;
      }>('wallet_init');

      set({
        initialized: true,
        accounts: result.accounts,
        mnemonic: result.mnemonic,
        loading: false,
      });

      // Auto-fetch balances after init
      get().fetchAllBalances();
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  importWallet: async (mnemonic: string) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{
        accounts: AccountInfo[];
        mnemonic: string | null;
      }>('wallet_import', { mnemonic });

      set({
        initialized: true,
        accounts: result.accounts,
        loading: false,
      });

      get().fetchAllBalances();
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  fetchAllBalances: async () => {
    try {
      const results = await invoke<BalanceResult[]>('wallet_get_all_balances');
      const balances: Record<string, BalanceResult> = {};
      let total = 0;

      for (const b of results) {
        balances[b.chain] = b;
        if (b.usdValue) total += b.usdValue;
      }

      set({ balances, totalUsdValue: total });
    } catch (err) {
      console.error('Failed to fetch balances:', err);
    }
  },

  fetchBalance: async (chain: ChainId) => {
    try {
      const result = await invoke<BalanceResult>('wallet_get_balance', { chain });
      set((state) => ({
        balances: { ...state.balances, [chain]: result },
      }));
    } catch (err) {
      console.error(`Failed to fetch ${chain} balance:`, err);
    }
  },

  sendCrypto: async (req: SendRequest) => {
    set({ sending: true, error: null });
    try {
      const result = await invoke<{
        txHash: string;
        explorerUrl: string;
        status: string;
        messageId: string | null;
      }>('wallet_send', { request: req });

      // Refresh balance after send
      get().fetchBalance(req.chain);
      get().fetchTransactions();

      set({ sending: false });

      return {
        txHash: result.txHash,
        explorerUrl: result.explorerUrl,
        messageId: result.messageId ?? undefined,
      };
    } catch (err) {
      set({ sending: false, error: String(err) });
      throw err;
    }
  },

  estimateGas: async (chain, to, amount) => {
    return invoke<GasEstimate>('wallet_estimate_gas', { chain, to, amount });
  },

  fetchTransactions: async (chain?: ChainId) => {
    try {
      const txs = await invoke<WalletTx[]>('wallet_get_transactions', {
        chain: chain ?? null,
        limit: 100,
      });
      set({ transactions: txs });
    } catch (err) {
      console.error('Failed to fetch transactions:', err);
    }
  },

  getAddress: async (chain: ChainId) => {
    return invoke<string>('wallet_get_address', { chain });
  },

  exportMnemonic: async () => {
    return invoke<string>('wallet_export_mnemonic');
  },

  setSelectedChain: (chain) => set({ selectedChain: chain }),
  clearMnemonic: () => set({ mnemonic: null }),
}));
